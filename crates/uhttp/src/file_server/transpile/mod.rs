use std::path::PathBuf;

pub mod helpers;
pub mod javascript;
pub mod scss;
pub mod tsconfig;
pub mod typescript;

#[derive(Debug)]
pub struct TransformerResult {
  pub code: String,
  pub map: Option<String>,
}

/// Optimisations applied to transpiled output.
///
/// These only shrink JavaScript: `mangle` and `compress` have no meaning for
/// a stylesheet, and `minify` is the only one that affects CSS output.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OptimizeOptions {
  /// Strip whitespace and comments from the output
  pub minify: bool,
  /// Rename identifiers to shorter names
  pub mangle: bool,
  /// Apply size reducing transforms to the syntax tree
  pub compress: bool,
}

impl OptimizeOptions {
  /// Whether any optimisation is enabled
  pub fn is_enabled(&self) -> bool {
    self.minify || self.mangle || self.compress
  }
}

pub struct TransformerContext {
  /// The bytes of the input file
  pub content: Vec<u8>,
  /// Path to the source file
  pub path: PathBuf,
  /// URL path the file is served from, used as the source map source
  pub source_url: String,
  /// Whether to generate a source map for the output
  pub source_maps: bool,
  /// The extension of the file being processed
  pub kind: String,
  /// The directory being served, the boundary imports may not cross
  pub root_dir: PathBuf,
  /// The nearest `tsconfig.json` or `deno.json` compiler options, if one was found
  pub tsconfig: tsconfig::TsConfig,
  /// Optimisations to apply to the output
  pub optimize: OptimizeOptions,
}

impl TransformerContext {
  pub fn kind(&self) -> &str {
    self.kind.as_str()
  }
}
