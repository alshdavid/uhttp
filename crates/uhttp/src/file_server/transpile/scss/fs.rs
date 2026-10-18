use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use dyn_fs::asynch::AsyncFileSystem;
use normalize_path::NormalizePath;

pub struct GrassFs {
  fs: Arc<dyn AsyncFileSystem>,
  tokio: tokio::runtime::Handle,
  root_dir: PathBuf,
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
    root_dir: &Path,
    tokio: tokio::runtime::Handle,
  ) -> Self {
    Self {
      fs: Arc::clone(fs),
      tokio,
      root_dir: root_dir.to_path_buf(),
    }
  }

  /// `grass` resolves `@use`/`@import` through this file system directly,
  /// bypassing the request resolver, so confinement to the served directory has
  /// to be enforced here. Without it a stylesheet could read any path on the
  /// host, for example `@use "../../../../etc/passwd"`.
  fn is_allowed(
    &self,
    path: &Path,
  ) -> bool {
    path.normalize().starts_with(&self.root_dir)
  }

  /// Refuse reads outside the served directory with the error `grass` expects
  /// for a missing file, so the import simply fails to resolve.
  fn guard(
    &self,
    path: &Path,
  ) -> io::Result<()> {
    if self.is_allowed(path) {
      return Ok(());
    }

    Err(io::Error::new(
      io::ErrorKind::NotFound,
      format!("{} is outside of the served directory", path.display()),
    ))
  }
}

impl grass::Fs for GrassFs {
  fn is_dir(
    &self,
    path: &std::path::Path,
  ) -> bool {
    self.is_allowed(path)
      && self.tokio.block_on(async {
        match self.fs.metadata(path).await {
          Ok(metadata) => metadata.is_dir(),
          Err(_) => false,
        }
      })
  }

  fn is_file(
    &self,
    path: &std::path::Path,
  ) -> bool {
    self.is_allowed(path)
      && self.tokio.block_on(async {
        match self.fs.metadata(path).await {
          Ok(metadata) => metadata.is_file(),
          Err(_) => false,
        }
      })
  }

  fn read(
    &self,
    path: &std::path::Path,
  ) -> std::io::Result<Vec<u8>> {
    self.guard(path)?;
    self.tokio.block_on(self.fs.read(path))
  }

  fn canonicalize(
    &self,
    path: &Path,
  ) -> io::Result<PathBuf> {
    self.guard(path)?;
    self.tokio.block_on(self.fs.canonicalize(path))
  }
}
