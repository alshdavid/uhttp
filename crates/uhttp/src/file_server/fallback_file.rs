use std::path::PathBuf;

use http::StatusCode;

use crate::file_server::resolver::FileServerResolver;

pub struct FallbackFileResolveOptions<'a> {
  pub resolver: &'a FileServerResolver,
  pub fallback_route: &'a Option<String>,
  pub fallback_status: &'a Option<StatusCode>,
}

pub struct FallbackFile {
  pub path: PathBuf,
  pub status: StatusCode,
  pub mime_type: String,
}

impl FallbackFile {
  pub fn resolve(options: FallbackFileResolveOptions<'_>) -> Self {
    let fallback_route = options.fallback_route.as_deref().unwrap_or("404.html");

    // The fallback is always a literal file, so it resolves exactly even when
    // the server is otherwise using implicit extension resolution.
    let resolved = options.resolver.resolve_exact(fallback_route);

    let fallback_status = match options.fallback_status.as_ref() {
      Some(status) => *status,
      None => StatusCode::NOT_FOUND,
    };

    Self {
      path: resolved.path,
      status: fallback_status,
      mime_type: resolved.mime_type,
    }
  }
}
