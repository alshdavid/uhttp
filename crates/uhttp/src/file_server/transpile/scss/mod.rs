use std::path::PathBuf;

/// `grass::Fs` adapter over the async file system. Not wired into the
/// transpiler yet, so the module is allowed to go unreferenced.
#[allow(dead_code)]
pub mod fs;
mod transpile;

pub use transpile::candidates;
pub use transpile::transpile;

/// Optimisations applied to compiled stylesheets.
///
/// Only `minify` has any meaning for CSS; the JavaScript-only optimisations
/// (`mangle` and `compress`) are not represented here.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OptimizeOptions {
  /// Strip whitespace and comments from the output
  pub minify: bool,
}

pub struct TransformerContext {
  /// Path to the source file
  pub path: PathBuf,
  /// The extension of the file being processed
  pub kind: String,
  /// The directory being served, the boundary imports may not cross
  pub root_dir: PathBuf,
  /// Optimisations to apply to the output
  pub optimize: OptimizeOptions,
}

impl TransformerContext {
  pub fn kind(&self) -> &str {
    self.kind.as_str()
  }
}
