use std::io;
use std::path::PathBuf;

use super::file_server_resolver::ResolveResult;

pub struct FileServerExplicitResolver {
  root_dir: PathBuf,
}

impl FileServerExplicitResolver {
  pub fn new(root_dir: PathBuf) -> Self {
    Self { root_dir }
  }

  pub fn resolve_exact(
    &self,
    specifier: &str,
  ) -> ResolveResult {
    let path = self.root_dir.join(specifier.trim_start_matches('/'));
    ResolveResult::from_path(path)
  }
}
