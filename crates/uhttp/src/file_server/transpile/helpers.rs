use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::Ident;
use swc_core::ecma::ast::ImportSpecifier;
use swc_core::ecma::ast::ModuleDecl;
use swc_core::ecma::ast::ModuleExportName;
use swc_core::ecma::ast::ModuleItem;
use swc_core::ecma::ast::Program;
use swc_core::ecma::ast::Str;
use swc_core::ecma::transforms::base::helpers::generated::ALL;
use swc_core::ecma::transforms::base::helpers::generated::HelperDef;

/// Request path, without the leading slash, the helpers are served from.
pub const SWC_HELPERS_DIR: &str = ".swc/";

fn find(name: &str) -> Option<&'static HelperDef> {
  ALL.iter().find(|helper| helper.local == name)
}

/// The name of the helper a request path names, if it names one.
///
/// Helpers are served from `.swc/<name>.js` and are not files on disk, so they
/// are handled before the request is resolved.
pub fn module_name(path: &str) -> Option<&str> {
  path
    .strip_prefix('/')?
    .strip_prefix(SWC_HELPERS_DIR)?
    .strip_suffix(".js")
}

/// One helper, plus the helpers it depends on, as an ES module named after it.
pub fn module_source(name: &str) -> Option<String> {
  let helper = find(name)?;

  let mut source = String::new();

  for dep in ALL {
    if !helper.deps.contains(dep.name) || dep.source.is_empty() {
      continue;
    }

    source.push_str(dep.source);
    if !source.ends_with('\n') {
      source.push('\n');
    }
  }

  source.push_str(&format!("export {{ {} }};\n", helper.local));

  Some(source)
}

/// Point swc's helper imports at the served modules.
///
/// swc asks for each helper as `{ _ as _name }` from `@swc/helpers/_/_name`,
/// the served modules export them under their own names instead.
pub fn rewrite_imports(program: &mut Program) {
  let Some(module) = program.as_mut_module() else {
    return;
  };

  for item in &mut module.body {
    let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item else {
      continue;
    };

    let Some(name) = import
      .src
      .value
      .to_string_lossy()
      .rsplit('/')
      .next()
      .map(ToString::to_string)
    else {
      continue;
    };

    if find(&name).is_none() {
      continue;
    }

    *import.src = Str::from(format!("/{}{}.js", SWC_HELPERS_DIR, name).as_str());

    for specifier in &mut import.specifiers {
      if let ImportSpecifier::Named(named) = specifier {
        named.imported = if named.local.sym == name.as_str() {
          None
        } else {
          Some(ModuleExportName::Ident(Ident::new(
            name.as_str().into(),
            DUMMY_SP,
            named.local.ctxt,
          )))
        };
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn names_the_helper_a_request_path_points_at() {
    assert_eq!(
      module_name("/.swc/_define_property.js"),
      Some("_define_property")
    );
    assert_eq!(
      module_name("/.swc/_apply_decs_2203_r.js"),
      Some("_apply_decs_2203_r")
    );
  }

  #[test]
  fn ignores_paths_that_do_not_name_a_helper_module() {
    assert_eq!(module_name("/main.tsx"), None);
    assert_eq!(module_name("/.swc/_define_property.ts"), None);
    assert_eq!(module_name("/nested/.swc/_define_property.js"), None);
    assert_eq!(
      module_name("/.http-server-rs/swc/_define_property.js"),
      None
    );
  }

  #[test]
  fn serves_the_helper_under_its_own_name() {
    let source = module_source("_define_property").expect("helper is known");

    assert!(source.contains("export { _define_property };"), "{source}");
    assert!(source.contains("function _define_property("), "{source}");
    assert!(module_source("_not_a_helper").is_none());
  }

  #[test]
  fn rewrites_helper_imports_to_the_served_path() {
    let mut program =
      swc_parser("import { _ as _define_property } from '@swc/helpers/_/_define_property';");
    rewrite_imports(&mut program);

    let module = program.as_module().expect("parsed as a module");
    let item = module
      .body
      .first()
      .expect("one item")
      .as_module_decl()
      .unwrap();
    let import = item.as_import().expect("an import");

    assert_eq!(import.src.value, "/.swc/_define_property.js");
  }

  #[test]
  fn leaves_unrelated_imports_alone() {
    let mut program = swc_parser("import { render } from 'preact';");
    rewrite_imports(&mut program);

    let module = program.as_module().expect("parsed as a module");
    let item = module
      .body
      .first()
      .expect("one item")
      .as_module_decl()
      .unwrap();
    let import = item.as_import().expect("an import");

    assert_eq!(import.src.value, "preact");
  }

  fn swc_parser(source: &str) -> Program {
    use swc_core::common::FileName;
    use swc_core::common::SourceMap;
    use swc_core::common::sync::Lrc;
    use swc_core::ecma::ast::EsVersion;
    use swc_core::ecma::parser::Parser;
    use swc_core::ecma::parser::StringInput;
    use swc_core::ecma::parser::Syntax;
    use swc_core::ecma::parser::lexer::Lexer;

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Anon.into(), source.to_string());

    let lexer = Lexer::new(
      Syntax::Es(Default::default()),
      EsVersion::latest(),
      StringInput::from(&*fm),
      None,
    );

    Parser::new_from(lexer)
      .parse_program()
      .expect("the fixture parses")
  }
}
