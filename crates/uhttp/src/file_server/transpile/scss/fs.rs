use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::channel;

use dyn_fs::asynch::AsyncFileSystem;

pub struct GrassFs {
  fs: Arc<dyn AsyncFileSystem>,
  tokio: tokio::runtime::Handle,
}

impl std::fmt::Debug for GrassFs {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    f.debug_tuple("GrassFs").finish()
  }
}

impl GrassFs {
  pub fn new(
    fs: &Arc<dyn AsyncFileSystem>,
    tokio: tokio::runtime::Handle,
  ) -> Self {
    Self {
      fs: Arc::clone(fs),
      tokio,
    }
  }
}

impl grass::Fs for GrassFs {
  fn is_dir(
    &self,
    path: &std::path::Path,
  ) -> bool {
    let (tx, rx) = channel::<bool>();
    self.tokio.spawn({
      let fs = self.fs.clone();
      let path = path.to_path_buf();
      async move {
        let _ = tx.send(match fs.metadata(&path).await {
          Ok(metadata) => metadata.is_dir(),
          Err(_) => false,
        });
      }
    });

    rx.recv().unwrap_or_default()
  }

  fn is_file(
    &self,
    path: &std::path::Path,
  ) -> bool {
    let (tx, rx) = channel::<bool>();
    self.tokio.spawn({
      let fs = self.fs.clone();
      let path = path.to_path_buf();
      async move {
        let _ = tx.send(match fs.metadata(&path).await {
          Ok(metadata) => metadata.is_file(),
          Err(_) => false,
        });
      }
    });

    rx.recv().unwrap_or_default()
  }

  fn read(
    &self,
    path: &std::path::Path,
  ) -> std::io::Result<Vec<u8>> {
    let (tx, rx) = channel::<std::io::Result<Vec<u8>>>();
    self.tokio.spawn({
      let fs = self.fs.clone();
      let path = path.to_path_buf();
      async move {
        let _ = tx.send(fs.read(&path).await);
      }
    });

    match rx.recv() {
      Ok(r) => r,
      Err(_) => Err(io::Error::other("Error reading file")),
    }
  }

  fn canonicalize(
    &self,
    path: &Path,
  ) -> io::Result<PathBuf> {
    self.tokio.block_on(self.fs.canonicalize(path))
  }
}
