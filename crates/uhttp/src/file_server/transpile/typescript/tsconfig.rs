use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::Deserialize;
use serde_json::Value;

const CONFIG_FILE_NAMES: [&str; 2] = ["deno.json", "tsconfig.json"];

/// The `jsx` compiler option values relevant to transpilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsxMode {
  /// `preserve` - JSX is left as-is and no transform runs.
  Preserve,
  /// `react` (classic) - uses `React.createElement` (or a custom factory).
  React,
  /// `react-jsx` - automatic runtime that injects `react/jsx-runtime`.
  ReactJsx,
  /// `react-jsxdev` - automatic runtime with development-only metadata.
  ReactJsxDev,
  /// `react-native` - JSX is preserved and passed through.
  ReactNative,
}

impl JsxMode {
  fn parse(value: &str) -> Option<Self> {
    match value.to_ascii_lowercase().as_str() {
      "preserve" => Some(Self::Preserve),
      "react" => Some(Self::React),
      "react-jsx" | "reactjsx" => Some(Self::ReactJsx),
      "react-jsxdev" | "reactjsxdev" => Some(Self::ReactJsxDev),
      "react-native" | "reactnative" => Some(Self::ReactNative),
      _ => None,
    }
  }
}

/// A resolution of the TypeScript compiler options that affect transpilation.
///
/// Only a subset of `compilerOptions` is represented - the options that the
/// swc transformer can act on. The options are read from a `tsconfig.json` or
/// a `deno.json`, which share the `compilerOptions` shape.
#[derive(Debug, Clone, PartialEq)]
pub struct TsConfig {
  /// The `jsx` compiler option, if set.
  pub jsx: Option<JsxMode>,
  /// The `jsxFactory` compiler option, e.g. `h` for Preact.
  pub jsx_factory: Option<String>,
  /// The `jsxFragmentFactory` compiler option, e.g. `Fragment`.
  pub jsx_fragment_factory: Option<String>,
  /// The `jsxImportSource` compiler option, e.g. `preact` for the automatic runtime.
  pub jsx_import_source: Option<String>,
  /// A value that changes when the config on disk changes.
  pub signature: u64,
}

impl TsConfig {
  /// The default options used when no config file could be found.
  pub fn defaults() -> Self {
    Self {
      jsx: None,
      jsx_factory: None,
      jsx_fragment_factory: None,
      jsx_import_source: None,
      signature: 0,
    }
  }
}

impl Default for TsConfig {
  fn default() -> Self {
    Self::defaults()
  }
}

/// The raw shape of a config file as it appears on disk.
///
/// `tsconfig.json` and `deno.json` agree on the parts read here; the extra
/// top-level keys a `deno.json` carries (`imports`, `tasks`, ...) are ignored.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTsConfig {
  /// Path to a config to inherit from. Can be a single string or an array.
  extends: Option<Value>,
  compiler_options: Option<CompilerOptions>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompilerOptions {
  jsx: Option<String>,
  jsx_factory: Option<String>,
  jsx_fragment_factory: Option<String>,
  jsx_import_source: Option<String>,
}

impl CompilerOptions {
  /// Flatten into a map so that inherited options can be merged by key.
  fn into_map(self) -> HashMap<String, Value> {
    let mut map = HashMap::new();

    let mut insert = |key: &str, value: Option<String>| {
      if let Some(value) = value {
        map.insert(key.to_string(), Value::String(value));
      }
    };

    insert("jsx", self.jsx);
    insert("jsxFactory", self.jsx_factory);
    insert("jsxFragmentFactory", self.jsx_fragment_factory);
    insert("jsxImportSource", self.jsx_import_source);

    map
  }
}

/// Locate a config from this dir up to (inclusive) the `stop_dir` boundary.
///
/// Returns the path to the nearest `deno.json` or `tsconfig.json`. A
/// `deno.json` is preferred when both sit in the same directory.
pub fn find_nearest_tsconfig(
  file_path: &Path,
  stop_dir: &Path,
) -> Option<PathBuf> {
  let mut current = file_path.parent()?;

  loop {
    for name in CONFIG_FILE_NAMES {
      let candidate = current.join(name);

      if candidate.is_file() {
        return Some(candidate);
      }
    }

    // Stop after checking the serve dir so that configs outside of the
    // served directory don't leak into the responses.
    if current == stop_dir {
      return None;
    }

    current = {
      let parent = current.parent()?;
      // Never walk above the stop dir
      if !parent.starts_with(stop_dir) && parent != stop_dir {
        return None;
      }
      parent
    };
  }
}

/// Read and resolve a config, following the `extends` chain.
///
/// Missing files and invalid JSON are treated as "no config" (`None`) because
/// a malformed config should not stop the server from serving files.
pub fn load_tsconfig(path: &Path) -> Option<TsConfig> {
  let (options, signature) = load_chain(path, 0, &mut Vec::new())?;

  let get = |key: &str| options.get(key).and_then(|value| value.as_str());

  Some(TsConfig {
    jsx: get("jsx").and_then(JsxMode::parse),
    jsx_factory: get("jsxFactory").map(ToString::to_string),
    jsx_fragment_factory: get("jsxFragmentFactory").map(ToString::to_string),
    jsx_import_source: get("jsxImportSource").map(ToString::to_string),
    signature,
  })
}

/// Recursively load a config and its parents, returning the merged compiler
/// options and a signature derived from the contents of every file read.
fn load_chain(
  path: &Path,
  depth: usize,
  visited: &mut Vec<PathBuf>,
) -> Option<(HashMap<String, Value>, u64)> {
  // Protect against circular `extends` references
  if depth > 16 {
    return None;
  }
  let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

  if visited.contains(&canonical) {
    return None;
  }
  visited.push(canonical.clone());

  let contents = fs::read_to_string(path).ok()?;
  let raw: RawTsConfig = serde_json::from_str(&contents).ok()?;

  let mut options = HashMap::new();

  // The `extends` chain is resolved first so that child options win
  if let Some(extends) = raw.extends.as_ref() {
    for specifier in extends_specifiers(extends) {
      if let Some(extended_path) = resolve_extends(path, &specifier)
        && let Some((parent_options, _)) = load_chain(&extended_path, depth + 1, visited)
      {
        options.extend(parent_options);
      }
    }
  }

  if let Some(compiler_options) = raw.compiler_options {
    for (key, value) in compiler_options.into_map() {
      options.insert(key, value);
    }
  }

  let signature = signature_of(&contents);

  Some((options, signature))
}

/// Normalise `extends` into a list of specifiers.
fn extends_specifiers(value: &Value) -> Vec<String> {
  match value {
    Value::String(value) => vec![value.clone()],
    Value::Array(values) => values
      .iter()
      .filter_map(|value| value.as_str().map(ToString::to_string))
      .collect(),
    _ => Vec::new(),
  }
}

/// Resolve an `extends` specifier to a path on disk.
///
/// A specifier without an extension is treated as a directory holding another
/// config of the same kind (`tsconfig.json` next to a `tsconfig.json`,
/// `deno.json` next to a `deno.json`). Also supports configs that live inside
/// a node package (e.g. `@tsconfig/node20/tsconfig.json`) by walking up the
/// directory tree looking for a `node_modules` folder.
fn resolve_extends(
  config_path: &Path,
  specifier: &str,
) -> Option<PathBuf> {
  let base_dir = config_path.parent()?;
  let config_file = config_path
    .file_name()
    .unwrap_or_else(|| "tsconfig.json".as_ref());

  let with_json = |value: PathBuf| -> PathBuf {
    if value.extension().is_some() {
      value
    } else {
      value.join(config_file)
    }
  };

  if specifier.starts_with('.') || specifier.starts_with('/') {
    let candidate = with_json(base_dir.join(specifier));
    if candidate.is_file() {
      return Some(candidate);
    }
    return None;
  }

  let mut current = Some(base_dir);

  while let Some(dir) = current {
    let candidate = with_json(dir.join("node_modules").join(specifier));

    if candidate.is_file() {
      return Some(candidate);
    }

    current = dir.parent();
  }

  None
}

/// A cheap content hash used to key the tsconfig cache.
fn signature_of(contents: &str) -> u64 {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::Hash;
  use std::hash::Hasher;

  let mut hasher = DefaultHasher::new();
  contents.hash(&mut hasher);
  hasher.finish()
}

/// A process-wide cache of resolved configs keyed by source file path.
///
/// Config resolution reads from disk, so caching avoids doing filesystem
/// work on every single request.
#[derive(Debug, Default)]
pub struct TsConfigCache {
  entries: Mutex<HashMap<PathBuf, CachedConfig>>,
}

#[derive(Debug, Clone)]
struct CachedConfig {
  config_path: Option<PathBuf>,
  signature: u64,
  config: TsConfig,
}

impl TsConfigCache {
  pub fn new() -> Self {
    Self::default()
  }

  /// Resolve the config for a file, using the cache when the underlying
  /// config files have not changed.
  pub fn resolve(
    &self,
    file_path: &Path,
    stop_dir: &Path,
  ) -> TsConfig {
    let key = file_path.to_path_buf();

    {
      let entries = self.entries.lock().ok();
      if let Some(entries) = entries.as_ref()
        && let Some(cached) = entries.get(&key)
        && is_fresh(cached, file_path, stop_dir)
      {
        return cached.config.clone();
      }
    }

    let config_path = find_nearest_tsconfig(file_path, stop_dir);
    let config = match config_path.as_ref() {
      Some(path) => load_tsconfig(path).unwrap_or_else(TsConfig::defaults),
      None => TsConfig::defaults(),
    };

    if let Ok(mut entries) = self.entries.lock() {
      entries.insert(
        key,
        CachedConfig {
          config_path,
          signature: config.signature,
          config: config.clone(),
        },
      );
    }

    config
  }
}

/// Returns `true` when the cached entry still reflects what is on disk.
fn is_fresh(
  cached: &CachedConfig,
  file_path: &Path,
  stop_dir: &Path,
) -> bool {
  // Re-run the lookup so that a config appearing, disappearing or taking
  // priority over the cached one invalidates the entry.
  if find_nearest_tsconfig(file_path, stop_dir) != cached.config_path {
    return false;
  }

  let Some(path) = cached.config_path.as_ref() else {
    return true;
  };

  let Ok(contents) = fs::read_to_string(path) else {
    return false;
  };

  signature_of(&contents) == cached.signature
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use super::*;

  fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
      "http-server-rs-tsconfig-{}-{}",
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
    let mut file = fs::File::create(path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
  }

  #[test]
  fn finds_nearest_tsconfig_walking_up() {
    let dir = temp_dir("nearest");
    let nested = dir.join("a/b/c");
    fs::create_dir_all(&nested).unwrap();

    write(&dir.join("tsconfig.json"), r#"{ "compilerOptions": {} }"#);
    write(
      &dir.join("a/tsconfig.json"),
      r#"{ "compilerOptions": { "jsx": "react" } }"#,
    );

    let file = nested.join("main.tsx");
    write(&file, "export const a = 1;");

    let found = find_nearest_tsconfig(&file, &dir).unwrap();
    assert_eq!(found, dir.join("a/tsconfig.json"));

    let config = load_tsconfig(&found).unwrap();
    assert_eq!(config.jsx, Some(JsxMode::React));
  }

  #[test]
  fn finds_nearest_deno_json_walking_up() {
    let dir = temp_dir("deno-nearest");
    let nested = dir.join("a/b/c");
    fs::create_dir_all(&nested).unwrap();

    write(
      &dir.join("a/deno.json"),
      r#"{
        "compilerOptions": {
          "strict": true,
          "lib": ["dom", "dom.iterable", "esnext"],
          "jsx": "react",
          "jsxFactory": "h",
          "jsxFragmentFactory": "Fragment"
        },
        "imports": {
          "preact": "https://esm.sh/preact@11"
        }
      }"#,
    );

    let file = nested.join("main.tsx");
    write(&file, "export const a = 1;");

    let found = find_nearest_tsconfig(&file, &dir).unwrap();
    assert_eq!(found, dir.join("a/deno.json"));

    let config = load_tsconfig(&found).unwrap();
    assert_eq!(config.jsx, Some(JsxMode::React));
    assert_eq!(config.jsx_factory.as_deref(), Some("h"));
    assert_eq!(config.jsx_fragment_factory.as_deref(), Some("Fragment"));
  }

  #[test]
  fn prefers_deno_json_over_tsconfig_in_same_dir() {
    let dir = temp_dir("deno-priority");
    write(
      &dir.join("tsconfig.json"),
      r#"{ "compilerOptions": { "jsx": "react-jsx" } }"#,
    );
    write(
      &dir.join("deno.json"),
      r#"{ "compilerOptions": { "jsx": "react", "jsxFactory": "h" } }"#,
    );

    let file = dir.join("main.tsx");
    write(&file, "export const a = 1;");

    let found = find_nearest_tsconfig(&file, &dir).unwrap();
    assert_eq!(found, dir.join("deno.json"));
    assert_eq!(load_tsconfig(&found).unwrap().jsx, Some(JsxMode::React));
  }

  #[test]
  fn nearer_dir_wins_over_deno_json_priority() {
    let dir = temp_dir("deno-distance");
    let nested = dir.join("app");
    fs::create_dir_all(&nested).unwrap();

    write(
      &dir.join("deno.json"),
      r#"{ "compilerOptions": { "jsx": "react" } }"#,
    );
    write(
      &nested.join("tsconfig.json"),
      r#"{ "compilerOptions": { "jsx": "react-jsx" } }"#,
    );

    let file = nested.join("main.tsx");
    write(&file, "export const a = 1;");

    let found = find_nearest_tsconfig(&file, &dir).unwrap();
    assert_eq!(found, nested.join("tsconfig.json"));
    assert_eq!(load_tsconfig(&found).unwrap().jsx, Some(JsxMode::ReactJsx));
  }

  #[test]
  fn resolves_deno_json_extends_chain() {
    let dir = temp_dir("deno-extends");
    write(
      &dir.join("deno.base.json"),
      r#"{ "compilerOptions": { "jsx": "react" } }"#,
    );
    write(
      &dir.join("deno.json"),
      r#"{ "extends": "./deno.base.json", "compilerOptions": { "jsxFactory": "h" } }"#,
    );

    let config = load_tsconfig(&dir.join("deno.json")).unwrap();
    assert_eq!(config.jsx, Some(JsxMode::React));
    assert_eq!(config.jsx_factory.as_deref(), Some("h"));
  }

  #[test]
  fn cache_switches_to_deno_json_when_it_appears() {
    let dir = temp_dir("deno-cache");
    write(
      &dir.join("tsconfig.json"),
      r#"{ "compilerOptions": { "jsx": "react-jsx" } }"#,
    );

    let file = dir.join("main.tsx");
    write(&file, "export const a = 1;");

    let cache = TsConfigCache::new();
    assert_eq!(cache.resolve(&file, &dir).jsx, Some(JsxMode::ReactJsx));

    write(
      &dir.join("deno.json"),
      r#"{ "compilerOptions": { "jsx": "react" } }"#,
    );
    assert_eq!(cache.resolve(&file, &dir).jsx, Some(JsxMode::React));

    write(
      &dir.join("deno.json"),
      r#"{ "compilerOptions": { "jsx": "react-jsx" } }"#,
    );
    assert_eq!(cache.resolve(&file, &dir).jsx, Some(JsxMode::ReactJsx));
  }

  #[test]
  fn stops_at_serve_dir_boundary() {
    let dir = temp_dir("boundary");
    let serve = dir.join("serve");
    fs::create_dir_all(&serve).unwrap();

    // tsconfig lives *above* the served directory and must be ignored
    write(
      &dir.join("tsconfig.json"),
      r#"{ "compilerOptions": { "jsx": "react" } }"#,
    );

    let file = serve.join("main.tsx");
    write(&file, "export const a = 1;");

    assert_eq!(find_nearest_tsconfig(&file, &serve), None);

    let cache = TsConfigCache::new();
    assert_eq!(cache.resolve(&file, &serve).jsx, None);
  }

  #[test]
  fn resolves_extends_chain() {
    let dir = temp_dir("extends");
    write(
      &dir.join("tsconfig.base.json"),
      r#"{ "compilerOptions": { "jsx": "react" } }"#,
    );
    write(
      &dir.join("tsconfig.json"),
      r#"{ "extends": "./tsconfig.base.json", "compilerOptions": {} }"#,
    );

    let config = load_tsconfig(&dir.join("tsconfig.json")).unwrap();
    assert_eq!(config.jsx, Some(JsxMode::React));
  }

  #[test]
  fn child_options_override_extends() {
    let dir = temp_dir("override");
    write(
      &dir.join("tsconfig.base.json"),
      r#"{ "compilerOptions": { "jsx": "react" } }"#,
    );
    write(
      &dir.join("tsconfig.json"),
      r#"{ "extends": "./tsconfig.base.json", "compilerOptions": { "jsx": "react-jsx" } }"#,
    );

    let config = load_tsconfig(&dir.join("tsconfig.json")).unwrap();
    assert_eq!(config.jsx, Some(JsxMode::ReactJsx));
  }

  #[test]
  fn malformed_json_is_ignored() {
    let dir = temp_dir("malformed");
    write(&dir.join("tsconfig.json"), "{ not valid json");

    assert!(load_tsconfig(&dir.join("tsconfig.json")).is_none());
  }

  #[test]
  fn parses_factory_options_and_ignores_unrelated_options() {
    let dir = temp_dir("factory");

    // Mirrors the shape of testing/transpile/preact-tsconfig/tsconfig.json,
    // which contains many options the server does not care about.
    write(
      &dir.join("tsconfig.json"),
      r#"{
        "compilerOptions": {
          "strict": true,
          "module": "nodenext",
          "moduleResolution": "nodenext",
          "lib": ["DOM", "ESNext"],
          "types": [],
          "jsx": "react",
          "jsxFactory": "h",
          "allowImportingTsExtensions": true,
          "noEmit": true
        }
      }"#,
    );

    let config = load_tsconfig(&dir.join("tsconfig.json")).unwrap();
    assert_eq!(config.jsx, Some(JsxMode::React));
    assert_eq!(config.jsx_factory.as_deref(), Some("h"));
    assert_eq!(config.jsx_fragment_factory, None);
    assert_eq!(config.jsx_import_source, None);
  }

  #[test]
  fn cache_invalidates_on_change() {
    let dir = temp_dir("cache");
    let config_path = dir.join("tsconfig.json");
    write(&config_path, r#"{ "compilerOptions": { "jsx": "react" } }"#);

    let file = dir.join("main.tsx");
    write(&file, "export const a = 1;");

    let cache = TsConfigCache::new();
    assert_eq!(cache.resolve(&file, &dir).jsx, Some(JsxMode::React));

    write(
      &config_path,
      r#"{ "compilerOptions": { "jsx": "react-jsx" } }"#,
    );
    assert_eq!(cache.resolve(&file, &dir).jsx, Some(JsxMode::ReactJsx));
  }
}
