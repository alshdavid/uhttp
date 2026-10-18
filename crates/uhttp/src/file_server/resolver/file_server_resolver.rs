use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use dyn_fs::asynch::AsyncFileSystem;
use dyn_fs::ext::PathExt;

use super::super::mime::detect_mime;
use super::FileServerExplicitResolver;
use super::FileServerImplicitResolver;

pub struct ResolveResult {
  pub path: PathBuf,
  pub ext: String,
  pub mime_type: String,
}

impl ResolveResult {
  pub(super) fn from_path(path: PathBuf) -> Self {
    let ext = path.try_extension().unwrap_or_default();
    let mime_type = detect_mime(&ext).to_string();
    Self {
      path,
      ext,
      mime_type,
    }
  }
}

pub struct FileServerResolverOptions<'a> {
  pub file_system: &'a Arc<dyn AsyncFileSystem>,
  pub root_dir: &'a Path,
  pub implicit_extensions: bool,
}

pub enum FileServerResolver {
  Explicit(FileServerExplicitResolver),
  Implicit(FileServerImplicitResolver),
}

impl FileServerResolver {
  pub fn new(options: FileServerResolverOptions) -> Self {
    match options.implicit_extensions {
      true => FileServerResolver::Implicit(FileServerImplicitResolver::new(
        Arc::clone(options.file_system),
        options.root_dir.to_path_buf(),
      )),
      false => FileServerResolver::Explicit(FileServerExplicitResolver::new(
        options.root_dir.to_path_buf(),
      )),
    }
  }

  pub async fn resolve(
    &self,
    specifier: &str,
  ) -> io::Result<ResolveResult> {
    match self {
      FileServerResolver::Explicit(resolver) => Ok(resolver.resolve_exact(specifier)),
      FileServerResolver::Implicit(resolver) => resolver.resolve(specifier).await,
    }
  }

  pub fn resolve_exact(
    &self,
    specifier: &str,
  ) -> ResolveResult {
    match self {
      FileServerResolver::Explicit(resolver) => resolver.resolve_exact(specifier),
      FileServerResolver::Implicit(resolver) => resolver.resolve_exact(specifier),
    }
  }
}
