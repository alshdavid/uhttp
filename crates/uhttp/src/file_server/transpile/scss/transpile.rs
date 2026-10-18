use std::io;
use std::path::Path;

use grass::Options;
use grass::OutputStyle;
use normalize_path::NormalizePath;

use crate::file_server::transpile::scss::fs::GrassFs;

/// Files a stylesheet can reference without naming the extension.
const PARTIAL_EXTENSIONS: [&str; 2] = ["scss", "sass"];

pub fn transpile(
  fs: &GrassFs,
  path: &Path,
  kind: &str,
  output_style: OutputStyle,
) -> crate::Result<String> {
  let source_path = match kind {
    "scss" | "sass" => path,
    _ => {
      return Err(io::Error::new(io::ErrorKind::InvalidInput, "Got an invalid file type").into());
    }
  };

  let mut options = Options::default()
    .fs(fs)
    .style(output_style)
    .load_path(path.parent().unwrap_or(Path::new(".")));

  if kind == "sass" {
    options = options.input_syntax(grass::InputSyntax::Sass);
  }

  let code =
    grass::from_path(source_path, &options).map_err(|error| io::Error::other(error.to_string()))?;

  Ok(code)
}

/// Resolve `@import`/`@use` references that name a stylesheet without its
/// extension. The left hand side is the text as written, the right hand side
/// the path to request in its place.
pub fn candidates(
  source: &str,
  source_path: &Path,
  root_dir: &Path,
) -> Vec<(String, String)> {
  let mut candidates = Vec::new();
  let dir = source_path.parent().unwrap_or(Path::new("."));

  for specifier in specifiers(source) {
    let Some(target) = find_in(dir, &specifier) else {
      continue;
    };

    // The URL has to be rooted at the served directory, the file that wrote
    // the reference is not the directory the request is made from.
    if let Some(url_path) = target
      .normalize()
      .strip_prefix(root_dir)
      .ok()
      .map(|path| path.to_string_lossy().replace('\\', "/"))
      && url_path != specifier
    {
      candidates.push((specifier.clone(), format!("/{}", url_path)));
    }
  }

  candidates
}

/// Find the file a specifier names inside a single directory, preferring the
/// partial form sass itself does, and stopping at one match per specifier.
fn find_in(
  dir: &Path,
  specifier: &str,
) -> Option<std::path::PathBuf> {
  let path = dir.join(specifier);
  let partial = match path.file_name().and_then(|value| value.to_str()) {
    Some(name) => path.with_file_name(format!("_{}", name)),
    None => path.clone(),
  };

  for candidate in [&path, &partial] {
    if candidate.extension().is_some() {
      if candidate.is_file() {
        return Some(candidate.clone());
      }

      continue;
    }

    for extension in PARTIAL_EXTENSIONS {
      let candidate = candidate.with_extension(extension);

      if candidate.is_file() {
        return Some(candidate);
      }
    }
  }

  None
}

/// The specifiers of every `@import` and `@use` in a stylesheet.
fn specifiers(source: &str) -> Vec<String> {
  let mut specifiers = Vec::new();

  for line in source.lines() {
    let line = line.trim_start();

    let rest = if let Some(rest) = line.strip_prefix("@use") {
      rest
    } else if let Some(rest) = line.strip_prefix("@import") {
      rest
    } else {
      continue;
    };

    if !rest.starts_with(char::is_whitespace) {
      continue;
    }

    for part in rest.split(',') {
      let part = part.trim();

      for quote in ['"', '\''] {
        let Some(start) = part.find(quote) else {
          continue;
        };

        let Some(end) = part[start + 1..].find(quote) else {
          continue;
        };

        let specifier = part[start + 1..start + 1 + end].to_string();

        if !specifier.is_empty() {
          specifiers.push(specifier);
        }

        break;
      }
    }
  }

  specifiers
}
