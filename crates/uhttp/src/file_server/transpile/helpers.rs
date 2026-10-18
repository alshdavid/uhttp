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
pub const SWC_HELPERS_DIR: &str = ".http-server-rs/swc/";

fn find(name: &str) -> Option<&'static HelperDef> {
  ALL.iter().find(|helper| helper.local == name)
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
