use std::io;
use std::path::Path;
use std::path::PathBuf;

use dyn_fs::asynch::AsyncFileSystem;

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

// pub struct ResolveResult {
//   pub path: PathBuf,
//   pub ext: String,
//   pub mime_type: String,
// }

// pub struct FileResolver {
//   fs: AsyncFileSystemRef,
//   root_dir: PathBuf,
//   implicit_extensions: bool,
// }

// impl FileResolver {
//   pub fn new() -> Self {
//     todo!()
//   }

//   pub async fn resolve(
//     &self,
//     specifier: &str,
//   ) -> crate::Result<ResolveResult> {
//     let url_path = match self.implicit_extensions {
//       true => {
//         resolve_implicit_extension(ResolveImplicitExtensionOptions {
//           fs: &*self.fs,
//           root_dir: &self.root_dir,
//           input: specifier,
//         })
//         .await?
//       }
//       false => PathBuf::from(specifier),
//     };

//     let mut extension = &url_path.try_extension()?;
//     let mut status = StatusCode::OK;
//     let file_path = self.root_dir.join(&url_path);

//     let mut file = match self.fs.open(&file_path).await {
//       Ok(file) => file,
//       Err(_) => todo!(),
//     };

//     let mime_type = mime_guess::from_ext(extension)
//         .first_or_octet_stream()
//         .to_string();

//     Ok(ResolveResult {
//         path: ,
//         ext: todo!(),
//         mime_type,
//     })
//   }
// }
