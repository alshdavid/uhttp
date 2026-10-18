use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::thread::JoinHandle;

use crossbeam::channel::Sender;
use dyn_fs::asynch::AsyncFileSystem;
use tokio::sync::oneshot;

use super::OptimizeOptions;
use super::TransformerContext;
use super::TransformerResult;
use super::helpers;
use super::javascript;
use super::tsconfig::ConfigFs;
use super::tsconfig::TsConfigCache;

enum TypeScriptTransformerEvent {
  Transpile {
    content: Vec<u8>,
    path: PathBuf,
    source_url: String,
    kind: String,
    source_maps: bool,
    tx: oneshot::Sender<crate::Result<TransformerResult>>,
  },
  Optimize {
    source: String,
    tx: oneshot::Sender<crate::Result<String>>,
  },
}

struct TypeScriptTransformerWorker {
  tx: Sender<TypeScriptTransformerEvent>,
  _handles: Vec<JoinHandle<()>>,
}

pub struct TypeScriptTransformer {
  fs: Arc<dyn AsyncFileSystem>,
  root_dir: PathBuf,
  optimize: OptimizeOptions,
  tsconfig_cache: Arc<TsConfigCache>,
  worker_threads: usize,
  worker: OnceLock<io::Result<TypeScriptTransformerWorker>>,
}

impl TypeScriptTransformer {
  pub fn new(
    fs: &Arc<dyn AsyncFileSystem>,
    root_dir: &Path,
    optimize: OptimizeOptions,
    worker_threads: usize,
  ) -> Self {
    Self {
      fs: Arc::clone(fs),
      root_dir: root_dir.to_path_buf(),
      optimize,
      tsconfig_cache: Arc::new(TsConfigCache::new()),
      worker_threads: worker_threads.max(1),
      worker: OnceLock::new(),
    }
  }

  pub fn helper_module_name(path: &str) -> Option<&str> {
    helpers::module_name(path)
  }

  pub fn helper_module_source(name: &str) -> Option<String> {
    helpers::module_source(name)
  }

  pub async fn transpile(
    &self,
    path: &Path,
    source_url: &str,
    kind: &str,
    source_maps: bool,
  ) -> crate::Result<Option<TransformerResult>> {
    let Ok(content) = self.fs.read(path).await else {
      return Ok(None);
    };

    let (tx, rx) = oneshot::channel();

    self
      .worker()?
      .tx
      .send(TypeScriptTransformerEvent::Transpile {
        content,
        path: path.to_path_buf(),
        source_url: source_url.to_string(),
        kind: kind.to_string(),
        source_maps,
        tx,
      })
      .map_err(|_| io::Error::other("the TypeScript transpiler is not running"))?;

    match rx.await {
      Ok(result) => result.map(Some),
      Err(error) => Err(io::Error::other(error).into()),
    }
  }

  pub async fn optimize_module(
    &self,
    source: &str,
  ) -> crate::Result<String> {
    let (tx, rx) = oneshot::channel();

    self
      .worker()?
      .tx
      .send(TypeScriptTransformerEvent::Optimize {
        source: source.to_string(),
        tx,
      })
      .map_err(|_| io::Error::other("the TypeScript transpiler is not running"))?;

    match rx.await {
      Ok(result) => result,
      Err(error) => Err(io::Error::other(error).into()),
    }
  }

  fn worker(&self) -> io::Result<&TypeScriptTransformerWorker> {
    self
      .worker
      .get_or_init(|| {
        let (tx, rx) = crossbeam::channel::unbounded::<TypeScriptTransformerEvent>();

        let config_fs = ConfigFs::new(Arc::clone(&self.fs), tokio::runtime::Handle::current());
        let tsconfig_cache = Arc::clone(&self.tsconfig_cache);
        let root_dir = self.root_dir.clone();
        let optimize = self.optimize;

        let mut handles = Vec::<JoinHandle<()>>::new();

        for i in 0..self.worker_threads {
          let rx = rx.clone();
          let config_fs = config_fs.clone();
          let tsconfig_cache = Arc::clone(&tsconfig_cache);
          let root_dir = root_dir.clone();

          handles.push(
            std::thread::Builder::new()
              .name(format!("typescript-transformer-{}", i))
              .spawn(move || {
                while let Ok(event) = rx.recv() {
                  match event {
                    TypeScriptTransformerEvent::Transpile {
                      content,
                      path,
                      source_url,
                      kind,
                      source_maps,
                      tx,
                    } => {
                      let tsconfig = tsconfig_cache.resolve(&config_fs, &path, &root_dir);

                      let _ = tx.send(super::transpile::transpile(TransformerContext {
                        content,
                        path,
                        source_url,
                        source_maps,
                        kind,
                        tsconfig,
                        optimize,
                      }));
                    }
                    TypeScriptTransformerEvent::Optimize { source, tx } => {
                      let _ = tx.send(javascript::optimize_module(&source, optimize));
                    }
                  }
                }
              })?,
          );
        }

        Ok(TypeScriptTransformerWorker {
          tx,
          _handles: handles,
        })
      })
      .as_ref()
      .map_err(|error| io::Error::new(error.kind(), error.to_string()))
  }
}
