use std::collections::HashMap;
#[cfg(feature = "file_server_transpile")]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "file_server_transpile")]
use base64::Engine as _;
use tokio::io::AsyncWriteExt;

#[cfg(feature = "file_server_transpile")]
use super::cache::CacheKey;
use super::cache::CachedResponse;
use super::cache::FileServerCache;
use super::cache::TeeWriter;
use super::compression::brotli_stream;
use super::compression::gzip_stream;
use super::compression::zstd_stream;
use super::encoding::Encoding;
use super::encoding::EncodingResolver;
use super::etag::EtagResolver;
use super::fallback_file::FallbackFile;
use super::fallback_file::FallbackFileResolveOptions;
use super::file_system::AsyncFileSystem;
use super::resolver::FileServerResolver;
use super::resolver::FileServerResolverOptions;
#[cfg(feature = "file_server_transpile")]
use super::transpile::typescript;
use crate::HandleFunc;
use crate::Request;
use crate::Response;
use crate::ResponseBody;
use crate::StatusCode;
use crate::file_server::compression::uncompressed_stream;
use crate::headers;

/// Size of the buffer bodies are streamed through on their way to the client
pub const BODY_BUFFER_SIZE: usize = 64 * 1024;

/// Content type responses carrying transpiled TypeScript are served with
#[cfg(feature = "file_server_transpile")]
const JAVASCRIPT_CONTENT_TYPE: &str = "application/javascript; charset=utf-8";

/// Content type linked source maps are served with
#[cfg(feature = "file_server_transpile")]
const SOURCE_MAP_CONTENT_TYPE: &str = "application/json; charset=utf-8";

/// Content type transpiled stylesheets are served with
#[cfg(feature = "file_server_transpile")]
const STYLESHEET_CONTENT_TYPE: &str = "text/css; charset=utf-8";

#[derive(Debug, Default)]
pub enum ETagStrategy {
  /// Non cryptographic hash of the file, slower
  Hash,
  /// Faster
  #[default]
  LastModified,
  /// No etag calc
  Disabled,
}

#[cfg(feature = "file_server_transpile")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SourceMapMode {
  #[default]
  Inline,
  Linked,
  Disabled,
}

#[cfg(feature = "file_server_transpile")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TranspileOptions {
  /// How source maps are supplied
  pub source_maps: SourceMapMode,
  /// Strip whitespace and comments from the output
  pub minify: bool,
  /// Rename identifiers to shorter names
  pub mangle: bool,
  /// Apply size reducing transforms to the syntax tree
  pub compress: bool,
  /// Defaults to [`std::thread::available_parallelism`] when `None`.
  pub worker_threads: Option<usize>,
}

#[cfg(feature = "file_server_transpile")]
impl TranspileOptions {
  fn typescript_optimize(&self) -> typescript::OptimizeOptions {
    typescript::OptimizeOptions {
      minify: self.minify,
      mangle: self.mangle,
      compress: self.compress,
    }
  }
}

/// The transpiler a request is handled by.
#[cfg(feature = "file_server_transpile")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TranspileTarget {
  /// TypeScript, transpiled to JavaScript
  TypeScript,
  /// SCSS, transpiled to CSS
  StyleSheet,
}

/// Extensions that are JIT transpiled, mapped to the transpiler that handles
/// them and the syntax they are parsed as.
#[cfg(feature = "file_server_transpile")]
fn transpile_target(path: &Path) -> Option<(TranspileTarget, &'static str)> {
  match path.extension().and_then(|extension| extension.to_str()) {
    Some("ts" | "mts" | "cts") => Some((TranspileTarget::TypeScript, "ts")),
    Some("tsx") => Some((TranspileTarget::TypeScript, "tsx")),
    Some("scss") => Some((TranspileTarget::StyleSheet, "scss")),
    Some("sass") => Some((TranspileTarget::StyleSheet, "sass")),
    _ => None,
  }
}

/// The transpiled response a request maps to.
#[cfg(feature = "file_server_transpile")]
struct TranspileRequest {
  /// The transpiler that renders the response
  target: TranspileTarget,
  /// The syntax the source is parsed as
  kind: &'static str,
  /// Value for the `Content-Type` header
  mime_type: &'static str,
  /// The source file to transpile
  source_file_path: PathBuf,
  /// URL the source is served from, recorded in the source map
  source_url: String,
  /// Whether only the source map is requested
  source_map: bool,
}

/// Stream an in-memory body to the client, applying the negotiated content
/// encoding on the way.
///
/// When `cache` is set the encoded body is buffered, up to the cache entry
/// limit, and stored so later requests can replay it.
#[cfg(feature = "file_server_transpile")]
fn encode_body(
  encoding: Encoding,
  content: String,
  mime_type: &'static str,
  cache: Option<(Arc<FileServerCache>, CacheKey, u64)>,
) -> ResponseBody {
  let (body, writer) = ResponseBody::chunked(BODY_BUFFER_SIZE);
  let mut tee = TeeWriter::new(
    writer,
    cache.as_ref().map(|(cache, _, _)| cache.max_entry_bytes()),
  );

  let content_encoding = encoding.header_value();

  tokio::task::spawn(async move {
    let input = content.as_bytes();

    let _ = match encoding {
      Encoding::Zstd => zstd_stream(input, &mut tee).await,
      Encoding::Brotli => brotli_stream(input, &mut tee).await,
      Encoding::Gzip => gzip_stream(input, &mut tee).await,
      Encoding::Identity => uncompressed_stream(input, &mut tee).await,
    };
    let _ = tee.shutdown().await;

    if let (Some((cache, key, generation)), Some(buffered)) = (cache, tee.into_buffered()) {
      cache.set(
        key,
        CachedResponse {
          status: StatusCode::OK,
          mime_type: mime_type.to_string(),
          content_encoding,
          etag: None,
          body: Arc::from(buffered),
        },
        generation,
      );
    }
  });

  body
}

/// Build a response that replays a cached body verbatim, honouring conditional
/// requests.
fn replay_cached(
  req: &Request,
  mut res: Response,
  options: &FileServerOptions,
  cached: CachedResponse,
) -> crate::HandlerResult {
  for (key, value) in &options.custom_headers {
    res = res.header(key.as_str(), value.as_str());
  }

  res = res.header(headers::CONTENT_TYPE, cached.mime_type.as_str());

  if let Some(content_encoding) = cached.content_encoding {
    res = res.header(headers::CONTENT_ENCODING, content_encoding);
  }

  if let Some(etag) = cached.etag.as_ref() {
    if !has_modified(req, etag) {
      return res.status(StatusCode::NOT_MODIFIED).empty();
    }
    res = res.header(headers::ETAG, etag.as_str());
  }

  let (body, mut writer) = ResponseBody::chunked(BODY_BUFFER_SIZE);

  tokio::task::spawn(async move {
    let _ = writer.write_all(cached.body.as_ref()).await;
    let _ = writer.shutdown().await;
  });

  res.status(cached.status).body(body)
}

pub struct FileServerOptions {
  /// The filesystem files are read from
  pub file_system: Arc<dyn AsyncFileSystem>,
  /// The root directory to get files from
  pub root_dir: PathBuf,
  /// Send back compressed responses
  pub compress: bool,
  /// How to supply etag
  pub etag: ETagStrategy,
  /// Add Custom Headers
  pub custom_headers: HashMap<String, String>,
  /// Optional in-memory cache of rendered responses.
  pub cache: Option<Arc<FileServerCache>>,
  /// Relative path to fallback URL. Defaults to "404.html"
  pub fallback_route: Option<String>,
  /// Defaults to [`crate::StatusCode::NOT_FOUND] (404)
  pub fallback_status: Option<StatusCode>,
  /// Try to resolve html files on paths without specified files
  ///  `/`    -> `/index.html`
  ///  `/foo` -> `/foo/index.html` | `/foo.html`
  pub implicit_extensions: bool,
  #[cfg(feature = "file_server_transpile")]
  /// JIT Transpilation of TypeScript and SCSS sources
  pub transpile: Option<TranspileOptions>,
}

/// Serve files from the filesystem
pub fn create(options: FileServerOptions) -> crate::Result<HandleFunc> {
  let options = Arc::new(options);

  let resolver = Arc::new(FileServerResolver::new(FileServerResolverOptions {
    file_system: &options.file_system,
    root_dir: &options.root_dir,
    implicit_extensions: options.implicit_extensions,
  }));

  let etag_resolver = Arc::new(EtagResolver::new(&options.etag));

  let encoding_resolver = Arc::new(EncodingResolver::new(options.compress));

  let fallback = Arc::new(FallbackFile::resolve(FallbackFileResolveOptions {
    resolver: &resolver,
    fallback_route: &options.fallback_route,
    fallback_status: &options.fallback_status,
  }));

  #[cfg(feature = "file_server_transpile")]
  let worker_threads: usize = options
    .transpile
    .and_then(|options| options.worker_threads)
    .unwrap_or(std::thread::available_parallelism()?.into());

  #[cfg(feature = "file_server_transpile")]
  let scss_transformer = Arc::new(super::transpile::scss::ScssTransformer::new(
    &options.file_system,
    &options.root_dir,
    match options.transpile {
      Some(options) => match options.minify {
        true => grass::OutputStyle::Compressed,
        false => grass::OutputStyle::Expanded,
      },
      None => grass::OutputStyle::Expanded,
    },
    worker_threads,
  ));

  #[cfg(feature = "file_server_transpile")]
  let typescript_transformer = Arc::new(typescript::TypeScriptTransformer::new(
    &options.file_system,
    &options.root_dir,
    match options.transpile {
      Some(options) => options.typescript_optimize(),
      None => Default::default(),
    },
    worker_threads,
  ));

  Ok(Box::new(move |req, mut res| {
    let options = Arc::clone(&options);
    let resolver = Arc::clone(&resolver);
    let etag_resolver = Arc::clone(&etag_resolver);
    let encoding_resolver = Arc::clone(&encoding_resolver);
    let fallback = Arc::clone(&fallback);
    #[cfg(feature = "file_server_transpile")]
    let scss_transformer = Arc::clone(&scss_transformer);
    #[cfg(feature = "file_server_transpile")]
    let typescript_transformer = Arc::clone(&typescript_transformer);

    Box::pin(async move {
      // The encoding is needed by the transpile path as well as the static
      // file path, so it is negotiated up front.
      let encoding = match encoding_resolver.as_ref() {
        Some(resolver) => resolver.resolve(&req),
        None => Encoding::Identity,
      };

      // swc shared helpers the transpiler emits imports for. They are
      // synthesised rather than read from disk, so they are served before the
      // request is resolved.
      #[cfg(feature = "file_server_transpile")]
      if options.transpile.is_some()
        && let Some(name) = typescript::TypeScriptTransformer::helper_module_name(req.uri.path())
        && let Some(source) = typescript::TypeScriptTransformer::helper_module_source(name)
      {
        // The helpers are ordinary JavaScript, so they are optimised directly
        // rather than transpiled. A helper that fails to optimise is served
        // as-is.
        let source = typescript_transformer
          .optimize_module(&source)
          .await
          .unwrap_or(source);

        let cache_key = (PathBuf::from(req.uri.path()), encoding);

        if let Some(cache) = options.cache.as_ref()
          && let Some(cached) = cache.get(&cache_key)
        {
          return replay_cached(&req, res, &options, cached);
        }

        let cache_slot = options
          .cache
          .as_ref()
          .map(|cache| (Arc::clone(cache), cache_key.clone(), cache.generation()));

        res = res.header(headers::CONTENT_TYPE, JAVASCRIPT_CONTENT_TYPE);

        if let Some(content_encoding) = encoding.header_value() {
          res = res.header(headers::CONTENT_ENCODING, content_encoding);
        }

        return res.status(StatusCode::OK).body(encode_body(
          encoding,
          source,
          JAVASCRIPT_CONTENT_TYPE,
          cache_slot,
        ));
      }

      let resolved = match resolver.resolve(req.uri.path()).await {
        Ok(resolved) => Some(resolved),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
      };

      // JIT transpile TypeScript and SCSS sources. The request names the source
      // file directly; the transpiled module is served from memory and requests
      // for a sibling `.js` file are never resolved to it here.
      #[cfg(feature = "file_server_transpile")]
      if let Some(transpile_options) = options.transpile {
        let request_path = req.uri.path();

        // Map the request to a transpiled response from its URL alone, without
        // running the transpiler or touching the filesystem, so a cached entry
        // can be replayed even once the source is gone from disk.
        //
        // A linked source map is requested as the `.map` sibling of a source
        // file (e.g. `/main.tsx.map`).
        let transpile_request =
          if let Some((target, kind)) = transpile_target(Path::new(request_path)) {
            Some(TranspileRequest {
              target,
              kind,
              mime_type: match target {
                TranspileTarget::TypeScript => JAVASCRIPT_CONTENT_TYPE,
                TranspileTarget::StyleSheet => STYLESHEET_CONTENT_TYPE,
              },
              source_file_path: options.root_dir.join(request_path.trim_start_matches('/')),
              source_url: request_path.to_string(),
              source_map: false,
            })
          } else if transpile_options.source_maps == SourceMapMode::Linked
            && let Some(source_path) = request_path.strip_suffix(".map")
            && let Some((TranspileTarget::TypeScript, kind)) =
              transpile_target(Path::new(source_path))
          {
            Some(TranspileRequest {
              target: TranspileTarget::TypeScript,
              kind,
              mime_type: SOURCE_MAP_CONTENT_TYPE,
              source_file_path: options.root_dir.join(source_path.trim_start_matches('/')),
              source_url: source_path.to_string(),
              source_map: true,
            })
          } else {
            None
          };

        if let Some(transpile_request) = transpile_request {
          // Transpiled responses are keyed by the request path so a source and
          // its `.map` sibling do not share an entry.
          let cache_key = (PathBuf::from(request_path), encoding);

          // A cached response is replayed regardless of whether the source
          // still exists on disk: a deleted or edited source keeps serving the
          // last rendered response until the entry is invalidated.
          if let Some(cache) = options.cache.as_ref()
            && let Some(cached) = cache.get(&cache_key)
          {
            return replay_cached(&req, res, &options, cached);
          }

          // A real source map on disk takes precedence over one generated from
          // the source.
          let shadowed_by_disk = transpile_request.source_map && resolved.is_some();

          // Captured before rendering so a response rendered against an older
          // generation cannot repopulate a cleared cache.
          let cache_slot = options
            .cache
            .as_ref()
            .map(|cache| (Arc::clone(cache), cache_key.clone(), cache.generation()));

          let content = match transpile_request.target {
            TranspileTarget::TypeScript if shadowed_by_disk => None,
            TranspileTarget::TypeScript => {
              let source_maps = !matches!(transpile_options.source_maps, SourceMapMode::Disabled);

              typescript_transformer
                .transpile(
                  &transpile_request.source_file_path,
                  &transpile_request.source_url,
                  transpile_request.kind,
                  source_maps,
                )
                .await?
                .and_then(|result| {
                  if transpile_request.source_map {
                    return result.map;
                  }

                  Some(match (transpile_options.source_maps, result.map) {
                    // Inlined so the response stays a single, self-contained request.
                    (SourceMapMode::Inline, Some(map)) => format!(
                      "{}\n//# sourceMappingURL=data:application/json;base64,{}\n",
                      result.code,
                      base64::engine::general_purpose::STANDARD.encode(map.as_bytes()),
                    ),
                    // Linked so the browser fetches the `.map` sibling separately.
                    (SourceMapMode::Linked, Some(_)) => {
                      let map_name = transpile_request
                        .source_file_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default();

                      format!("{}\n//# sourceMappingURL={}.map\n", result.code, map_name)
                    }
                    _ => result.code,
                  })
                })
            }
            TranspileTarget::StyleSheet => Some(
              scss_transformer
                .transpile(&transpile_request.source_file_path, transpile_request.kind)
                .await?,
            ),
          };

          if let Some(content) = content {
            for (key, value) in &options.custom_headers {
              res = res.header(key.as_str(), value.as_str());
            }

            res = res.header(headers::CONTENT_TYPE, transpile_request.mime_type);

            if let Some(content_encoding) = encoding.header_value() {
              res = res.header(headers::CONTENT_ENCODING, content_encoding);
            }

            return res.status(StatusCode::OK).body(encode_body(
              encoding,
              content,
              transpile_request.mime_type,
              cache_slot,
            ));
          }
        }
      }

      let cache_key = resolved
        .as_ref()
        .map(|resolved| (resolved.path.clone(), encoding));

      // Fast path: replay a fully rendered response from the cache.
      if let Some(cache) = options.cache.as_ref()
        && let Some(cache_key) = cache_key.as_ref()
        && let Some(cached) = cache.get(cache_key)
      {
        return replay_cached(&req, res, &options, cached);
      }

      let mut status = StatusCode::OK;
      let mut mime_type = Option::<&String>::None;
      let mut cacheable = true;

      let mut file = 'open: {
        if let Some(resolved) = resolved.as_ref()
          && let Ok(file) = options.file_system.open(&resolved.path).await
        {
          mime_type.replace(&resolved.mime_type);
          break 'open file;
        }

        match options.file_system.open(&fallback.path).await {
          Ok(file) => {
            status = fallback.status;
            mime_type.replace(&fallback.mime_type);
            cacheable = false;
            file
          }
          Err(_) => return res.status(StatusCode::NOT_FOUND).empty(),
        }
      };

      for (key, value) in &options.custom_headers {
        res = res.header(key.as_str(), value.as_str());
      }

      if let Some(mime_type) = mime_type.as_ref() {
        res = res.header(headers::CONTENT_TYPE, mime_type.as_str());
      }

      if let Some(content_encoding) = encoding.header_value() {
        res = res.header(headers::CONTENT_ENCODING, content_encoding);
      }

      let etag = match etag_resolver.as_ref() {
        Some(etag_resolver) => {
          etag_resolver
            .resolve(&mut *file, encoding.etag_suffix())
            .await?
        }
        None => None,
      };

      if let Some(etag) = etag.as_ref() {
        if !has_modified(&req, etag) {
          return res.status(StatusCode::NOT_MODIFIED).empty();
        }
        res = res.header(headers::ETAG, etag.as_str());
      }

      let cache_slot = match (options.cache.as_ref(), cacheable) {
        (Some(cache), true) => {
          let fits = match file.metadata().await {
            Ok(meta) => meta.len() as usize <= cache.max_entry_bytes(),
            Err(_) => false,
          };
          fits.then(|| (Arc::clone(cache), cache.generation()))
        }
        _ => None,
      };

      let (body, writer) = ResponseBody::chunked(BODY_BUFFER_SIZE);
      let mut tee = TeeWriter::new(
        writer,
        cache_slot
          .as_ref()
          .map(|(cache, _)| cache.max_entry_bytes()),
      );

      let content_encoding = encoding.header_value();
      let mime_type = mime_type.cloned();

      tokio::task::spawn(async move {
        let _ = match encoding {
          Encoding::Zstd => zstd_stream(&mut file, &mut tee).await,
          Encoding::Brotli => brotli_stream(&mut file, &mut tee).await,
          Encoding::Gzip => gzip_stream(&mut file, &mut tee).await,
          Encoding::Identity => uncompressed_stream(&mut file, &mut tee).await,
        };
        let _ = tee.shutdown().await;

        if let (Some((cache, generation)), Some(buffered), Some(cache_key)) =
          (cache_slot, tee.into_buffered(), cache_key)
        {
          cache.set(
            cache_key,
            CachedResponse {
              status,
              mime_type: mime_type.unwrap_or_default(),
              content_encoding,
              etag,
              body: Arc::from(buffered),
            },
            generation,
          );
        }
      });

      res.status(status).body(body)
    })
  }))
}

fn has_modified(
  req: &Request,
  etag: &str,
) -> bool {
  if let Some(if_none_match) = req.headers.get(headers::IF_NONE_MATCH)
    && if_none_match == etag
  {
    return false;
  }
  true
}
