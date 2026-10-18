use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use dyn_fs::asynch::AsyncFileSystem;
use grass::OutputStyle;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::oneshot;

use super::fs::GrassFs;

struct ScssTransformerEvent {
  path: PathBuf,
  kind: String,
  tx: oneshot::Sender<anyhow::Result<String>>,
}

pub struct ScssTransformer {
  tx: UnboundedSender<ScssTransformerEvent>,
  _handle: std::thread::JoinHandle<()>,
}

impl ScssTransformer {
  pub fn new(
    fs: &Arc<dyn AsyncFileSystem>,
    root_dir: &Path,
    output_style: OutputStyle,
  ) -> std::io::Result<Self> {
    let (tx, mut rx) = unbounded_channel();

    let fs = GrassFs::new(
      fs,
      root_dir,
      match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle,
        Err(err) => return Err(std::io::Error::other(err)),
      },
    );

    let root_dir = root_dir.to_path_buf();

    let handle = std::thread::Builder::new()
      .name("scss-transformer".to_string())
      .spawn(move || {
        while let Some(ScssTransformerEvent { path, kind, tx }) = rx.blocking_recv() {
          let _ = tx.send('block: {
            let mut code = match super::transpile::transpile(&fs, &path, &kind, output_style) {
              Ok(code) => code,
              Err(error) => break 'block Err(anyhow::anyhow!("{}", error)),
            };

            for (specifier, replacement) in super::transpile::candidates(&code, &path, &root_dir) {
              code = code.replace(&format!("\"{specifier}\""), &format!("\"{replacement}\""));
              code = code.replace(&format!("'{specifier}'"), &format!("'{replacement}'"));
            }

            Ok(code)
          });
        }
      });

    Ok(Self {
      tx,
      _handle: handle.expect("failed to spawn the scss transformer thread"),
    })
  }

  pub async fn transpile(
    &self,
    path: &Path,
    kind: &str,
  ) -> Option<String> {
    let (tx, rx) = oneshot::channel();
    let _ = self.tx.send(ScssTransformerEvent {
      path: path.to_path_buf(),
      kind: kind.to_string(),
      tx,
    });
    match rx.await {
      Ok(Ok(code)) => Some(code),
      Ok(Err(_)) => None,
      Err(_) => None,
    }
  }
}
