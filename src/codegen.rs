//! # Rocket - Code Generation NAPI Bindings
//!
//! Exposes Rocket's code generation facilities to Node.js/TypeScript,
//! including route attribute parsing, macro code generators, derive metadata,
//! and `ROCKET_CODEGEN_DEBUG` logging support.

use std::collections::HashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;

static ROCKET_IDENT_PREFIX: &str = "__rocket_";
static URI_MACRO_PREFIX: &str = "rocket_uri_macro_";

/// Dynamic parameter segment details.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct RouteSegmentMeta {
  pub name: String,
  pub is_trailing: bool,
}

/// Parsed route attribute metadata.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct RouteAttributeMeta {
  pub method: Option<String>,
  pub uri: String,
  pub path_segments: Vec<RouteSegmentMeta>,
  pub query_segments: Vec<RouteSegmentMeta>,
  pub rank: Option<i32>,
  pub format: Option<String>,
  pub data_param: Option<String>,
}

/// Parsed catcher attribute metadata.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct CatcherAttributeMeta {
  pub status_code: Option<u16>,
  pub is_default: bool,
}

/// Structure containing code generation output for Rust and TypeScript.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct GeneratedCode {
  pub rust_code: String,
  pub ts_code: String,
  pub debug_emitted: bool,
}

/// Helper function to handle `ROCKET_CODEGEN_DEBUG` logging.
#[napi]
pub fn emit_codegen_debug(output: String) -> Option<String> {
  if std::env::var_os("ROCKET_CODEGEN_DEBUG").is_some() {
    let msg = format!("[ROCKET_CODEGEN_DEBUG] {output}");
    println!("{msg}");
    Some(msg)
  } else {
    None
  }
}

/// Parses route attribute parameters (e.g. `"/<foo>/bar/<baz..>?<msg>", data = "<form>", rank = 1, format = "json"`).
#[napi]
pub fn parse_route_attribute(method: Option<String>, args: String) -> Result<RouteAttributeMeta> {
  let trimmed = args.trim();
  let parts: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
  let uri_str = if !parts.is_empty() {
    parts[0].trim_matches('"').trim_matches('\'').to_string()
  } else {
    "/".to_string()
  };

  let mut path_segments = Vec::new();
  let mut query_segments = Vec::new();

  let uri_parts: Vec<&str> = uri_str.split('?').collect();
  let path_part = uri_parts[0];
  let query_part = if uri_parts.len() > 1 { Some(uri_parts[1]) } else { None };

  for seg in path_part.split('/') {
    if seg.starts_with('<') && seg.ends_with('>') {
      let inner = seg[1..seg.len() - 1].trim();
      if let Some(stripped) = inner.strip_suffix("..") {
        path_segments.push(RouteSegmentMeta {
          name: stripped.to_string(),
          is_trailing: true,
        });
      } else {
        path_segments.push(RouteSegmentMeta {
          name: inner.to_string(),
          is_trailing: false,
        });
      }
    }
  }

  if let Some(q) = query_part {
    for seg in q.split('&') {
      if seg.starts_with('<') && seg.ends_with('>') {
        let inner = seg[1..seg.len() - 1].trim();
        if let Some(stripped) = inner.strip_suffix("..") {
          query_segments.push(RouteSegmentMeta {
            name: stripped.to_string(),
            is_trailing: true,
          });
        } else {
          query_segments.push(RouteSegmentMeta {
            name: inner.to_string(),
            is_trailing: false,
          });
        }
      } else if seg.contains('=') {
        let kv: Vec<&str> = seg.split('=').collect();
        if kv.len() == 2 && kv[1].starts_with('<') && kv[1].ends_with('>') {
          let inner = kv[1][1..kv[1].len() - 1].trim();
          query_segments.push(RouteSegmentMeta {
            name: inner.to_string(),
            is_trailing: false,
          });
        }
      }
    }
  }

  let mut rank = None;
  let mut format_type = None;
  let mut data_param = None;

  for part in parts.iter().skip(1) {
    if part.starts_with("rank") && part.contains('=') {
      if let Some(val) = part.split('=').nth(1) {
        rank = val.trim().parse::<i32>().ok();
      }
    } else if part.starts_with("format") && part.contains('=') {
      if let Some(val) = part.split('=').nth(1) {
        format_type = Some(val.trim().trim_matches('"').trim_matches('\'').to_string());
      }
    } else if part.starts_with("data") && part.contains('=') {
      if let Some(val) = part.split('=').nth(1) {
        let clean = val.trim().trim_matches('"').trim_matches('\'').trim_matches('<').trim_matches('>');
        data_param = Some(clean.to_string());
      }
    }
  }

  Ok(RouteAttributeMeta {
    method: method.map(|m| m.to_uppercase()),
    uri: uri_str,
    path_segments,
    query_segments,
    rank,
    format: format_type,
    data_param,
  })
}

/// Parses catcher attribute parameters (e.g. `404` or `default`).
#[napi]
pub fn parse_catcher_attribute(args: String) -> Result<CatcherAttributeMeta> {
  let trimmed = args.trim();
  if trimmed == "default" {
    Ok(CatcherAttributeMeta {
      status_code: None,
      is_default: true,
    })
  } else if let Ok(code) = trimmed.parse::<u16>() {
    Ok(CatcherAttributeMeta {
      status_code: Some(code),
      is_default: false,
    })
  } else {
    Err(Error::new(
      Status::InvalidArg,
      format!("Invalid catcher attribute parameter: {args}"),
    ))
  }
}

/// Generates code representation for a route attribute (`#[get(...)]`, `#[post(...)]`, etc.).
#[napi]
pub fn generate_route_code(
  method: String,
  uri: String,
  handler_name: String,
  rank: Option<i32>,
  format_type: Option<String>,
) -> GeneratedCode {
  let method_upper = method.to_uppercase();
  let rank_str = rank.map(|r| format!(", rank = {r}")).unwrap_or_default();
  let format_str = format_type.map(|f| format!(", format = \"{f}\"")).unwrap_or_default();

  let rust_code = format!(
    "#[{method_lower}(\"{uri}\"{rank_str}{format_str})]\nfn {handler_name}() -> &'static str {{ \"OK\" }}",
    method_lower = method_upper.to_lowercase()
  );

  let ts_code = format!(
    "export const {handler_name}Route = createRoute({{ method: \"{method_upper}\", path: \"{uri}\", name: \"{handler_name}\" }});"
  );

  let debug_emitted = emit_codegen_debug(rust_code.clone()).is_some();

  GeneratedCode {
    rust_code,
    ts_code,
    debug_emitted,
  }
}

/// Generates code representation for `routes![handler1, handler2, ...]`.
#[napi]
pub fn generate_routes_macro(handlers: Vec<String>) -> GeneratedCode {
  let list = handlers.join(", ");
  let rust_code = format!("routes![{list}]");
  let ts_code = format!("export const routes = [{list}];");

  let debug_emitted = emit_codegen_debug(rust_code.clone()).is_some();

  GeneratedCode {
    rust_code,
    ts_code,
    debug_emitted,
  }
}

/// Generates code representation for `catchers![catcher1, catcher2, ...]`.
#[napi]
pub fn generate_catchers_macro(catchers: Vec<String>) -> GeneratedCode {
  let list = catchers.join(", ");
  let rust_code = format!("catchers![{list}]");
  let ts_code = format!("export const catchers = [{list}];");

  let debug_emitted = emit_codegen_debug(rust_code.clone()).is_some();

  GeneratedCode {
    rust_code,
    ts_code,
    debug_emitted,
  }
}

/// Generates code representation for `uri!(route_name(p1 = v1, p2 = v2))`.
#[napi]
pub fn generate_uri_macro(route_name: String, params: HashMap<String, String>) -> GeneratedCode {
  let mut args_vec = Vec::new();
  for (k, v) in &params {
    args_vec.push(format!("{k} = \"{v}\""));
  }
  let args_str = args_vec.join(", ");
  let rust_code = format!("uri!({route_name}({args_str}))");

  let mut query_params = Vec::new();
  for (k, v) in &params {
    query_params.push(format!("{k}={v}"));
  }
  let url = if query_params.is_empty() {
    format!("/{route_name}")
  } else {
    format!("/{route_name}?{query_str}", query_str = query_params.join("&"))
  };

  let ts_code = format!("export const {URI_MACRO_PREFIX}{route_name} = \"{url}\";");

  let debug_emitted = emit_codegen_debug(rust_code.clone()).is_some();

  GeneratedCode {
    rust_code,
    ts_code,
    debug_emitted,
  }
}

/// Generates code representation for `#[launch]` entry function.
#[napi]
pub fn generate_launch_code(function_name: Option<String>) -> GeneratedCode {
  let fname = function_name.unwrap_or_else(|| "rocket".to_string());
  let rust_code = format!(
    "#[launch]\nfn {fname}() -> _ {{\n    rocket::build()\n}}"
  );

  let ts_code = format!(
    "export function launchApp() {{\n    const app = RocketServer.build();\n    return app;\n}}"
  );

  let debug_emitted = emit_codegen_debug(rust_code.clone()).is_some();

  GeneratedCode {
    rust_code,
    ts_code,
    debug_emitted,
  }
}
