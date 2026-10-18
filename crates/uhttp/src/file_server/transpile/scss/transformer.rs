use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::thread::JoinHandle;

use crossbeam::channel::Sender;
use dyn_fs::asynch::AsyncFileSystem;
use grass::OutputStyle;
use tokio::sync::oneshot;

use super::fs::GrassFs;

struct ScssTransformerEvent {
  path: PathBuf,
  kind: String,
  tx: oneshot::Sender<anyhow::Result<String>>,
}

/// The dedicated thread a [`ScssTransformer`] compiles on, along with the
/// channel that feeds it.
struct ScssTransformerWorker {
  tx: Sender<ScssTransformerEvent>,
  _handles: Vec<std::thread::JoinHandle<()>>,
}

pub struct ScssTransformer {
  fs: Arc<dyn AsyncFileSystem>,
  root_dir: PathBuf,
  output_style: OutputStyle,
  worker_threads: usize,
  worker: OnceLock<io::Result<ScssTransformerWorker>>,
}

impl ScssTransformer {
  pub fn new(
    fs: &Arc<dyn AsyncFileSystem>,
    root_dir: &Path,
    output_style: OutputStyle,
    worker_threads: usize,
  ) -> Self {
    Self {
      fs: Arc::clone(fs),
      root_dir: root_dir.to_path_buf(),
      output_style,
      worker_threads: worker_threads.max(1),
      worker: OnceLock::new(),
    }
  }

  fn worker(&self) -> io::Result<&ScssTransformerWorker> {
    self
      .worker
      .get_or_init(|| {
        let (tx, rx) = crossbeam::channel::unbounded::<ScssTransformerEvent>();

        let fs = Arc::new(GrassFs::new(
          &self.fs,
          &self.root_dir,
          tokio::runtime::Handle::current(),
        ));
        let root_dir = self.root_dir.clone();
        let output_style = self.output_style;

        let mut handles = Vec::<JoinHandle<()>>::new();

        for i in 0..self.worker_threads {
          let rx = rx.clone();
          let fs = fs.clone();
          let root_dir = root_dir.clone();

          handles.push(
            std::thread::Builder::new()
              .name(format!("scss-transformer-{}", i))
              .spawn(move || {
                while let Ok(ScssTransformerEvent { path, kind, tx }) = rx.recv() {
                  let _ = tx.send('block: {
                    let mut code =
                      match super::transpile::transpile(&fs, &path, &kind, output_style) {
                        Ok(code) => code,
                        Err(error) => break 'block Err(anyhow::anyhow!("{}", error)),
                      };

                    for (specifier, replacement) in
                      super::transpile::candidates(&code, &path, &root_dir)
                    {
                      code =
                        code.replace(&format!("\"{specifier}\""), &format!("\"{replacement}\""));
                      code = code.replace(&format!("'{specifier}'"), &format!("'{replacement}'"));
                    }

                    Ok(code)
                  });
                }
              })?,
          );
        }

        Ok(ScssTransformerWorker {
          tx,
          _handles: handles,
        })
      })
      .as_ref()
      .map_err(|error| io::Error::new(error.kind(), error.to_string()))
  }

  pub async fn transpile(
    &self,
    path: &Path,
    kind: &str,
  ) -> io::Result<String> {
    let (tx, rx) = oneshot::channel();
    let _ = self.worker()?.tx.send(ScssTransformerEvent {
      path: path.to_path_buf(),
      kind: kind.to_string(),
      tx,
    });

    match rx.await {
      Ok(Ok(code)) => Ok(code),
      Ok(Err(error)) => Err(io::Error::other(error)),
      Err(error) => Err(io::Error::other(error)),
    }
  }
}
