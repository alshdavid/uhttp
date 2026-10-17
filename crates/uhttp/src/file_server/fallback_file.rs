use std::path::Path;
use std::path::PathBuf;

use http::StatusCode;
use mime_guess::Mime;

use super::mime::detect_mime;

pub struct FallbackFileResolveOptions<'a> {
  pub root_dir: &'a Path,
  pub fallback_route: &'a Option<String>,
  pub fallback_status: &'a Option<StatusCode>,
}

pub struct FallbackFile {
  pub path: PathBuf,
  pub status: StatusCode,
  pub mime_type: Mime,
}

impl FallbackFile {
  pub fn resolve(options: FallbackFileResolveOptions<'_>) -> Self {
    let fallback_path = match options.fallback_route.as_ref() {
      Some(path) => options.root_dir.join(path),
      None => options.root_dir.join("404.html"),
    };

    let fallback_status = match options.fallback_status.as_ref() {
      Some(status) => *status,
      None => StatusCode::NOT_FOUND,
    };

    let mime_type = detect_mime(&fallback_path);

    Self {
      path: fallback_path,
      status: fallback_status,
      mime_type,
    }
  }
}
