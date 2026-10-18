use std::io;

use swc_core::common::FileName;
use swc_core::common::GLOBALS;
use swc_core::common::Globals;
use swc_core::common::Mark;
use swc_core::common::SourceMap;
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::sync::Lrc;
use swc_core::ecma::ast::EsVersion;
use swc_core::ecma::codegen::Config as CodegenConfig;
use swc_core::ecma::codegen::Emitter;
use swc_core::ecma::codegen::text_writer::JsWriter;
use swc_core::ecma::minifier::optimize;
use swc_core::ecma::minifier::option::CompressOptions;
use swc_core::ecma::minifier::option::ExtraOptions;
use swc_core::ecma::minifier::option::MangleOptions;
use swc_core::ecma::minifier::option::MinifyOptions;
use swc_core::ecma::parser::Parser;
use swc_core::ecma::parser::StringInput;
use swc_core::ecma::parser::Syntax;
use swc_core::ecma::parser::lexer::Lexer;
use swc_core::ecma::transforms::base::fixer::fixer;
use swc_core::ecma::transforms::base::resolver;

use super::OptimizeOptions;

/// Apply the optimisations to a JavaScript module that is already in its final
/// form.
///
/// Used for sources that are not transpiled, such as the swc helpers, where
/// running the TypeScript transforms over the code would serve no purpose.
pub fn optimize_module(
  source: &str,
  options: OptimizeOptions,
) -> crate::Result<String> {
  if !options.is_enabled() {
    return Ok(source.to_string());
  }

  let cm: Lrc<SourceMap> = Default::default();
  let fm = cm.new_source_file(FileName::Anon.into(), source.to_string());
  let comments = SingleThreadedComments::default();

  let lexer = Lexer::new(
    Syntax::Es(Default::default()),
    EsVersion::latest(),
    StringInput::from(&*fm),
    Some(&comments),
  );

  let mut parser = Parser::new_from(lexer);

  let program = parser
    .parse_program()
    .map_err(|error| io::Error::other(format!("Parse error: {}", error.kind().msg())))?;

  let errors: Vec<String> = parser
    .take_errors()
    .into_iter()
    .map(|error| error.kind().msg().to_string())
    .collect();

  if !errors.is_empty() {
    return Err(io::Error::other(format!("Parse errors: {}", errors.join(", "))).into());
  }

  let globals = Globals::default();

  GLOBALS.set(&globals, || {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::new();

    let mut program = program.apply(resolver(unresolved_mark, top_level_mark, true));

    program = optimize(
      program,
      cm.clone(),
      Some(&comments),
      None,
      &MinifyOptions {
        compress: options.compress.then(CompressOptions::default),
        mangle: options.mangle.then(MangleOptions::default),
        ..MinifyOptions::default()
      },
      &ExtraOptions {
        unresolved_mark,
        top_level_mark,
        mangle_name_cache: None,
      },
    );

    // The minifier can drop or create parentheses, so the fixer has to run
    // after it rather than before.
    program = program.apply(fixer(Some(&comments)));

    let mut codegen_config = CodegenConfig::default();
    codegen_config.minify = options.minify;

    let mut code = Vec::new();

    {
      let mut emitter = Emitter {
        cfg: codegen_config,
        cm: cm.clone(),
        comments: match options.minify {
          true => None,
          false => Some(&comments),
        },
        wr: JsWriter::new(cm.clone(), "\n", &mut code, None),
      };

      emitter.emit_program(&program)?;
    }

    Ok(String::from_utf8(code)?)
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  const SOURCE: &str = r#"
    // a comment
    export function _double(value) {
      return value * 2;
    }
  "#;

  fn optimized(options: OptimizeOptions) -> String {
    optimize_module(SOURCE, options).expect("the fixture optimises")
  }

  #[test]
  fn disabled_returns_the_source_unchanged() {
    assert_eq!(optimized(OptimizeOptions::default()), SOURCE);
  }

  #[test]
  fn minify_strips_whitespace_and_comments() {
    let code = optimized(OptimizeOptions {
      minify: true,
      ..OptimizeOptions::default()
    });

    assert!(!code.contains("// a comment"), "{code}");
    assert!(!code.contains('\n'), "{code}");
    assert!(code.contains("_double"), "{code}");
  }

  #[test]
  fn mangle_keeps_the_exported_name() {
    let code = optimized(OptimizeOptions {
      minify: true,
      mangle: true,
      ..OptimizeOptions::default()
    });

    assert!(code.contains("_double"), "{code}");
  }

  #[test]
  fn compress_rewrites_expressions() {
    let code = optimize_module(
      "export const answer = () => 20 + 22;",
      OptimizeOptions {
        minify: true,
        compress: true,
        ..OptimizeOptions::default()
      },
    )
    .expect("the fixture optimises");

    assert!(code.contains("42"), "{code}");
  }

  #[test]
  fn optimised_output_parses() {
    for options in [
      OptimizeOptions {
        minify: true,
        ..OptimizeOptions::default()
      },
      OptimizeOptions {
        mangle: true,
        ..OptimizeOptions::default()
      },
      OptimizeOptions {
        compress: true,
        ..OptimizeOptions::default()
      },
      OptimizeOptions {
        minify: true,
        mangle: true,
        compress: true,
      },
    ] {
      let code = optimize_module(SOURCE, options).expect("the fixture optimises");

      assert!(
        parses(&code),
        "{options:?} produced code that does not parse:\n{code}"
      );
    }
  }

  #[test]
  fn a_parse_error_is_reported() {
    let error = optimize_module(
      "export const = ;",
      OptimizeOptions {
        minify: true,
        ..OptimizeOptions::default()
      },
    )
    .expect_err("the fixture does not parse");

    // Either the parser or the stray token report is surfaced, both are errors
    assert!(!error.to_string().is_empty());
  }

  fn parses(source: &str) -> bool {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Anon.into(), source.to_string());

    let lexer = Lexer::new(
      Syntax::Es(Default::default()),
      EsVersion::latest(),
      StringInput::from(&*fm),
      None,
    );

    let mut parser = Parser::new_from(lexer);

    parser.parse_program().is_ok() && parser.take_errors().is_empty()
  }
}
