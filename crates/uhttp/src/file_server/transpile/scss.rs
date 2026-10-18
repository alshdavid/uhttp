use std::path::Path;

use grass::Options;
use grass::OutputStyle;
use normalize_path::NormalizePath;

use super::TransformerContext;
use super::TransformerResult;

/// Files a stylesheet can reference without naming the extension.
const PARTIAL_EXTENSIONS: [&str; 2] = ["scss", "sass"];

/// `grass` resolves imports against the process working directory and reports
/// paths relative to it in error messages. Compilation runs on a blocking
/// thread, so the chdir would be process-wide; instead imports are resolved
/// through a file system that refuses to leave the served directory.
#[derive(Debug)]
struct ServeFs {
  root: std::path::PathBuf,
}

impl ServeFs {
  fn new(root: &Path) -> Self {
    Self {
      root: root.to_path_buf(),
    }
  }

  fn is_allowed(
    &self,
    path: &Path,
  ) -> bool {
    path.normalize().starts_with(&self.root)
  }
}

impl grass::Fs for ServeFs {
  fn is_dir(
    &self,
    path: &Path,
  ) -> bool {
    self.is_allowed(path) && path.is_dir()
  }

  fn is_file(
    &self,
    path: &Path,
  ) -> bool {
    self.is_allowed(path) && path.is_file()
  }

  fn read(
    &self,
    path: &Path,
  ) -> std::io::Result<Vec<u8>> {
    if !self.is_allowed(path) {
      return Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("{} is outside of the served directory", path.display()),
      ));
    }

    std::fs::read(path)
  }

  fn canonicalize(
    &self,
    path: &Path,
  ) -> std::io::Result<std::path::PathBuf> {
    if !self.is_allowed(path) {
      return Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("{} is outside of the served directory", path.display()),
      ));
    }

    std::fs::canonicalize(path)
  }
}

fn options(ctx: &TransformerContext) -> Options<'_> {
  let fs = Box::leak(Box::new(ServeFs::new(&ctx.root_dir)));

  Options::default()
    .fs(fs)
    .style(OutputStyle::Expanded)
    .load_path(ctx.path.parent().unwrap_or(Path::new(".")))
}

pub fn transpile(ctx: TransformerContext) -> anyhow::Result<TransformerResult> {
  let source_path = match ctx.kind() {
    "scss" | "sass" => ctx.path.clone(),
    _ => anyhow::bail!("Got an invalid file type"),
  };

  let mut options = options(&ctx);

  if ctx.kind() == "sass" {
    options = options.input_syntax(grass::InputSyntax::Sass);
  }

  let code =
    grass::from_path(&source_path, &options).map_err(|error| anyhow::anyhow!("{}", error))?;

  Ok(TransformerResult { code, map: None })
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

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use super::*;

  fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
      "http-server-rs-scss-{}-{}",
      name,
      std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
  }

  fn write(
    path: &Path,
    contents: &str,
  ) {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
  }

  #[test]
  fn resolves_relative_imports() {
    let dir = temp_dir("relative");
    let entry = dir.join("main.scss");
    write(&entry, "@use 'theme';\na { color: theme.$color; }\n");
    write(&dir.join("_theme.scss"), "$color: red;\n");

    let result = transpile(TransformerContext {
      content: b"".to_vec(),
      path: entry,
      source_url: "/main.scss".to_string(),
      source_maps: false,
      kind: "scss".to_string(),
      root_dir: dir,
      tsconfig: Default::default(),
    });

    assert!(
      result.is_ok(),
      "{}",
      result
        .as_ref()
        .err()
        .map(|error| format!("{:?}", error))
        .unwrap_or_default()
    );

    let result = result.unwrap();

    assert!(result.code.contains("color: red"), "{}", result.code);
  }

  #[test]
  fn refuses_to_read_outside_the_served_directory() {
    let dir = temp_dir("escape");
    let serve_dir = dir.join("serve");
    fs::create_dir_all(&serve_dir).unwrap();

    let entry = serve_dir.join("main.scss");
    write(&entry, "@use '../secret';\n");
    write(&dir.join("_secret.scss"), "$color: red;\n");

    assert!(
      transpile(TransformerContext {
        content: b"".to_vec(),
        path: entry,
        source_url: "/main.scss".to_string(),
        source_maps: false,
        kind: "scss".to_string(),
        root_dir: serve_dir,
        tsconfig: Default::default(),
      })
      .is_err()
    );
  }

  #[test]
  fn names_candidates_for_extensionless_imports() {
    let dir = temp_dir("candidates");
    let nested = dir.join("nested");

    let entry = nested.join("main.scss");
    write(&entry, "");
    write(&nested.join("_theme.scss"), "");
    write(&dir.join("_base.scss"), "");

    let found = candidates("@use 'theme';\n@use '../base';\n", &entry, &dir);

    assert_eq!(
      found,
      vec![
        ("theme".to_string(), "/nested/_theme.scss".to_string()),
        ("../base".to_string(), "/_base.scss".to_string()),
      ]
    );
  }
}
