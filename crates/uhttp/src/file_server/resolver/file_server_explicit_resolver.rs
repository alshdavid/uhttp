use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use dyn_fs::asynch::AsyncFileSystem;

use super::file_server_resolver::ResolveResult;

pub struct FileServerExplicitResolver {
  fs: Arc<dyn AsyncFileSystem>,
  root_dir: PathBuf,
}

impl FileServerExplicitResolver {
  pub fn new(
    fs: Arc<dyn AsyncFileSystem>,
    root_dir: PathBuf,
  ) -> Self {
    Self { fs, root_dir }
  }

  pub async fn resolve(
    &self,
    specifier: &str,
  ) -> io::Result<ResolveResult> {
    let path = self.root_dir.join(specifier.trim_start_matches('/'));

    match self.fs.metadata(&path).await {
      Ok(metadata) if metadata.is_file() => Ok(ResolveResult::from_path(path)),
      Ok(_) => Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Unable to resolve file",
      )),
      Err(e) => Err(e),
    }
  }

  pub fn resolve_exact(
    &self,
    specifier: &str,
  ) -> ResolveResult {
    let path = self.root_dir.join(specifier.trim_start_matches('/'));
    ResolveResult::from_path(path)
  }
}
