use std::path::Path;

use dyn_fs::ext::PathExt;
use mime_guess::Mime;
use mime_guess::mime;

pub fn detect_mime_for_path(path: &Path) -> Mime {
  let base_mime = match path.try_extension() {
    Ok(extension) => mime_guess::from_ext(&extension).first_or_octet_stream(),
    Err(_) => mime_guess::mime::APPLICATION_OCTET_STREAM,
  };

  match is_textual(&base_mime) {
    true => format!("{}; charset=utf-8", base_mime.essence_str())
      .parse()
      .unwrap_or(base_mime),
    false => base_mime,
  }
}

pub fn detect_mime(extension: &str) -> Mime {
  let base_mime = match extension {
    "" => mime_guess::mime::APPLICATION_OCTET_STREAM,
    extension => mime_guess::from_ext(&extension).first_or_octet_stream(),
  };

  match is_textual(&base_mime) {
    true => format!("{}; charset=utf-8", base_mime.essence_str())
      .parse()
      .unwrap_or(base_mime),
    false => base_mime,
  }
}

/// Whether the media type represents text that benefits from an explicit
/// charset. `mime_guess` never emits parameters, so a charset is added for
/// textual types and the common structured-text suffixes.
fn is_textual(mime_type: &Mime) -> bool {
  if mime_type.type_() == mime::TEXT {
    return true;
  }

  let subtype = mime_type.subtype().as_str();

  subtype.ends_with("+json")
    || subtype.ends_with("+xml")
    || matches!(subtype, "json" | "xml" | "javascript" | "ecmascript")
}
