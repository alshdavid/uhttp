use swc_core::common::FileName;
use swc_core::common::GLOBALS;
use swc_core::common::Globals;
use swc_core::common::Mark;
use swc_core::common::SourceFile;
use swc_core::common::SourceMap;
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::source_map::SourceMapGenConfig;
use swc_core::common::sync::Lrc;
use swc_core::ecma::ast::EsVersion;
use swc_core::ecma::ast::Program;
use swc_core::ecma::codegen::Config as CodegenConfig;
use swc_core::ecma::codegen::Emitter;
use swc_core::ecma::codegen::text_writer::JsWriter;
use swc_core::ecma::parser::Parser;
use swc_core::ecma::parser::StringInput;
use swc_core::ecma::parser::Syntax;
use swc_core::ecma::parser::TsSyntax;
use swc_core::ecma::parser::lexer::Lexer;
use swc_core::ecma::transforms::base::fixer::fixer;
use swc_core::ecma::transforms::base::helpers::HELPERS;
use swc_core::ecma::transforms::base::helpers::Helpers;
use swc_core::ecma::transforms::base::helpers::inject_helpers;
use swc_core::ecma::transforms::base::hygiene::hygiene;
use swc_core::ecma::transforms::base::resolver;
use swc_core::ecma::transforms::proposal::decorator_2022_03::decorator_2022_03;
use swc_core::ecma::transforms::react::Options as ReactOptions;
use swc_core::ecma::transforms::react::Runtime;
use swc_core::ecma::transforms::react::react;
use swc_core::ecma::transforms::typescript::Config as TsConfig;
use swc_core::ecma::transforms::typescript::TsxConfig;
use swc_core::ecma::transforms::typescript::tsx as tsx_strip;
use swc_core::ecma::transforms::typescript::typescript;

use super::TransformerContext;
use super::TransformerResult;
use super::helpers::rewrite_imports;
use super::tsconfig::JsxMode;

/// Options picked from the tsconfig used to drive the transform.
#[derive(Debug, Clone, Default)]
struct JsxConfig {
  runtime: Option<Runtime>,
  pragma: Option<String>,
  pragma_frag: Option<String>,
  import_source: Option<String>,
  development: bool,
  /// Explicitly preserve JSX in the output (skip the JSX transform).
  preserve: bool,
}

/// Translate the nearest tsconfig's `jsx` option into transform settings.
fn jsx_config_from_tsconfig(ctx: &TransformerContext) -> JsxConfig {
  let tsconfig = &ctx.tsconfig;

  // Options for the classic runtime, honouring custom factories.
  let classic = || JsxConfig {
    runtime: Some(Runtime::Classic),
    pragma: tsconfig.jsx_factory.clone(),
    pragma_frag: tsconfig.jsx_fragment_factory.clone(),
    ..JsxConfig::default()
  };

  // Options for the automatic runtime, honouring a custom import source.
  let automatic = |development: bool| JsxConfig {
    runtime: Some(Runtime::Automatic),
    import_source: tsconfig.jsx_import_source.clone(),
    development,
    ..JsxConfig::default()
  };

  match tsconfig.jsx {
    // `react` - the classic runtime
    Some(JsxMode::React) => classic(),
    // `react-jsx` / `react-jsxdev` - the automatic runtime
    Some(JsxMode::ReactJsx) => automatic(false),
    Some(JsxMode::ReactJsxDev) => automatic(true),
    // `react-native` emits JSX and defers the transform to the bundler.
    // Treat it like `preserve` for our purposes.
    Some(JsxMode::Preserve) | Some(JsxMode::ReactNative) => JsxConfig {
      preserve: true,
      ..JsxConfig::default()
    },
    // No `jsx` setting: fall back to the classic runtime, which does not
    // require the browser to resolve `react/jsx-runtime`.
    None => classic(),
  }
}

/// Parse the source, returning every error the parser reported.
fn parse(
  fm: &Lrc<SourceFile>,
  comments: &SingleThreadedComments,
  tsx: bool,
) -> Result<Program, Vec<String>> {
  let lexer = Lexer::new(
    Syntax::Typescript(TsSyntax {
      tsx,
      decorators: true,
      ..TsSyntax::default()
    }),
    EsVersion::latest(),
    StringInput::from(&**fm),
    Some(comments),
  );

  let mut parser = Parser::new_from(lexer);

  let program = parser.parse_program();

  let mut errors: Vec<String> = parser
    .take_errors()
    .into_iter()
    .map(|error| error.kind().msg().to_string())
    .collect();

  match program {
    Ok(program) if errors.is_empty() => Ok(program),
    Ok(_) => Err(errors),
    Err(error) => {
      errors.push(error.kind().msg().to_string());
      Err(errors)
    }
  }
}

/// Source maps carry their sources because the URLs they belong to are served
/// the transpiled output rather than the original.
struct SourceMapConfig {
  source_url: String,
}

impl SourceMapGenConfig for SourceMapConfig {
  fn file_name_to_source(
    &self,
    _: &FileName,
  ) -> String {
    self.source_url.clone()
  }

  fn inline_sources_content(
    &self,
    _: &FileName,
  ) -> bool {
    true
  }
}

pub fn transpile(ctx: TransformerContext) -> anyhow::Result<TransformerResult> {
  let jsx_config = jsx_config_from_tsconfig(&ctx);

  let source = String::from_utf8(ctx.content)?;

  let is_tsx = match ctx.kind.as_str() {
    "ts" => false,
    "tsx" => true,
    _ => {
      anyhow::bail!("Got an invalid file type");
    }
  };

  let cm: Lrc<SourceMap> = Default::default();
  let fm = cm.new_source_file(FileName::Real(ctx.path.clone()).into(), source.clone());
  let comments = SingleThreadedComments::default();

  let program = match parse(&fm, &comments, is_tsx) {
    Ok(program) => program,
    Err(errors) => {
      // If we're parsing a .ts file, it is common for users to accidentally include JSX.
      // Re-parse in TSX mode to detect this case and emit a clearer error.
      if !is_tsx {
        // To avoid false positives (e.g. angle-bracket type assertions), also require
        // an obvious JSX marker in the source when TS parsing fails but TSX parsing succeeds.
        let looks_like_jsx =
          source.contains("</") || source.contains("/>") || source.contains("<>");
        if looks_like_jsx && parse(&fm, &comments, true).is_ok() {
          anyhow::bail!(
            "JSX syntax detected in a .ts file: {}. Rename the file to .tsx or remove JSX.",
            ctx.path.display()
          );
        }
      }

      anyhow::bail!("Parse errors: {}", errors.join(", "));
    }
  };

  let globals = Globals::default();

  // Script programs cannot import the shared helpers module, so swc inlines
  // their helpers instead.
  let external_helpers = program.is_module();

  let (code, map) = GLOBALS.set(&globals, || {
    HELPERS.set(
      &Helpers::new(external_helpers),
      || -> anyhow::Result<(String, Option<String>)> {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        let mut program = program.apply(resolver(unresolved_mark, top_level_mark, true));

        // Stage 3 decorators have to run before the types are stripped, they are
        // free to read type information off the members they decorate.
        program = program.apply(decorator_2022_03());

        let ts_config = TsConfig {
          // Output runs as-is in the browser, so class fields keep their native
          // ([[Define]]) semantics rather than moving into the constructor.
          native_class_properties: true,
          ..TsConfig::default()
        };

        if is_tsx && !jsx_config.preserve {
          program = program.apply(tsx_strip(
            cm.clone(),
            ts_config,
            TsxConfig {
              pragma: Some(
                jsx_config
                  .pragma
                  .clone()
                  .unwrap_or_else(|| "React.createElement".to_string())
                  .into(),
              ),
              pragma_frag: Some(
                jsx_config
                  .pragma_frag
                  .clone()
                  .unwrap_or_else(|| "React.Fragment".to_string())
                  .into(),
              ),
            },
            comments.clone(),
            unresolved_mark,
            top_level_mark,
          ));

          program = program.apply(react(
            cm.clone(),
            Some(comments.clone()),
            ReactOptions {
              runtime: Some(jsx_config.runtime.unwrap_or(Runtime::Classic)),
              import_source: jsx_config.import_source.clone().map(|value| value.into()),
              pragma: jsx_config.pragma.clone().map(|value| value.into()),
              pragma_frag: jsx_config.pragma_frag.clone().map(|value| value.into()),
              development: Some(jsx_config.development),
              ..ReactOptions::default()
            },
            top_level_mark,
            unresolved_mark,
          ));
        } else {
          program = program.apply(typescript(ts_config, unresolved_mark, top_level_mark));
        }

        program = program.apply(inject_helpers(unresolved_mark));
        program = program.apply(hygiene());
        program = program.apply(fixer(Some(&comments)));

        if external_helpers {
          rewrite_imports(&mut program);
        }

        let mut code = Vec::new();
        let mut mappings = ctx.source_maps.then(Vec::new);

        {
          let mut emitter = Emitter {
            cfg: CodegenConfig::default(),
            cm: cm.clone(),
            comments: Some(&comments),
            wr: JsWriter::new(cm.clone(), "\n", &mut code, mappings.as_mut()),
          };

          emitter.emit_program(&program)?;
        }

        let map = match mappings {
          Some(mappings) => {
            let source_map = cm.build_source_map(
              &mappings,
              None,
              SourceMapConfig {
                source_url: ctx.source_url.clone(),
              },
            );

            let mut map = Vec::new();
            source_map.to_writer(&mut map)?;

            Some(String::from_utf8(map)?)
          }
          None => None,
        };

        Ok((String::from_utf8(code)?, map))
      },
    )
  })?;

  Ok(TransformerResult { code, map })
}
