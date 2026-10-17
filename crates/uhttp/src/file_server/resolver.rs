use std::path::Path;
use std::path::PathBuf;

pub(crate) fn determine_file(input: &str) -> PathBuf {
  if input == "/" {
    PathBuf::from("/index.html".trim_start_matches("/"))
  } else if PathBuf::from(input).extension().is_some() {
    PathBuf::from(input.trim_start_matches("/"))
  } else {
    PathBuf::from(format!("{}.html", input.trim_start_matches("/")))
  }
}

pub(crate) fn try_extension(input: &Path) -> crate::Result<&str> {
  let Some(ext) = input.extension() else {
    return Ok(Default::default());
  };

  let Some(ext) = ext.to_str() else {
    return Ok(Default::default());
  };

  Ok(ext)
}
