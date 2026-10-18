use std::path::PathBuf;

pub mod helpers;
pub mod scss;
pub mod tsconfig;
pub mod typescript;

#[derive(Debug)]
pub struct TransformerResult {
  pub code: String,
  pub map: Option<String>,
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
}

impl TransformerContext {
  pub fn kind(&self) -> &str {
    self.kind.as_str()
  }
}
