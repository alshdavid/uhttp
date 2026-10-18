use super::SourceMapMode;
use super::TranspileOptions;

/// Builder for [`TranspileOptions`]
///
/// ```rust
/// use uhttp::file_server::SourceMapMode;
/// use uhttp::file_server::TranspileOptions;
///
/// let options = TranspileOptions::builder()
///   .source_maps(SourceMapMode::Linked)
///   .minify(true)
///   .worker_threads(4)
///   .build();
/// ```
#[derive(Debug, Default)]
pub struct TranspileOptionsBuilder {
  source_maps: SourceMapMode,
  minify: bool,
  mangle: bool,
  compress: bool,
  worker_threads: Option<usize>,
}

impl TranspileOptions {
  /// Create a [`TranspileOptionsBuilder`]
  pub fn builder() -> TranspileOptionsBuilder {
    TranspileOptionsBuilder::default()
  }
}

impl TranspileOptionsBuilder {
  /// How source maps are supplied. Defaults to [`SourceMapMode::Inline`].
  pub fn source_maps(
    mut self,
    source_maps: SourceMapMode,
  ) -> Self {
    self.source_maps = source_maps;
    self
  }

  /// Strip whitespace and comments from the output. Defaults to `false`.
  pub fn minify(
    mut self,
    minify: bool,
  ) -> Self {
    self.minify = minify;
    self
  }

  /// Rename identifiers to shorter names. Defaults to `false`.
  pub fn mangle(
    mut self,
    mangle: bool,
  ) -> Self {
    self.mangle = mangle;
    self
  }

  /// Apply size reducing transforms to the syntax tree. Defaults to `false`.
  pub fn compress(
    mut self,
    compress: bool,
  ) -> Self {
    self.compress = compress;
    self
  }

  /// Number of worker threads the transpilers run on. Defaults to
  /// [`std::thread::available_parallelism`].
  pub fn worker_threads(
    mut self,
    worker_threads: usize,
  ) -> Self {
    self.worker_threads = Some(worker_threads);
    self
  }

  /// Construct the [`TranspileOptions`]
  pub fn build(self) -> TranspileOptions {
    TranspileOptions {
      source_maps: self.source_maps,
      minify: self.minify,
      mangle: self.mangle,
      compress: self.compress,
      worker_threads: self.worker_threads.unwrap_or_else(default_worker_threads),
    }
  }
}

fn default_worker_threads() -> usize {
  std::thread::available_parallelism()
    .map(usize::from)
    .unwrap_or(1)
}
