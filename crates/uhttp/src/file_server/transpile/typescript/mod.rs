use std::path::PathBuf;

mod helpers;
mod javascript;
mod transformer;
mod transpile;
mod tsconfig;

pub use transformer::TypeScriptTransformer;

#[derive(Debug)]
pub struct TransformerResult {
  pub code: String,
  /// The source map for the output, when one was requested
  pub map: Option<String>,
}

/// Optimisations applied to transpiled JavaScript output.
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
  pub fn is_enabled(&self) -> bool {
    self.minify || self.mangle || self.compress
  }
}

struct TransformerContext {
  pub content: Vec<u8>,
  pub path: PathBuf,
  /// URL path the file is served from, used as the source map source
  pub source_url: String,
  pub source_maps: bool,
  /// The extension of the file being processed
  pub kind: String,
  /// The nearest `tsconfig.json` or `deno.json` compiler options, if one was found
  pub tsconfig: tsconfig::TsConfig,
  pub optimize: OptimizeOptions,
}
