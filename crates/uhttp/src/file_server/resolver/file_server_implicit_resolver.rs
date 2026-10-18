use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use dyn_fs::asynch::AsyncFileSystem;

use super::file_server_resolver::ResolveResult;

/// ```text
/// /      -> /index.html
/// /foo   -> /foo/index.html | /foo.html
/// ```
pub struct FileServerImplicitResolver {
  fs: Arc<dyn AsyncFileSystem>,
  root_dir: PathBuf,
}

impl FileServerImplicitResolver {
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
    let path = resolve_implicit_extension(ResolveImplicitExtensionOptions {
      fs: &*self.fs,
      root_dir: &self.root_dir,
      input: specifier,
    })
    .await?;

    Ok(ResolveResult::from_path(path))
  }

  pub fn resolve_exact(
    &self,
    specifier: &str,
  ) -> ResolveResult {
    let path = self.root_dir.join(specifier.trim_start_matches('/'));
    ResolveResult::from_path(path)
  }
}

pub struct ResolveImplicitExtensionOptions<'a> {
  pub fs: &'a dyn AsyncFileSystem,
  pub root_dir: &'a Path,
  pub input: &'a str,
}

pub async fn resolve_implicit_extension<'a>(
  ResolveImplicitExtensionOptions {
    fs,
    root_dir,
    input,
  }: ResolveImplicitExtensionOptions<'a>
) -> io::Result<PathBuf> {
  let base_path = root_dir.join(input.trim_start_matches('/'));

  let candidates = vec![
    base_path.clone(),                // e.g., /foo
    base_path.with_extension("html"), // e.g., /foo.html
    base_path.join("index.html"),     // e.g., /foo/index.html
  ];

  for candidate in candidates {
    match fs.metadata(&candidate).await {
      Ok(metadata) => {
        if metadata.is_file() {
          return Ok(candidate);
        }
      }
      Err(e) if e.kind() == io::ErrorKind::NotFound => {
        continue;
      }
      Err(e) => {
        return Err(e);
      }
    }
  }

  Err(io::Error::new(
    io::ErrorKind::NotFound,
    "Unable to resolve file",
  ))
}
