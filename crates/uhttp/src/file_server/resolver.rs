use std::io;
use std::path::Path;
use std::path::PathBuf;

use dyn_fs::asynch::AsyncFileSystem;

pub async fn resolve_implicit_extension(
  fs: &dyn AsyncFileSystem,
  root_dir: &Path,
  input: &str,
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
