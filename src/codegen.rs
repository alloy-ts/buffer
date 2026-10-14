//! Rocket Web Framework Code Generation NAPI Bindings
//!
//! Exposes Rocket's route, catcher, form derive, responder derive, type-safe URI
//! generation, and app launching codegen metadata to TypeScript / Node.js.

use napi::{Error, Result, Status};
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

static CODEGEN_DEBUG: AtomicBool = AtomicBool::new(false);

#[napi(string_enum)]
pub enum HttpMethod {
  Get,
  Put,
  Post,
  Delete,
  Head,
  Patch,
  Options,
}

#[napi(object)]
#[derive(Clone)]
pub struct RouteOptions {
  pub rank: Option<i32>,
  pub format: Option<String>,
  pub data: Option<String>,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct RouteMetadata {
  pub method: String,
  pub path: String,
  pub name: String,
  pub rank: Option<i32>,
  pub format: Option<String>,
  pub data: Option<String>,
  pub path_params: Vec<String>,
  pub query_params: Vec<String>,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct CatcherMetadata {
  pub code: Option<u32>,
  pub name: String,
  pub is_default: bool,
}

#[napi(object)]
#[derive(Clone, Default)]
pub struct FormFieldConfig {
  pub name: Option<String>,
  pub uncased: Option<bool>,
  pub default_value: Option<String>,
  pub validate: Option<String>,
}

#[napi(object)]
#[derive(Clone, Default)]
pub struct ResponderConfig {
  pub status: Option<u32>,
  pub content_type: Option<String>,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct ResponseOutput {
  pub status: u32,
  pub content_type: String,
  pub headers: HashMap<String, String>,
  pub body: String,
}

#[napi(object)]
#[derive(Clone, Default)]
pub struct UriOptions {
  pub prefix: Option<String>,
  pub suffix: Option<String>,
}

fn extract_route_params(path: &str) -> (Vec<String>, Vec<String>) {
  let mut path_params = Vec::new();
  let mut query_params = Vec::new();

  let parts: Vec<&str> = path.splitn(2, '?').collect();
  let path_part = parts[0];

  for segment in path_part.split('/') {
    let s = segment.trim();
    if s.starts_with('<') && s.ends_with('>') {
      let mut param_name = &s[1..s.len() - 1];
      if param_name.ends_with("..") {
        param_name = &param_name[..param_name.len() - 2];
      }
      if !param_name.is_empty() && param_name != "_" {
        path_params.push(param_name.to_string());
      }
    }
  }

  if parts.len() > 1 {
    let query_part = parts[1];
    for param in query_part.split('&') {
      let p = param.trim();
      if p.starts_with('<') && p.ends_with('>') {
        let mut param_name = &p[1..p.len() - 1];
        if param_name.ends_with("..") {
          param_name = &param_name[..param_name.len() - 2];
        }
        if !param_name.is_empty() && param_name != "_" {
          query_params.push(param_name.to_string());
        }
      } else if let Some(idx) = p.find('=') {
        let key = &p[..idx];
        if !key.is_empty() {
          query_params.push(key.to_string());
        }
      } else if !p.is_empty() {
        query_params.push(p.to_string());
      }
    }
  }

  (path_params, query_params)
}

/// Route code generation and metadata builder for Rocket routes.
#[napi]
pub struct RouteCodegen {
  method: String,
  path: String,
  name: String,
  rank: Option<i32>,
  format: Option<String>,
  data: Option<String>,
}

#[napi]
impl RouteCodegen {
  #[napi(constructor)]
  pub fn new(
    method: Option<String>,
    path: String,
    name: String,
    options: Option<RouteOptions>,
  ) -> Self {
    let method = method.unwrap_or_else(|| "GET".to_string()).to_uppercase();
    let opts = options.unwrap_or(RouteOptions {
      rank: None,
      format: None,
      data: None,
    });
    RouteCodegen {
      method,
      path,
      name,
      rank: opts.rank,
      format: opts.format,
      data: opts.data,
    }
  }

  #[napi]
  pub fn generate(&self) -> RouteMetadata {
    let (path_params, query_params) = extract_route_params(&self.path);
    RouteMetadata {
      method: self.method.clone(),
      path: self.path.clone(),
      name: self.name.clone(),
      rank: self.rank,
      format: self.format.clone(),
      data: self.data.clone(),
      path_params,
      query_params,
    }
  }

  #[napi]
  pub fn emit_debug(&self) -> String {
    let meta = self.generate();
    format!(
      "Rocket Codegen Debug [{}] fn {}() -> Route {{ path: \"{}\", method: {:?}, rank: {:?}, format: {:?}, data: {:?} }}",
      meta.name, meta.name, meta.path, meta.method, meta.rank, meta.format, meta.data
    )
  }
}

/// Catcher code generation and metadata builder for Rocket error catchers.
#[napi]
pub struct CatcherCodegen {
  code: Option<u32>,
  name: String,
}

#[napi]
impl CatcherCodegen {
  #[napi(constructor)]
  pub fn new(code: Option<u32>, name: String) -> Self {
    CatcherCodegen { code, name }
  }

  #[napi]
  pub fn generate(&self) -> CatcherMetadata {
    CatcherMetadata {
      code: self.code,
      name: self.name.clone(),
      is_default: self.code.is_none(),
    }
  }
}

/// Form code generation for Rocket's `#[derive(FromForm)]`
#[napi]
pub struct FormCodegen {
  struct_name: String,
  fields: Vec<(String, FormFieldConfig)>,
}

#[napi]
impl FormCodegen {
  #[napi(constructor)]
  pub fn new(struct_name: String) -> Self {
    FormCodegen {
      struct_name,
      fields: Vec::new(),
    }
  }

  #[napi(getter)]
  pub fn struct_name(&self) -> String {
    self.struct_name.clone()
  }

  #[napi]
  pub fn add_field(&mut self, field_name: String, config: Option<FormFieldConfig>) {
    let cfg = config.unwrap_or_default();
    self.fields.push((field_name, cfg));
  }

  #[napi]
  pub fn parse_form(&self, query_string: String) -> HashMap<String, String> {
    let mut raw_map = HashMap::new();
    for pair in query_string.split('&') {
      if pair.is_empty() {
        continue;
      }
      let mut kv = pair.splitn(2, '=');
      if let Some(k) = kv.next() {
        let v = kv.next().unwrap_or("");
        raw_map.insert(k.trim().to_string(), v.trim().to_string());
      }
    }

    let mut result = HashMap::new();
    for (field_name, cfg) in &self.fields {
      let mut found_val: Option<String> = None;

      if let Some(ref custom_name) = cfg.name {
        if let Some(val) = raw_map.get(custom_name) {
          found_val = Some(val.clone());
        }
      }

      if found_val.is_none() && cfg.uncased.unwrap_or(false) {
        let lower_field = field_name.to_lowercase();
        for (k, v) in &raw_map {
          if k.to_lowercase() == lower_field {
            found_val = Some(v.clone());
            break;
          }
        }
      }

      if found_val.is_none() {
        if let Some(val) = raw_map.get(field_name) {
          found_val = Some(val.clone());
        }
      }

      let val = found_val
        .or_else(|| cfg.default_value.clone())
        .unwrap_or_default();

      result.insert(field_name.clone(), val);
    }

    result
  }
}

/// Form field enum code generation for Rocket's `#[derive(FromFormField)]`
#[napi]
pub struct FormFieldEnumCodegen {
  enum_name: String,
  variants: HashMap<String, Vec<String>>,
}

#[napi]
impl FormFieldEnumCodegen {
  #[napi(constructor)]
  pub fn new(enum_name: String) -> Self {
    FormFieldEnumCodegen {
      enum_name,
      variants: HashMap::new(),
    }
  }

  #[napi(getter)]
  pub fn enum_name(&self) -> String {
    self.enum_name.clone()
  }

  #[napi]
  pub fn add_variant(&mut self, variant: String, values: Option<Vec<String>>) {
    let vals = values.unwrap_or_else(|| vec![variant.to_lowercase()]);
    self.variants.insert(variant, vals);
  }

  #[napi]
  pub fn parse_value(&self, input: String) -> Result<String> {
    let lower_input = input.to_lowercase();
    for (variant, matched_vals) in &self.variants {
      for val in matched_vals {
        if val.to_lowercase() == lower_input {
          return Ok(variant.clone());
        }
      }
      if variant.to_lowercase() == lower_input {
        return Ok(variant.clone());
      }
    }
    Err(Error::new(
      Status::InvalidArg,
      format!("Unknown variant '{}' for enum '{}'", input, self.enum_name),
    ))
  }
}

/// Responder code generation for Rocket's `#[derive(Responder)]`
#[napi]
pub struct ResponderCodegen {
  type_name: String,
  status: u32,
  content_type: String,
  headers: HashMap<String, String>,
}

#[napi]
impl ResponderCodegen {
  #[napi(constructor)]
  pub fn new(type_name: String, config: Option<ResponderConfig>) -> Self {
    let cfg = config.unwrap_or_default();
    ResponderCodegen {
      type_name,
      status: cfg.status.unwrap_or(200),
      content_type: cfg
        .content_type
        .unwrap_or_else(|| "text/plain".to_string()),
      headers: HashMap::new(),
    }
  }

  #[napi(getter)]
  pub fn type_name(&self) -> String {
    self.type_name.clone()
  }

  #[napi]
  pub fn add_header(&mut self, key: String, value: String) {
    self.headers.insert(key, value);
  }

  #[napi]
  pub fn respond(&self, body: String) -> ResponseOutput {
    ResponseOutput {
      status: self.status,
      content_type: self.content_type.clone(),
      headers: self.headers.clone(),
      body,
    }
  }
}

/// Type-safe URI generation for Rocket's `uri!` macro
#[napi]
pub struct UriCodegen {
  uri_pattern: String,
}

#[napi]
impl UriCodegen {
  #[napi(constructor)]
  pub fn new(uri_pattern: String) -> Self {
    UriCodegen { uri_pattern }
  }

  #[napi]
  pub fn build(
    &self,
    params: HashMap<String, String>,
    options: Option<UriOptions>,
  ) -> Result<String> {
    let opts = options.unwrap_or_default();
    let parts: Vec<&str> = self.uri_pattern.splitn(2, '?').collect();
    let path_pattern = parts[0];

    // 1. Process path parameters on path_pattern
    let mut path_part = path_pattern.to_string();
    for (key, value) in &params {
      let target = format!("<{}>", key);
      let encoded_val = urlencoding_simple(value);
      path_part = path_part.replace(&target, &encoded_val);

      let target_trail = format!("<{}..>", key);
      path_part = path_part.replace(&target_trail, &encoded_val);
    }

    // 2. Process query parameters
    let mut final_query = Vec::new();
    if parts.len() > 1 {
      for q in parts[1].split('&') {
        let q = q.trim();
        if q.starts_with('<') && q.ends_with('>') {
          let param_name = &q[1..q.len() - 1];
          if let Some(val) = params.get(param_name) {
            if val != "_" && !val.is_empty() {
              final_query.push(format!("{}={}", param_name, urlencoding_simple(val)));
            }
          }
        } else if let Some(idx) = q.find('=') {
          let key = &q[..idx];
          let val = &q[idx + 1..];
          if val.starts_with('<') && val.ends_with('>') {
            let param_name = &val[1..val.len() - 1];
            if let Some(p_val) = params.get(param_name) {
              if p_val != "_" && !p_val.is_empty() {
                final_query.push(format!("{}={}", key, urlencoding_simple(p_val)));
              }
            }
          } else {
            final_query.push(q.to_string());
          }
        } else {
          final_query.push(q.to_string());
        }
      }
    }

    let mut full_uri = path_part;
    if !final_query.is_empty() {
      full_uri.push('?');
      full_uri.push_str(&final_query.join("&"));
    }

    if let Some(prefix) = opts.prefix {
      if prefix.ends_with('/') && full_uri.starts_with('/') {
        full_uri = format!("{}{}", prefix, &full_uri[1..]);
      } else if !prefix.ends_with('/') && !full_uri.starts_with('/') {
        full_uri = format!("{}/{}", prefix, full_uri);
      } else {
        full_uri = format!("{}{}", prefix, full_uri);
      }
    }

    if let Some(suffix) = opts.suffix {
      full_uri.push_str(&suffix);
    }

    Ok(full_uri)
  }
}

fn urlencoding_simple(s: &str) -> String {
  let mut out = String::new();
  for b in s.bytes() {
    match b {
      b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
        out.push(b as char);
      }
      _ => {
        out.push_str(&format!("%{:02X}", b));
      }
    }
  }
  out
}

/// Rocket Application instance simulating `rocket::build()`
#[napi]
pub struct RocketApp {
  routes: Vec<RouteMetadata>,
  catchers: Vec<CatcherMetadata>,
}

#[napi]
impl RocketApp {
  #[napi(constructor)]
  pub fn new() -> Self {
    RocketApp {
      routes: Vec::new(),
      catchers: Vec::new(),
    }
  }

  #[napi]
  pub fn mount(&mut self, base_path: String, routes: Vec<&RouteCodegen>) {
    let clean_base = if base_path.ends_with('/') {
      &base_path[..base_path.len() - 1]
    } else {
      &base_path[..]
    };

    for r in routes {
      let mut meta = r.generate();
      meta.path = format!("{}{}", clean_base, meta.path);
      self.routes.push(meta);
    }
  }

  #[napi]
  pub fn catch(&mut self, catchers: Vec<&CatcherCodegen>) {
    for c in catchers {
      self.catchers.push(c.generate());
    }
  }

  #[napi]
  pub fn launch(&self) -> String {
    format!(
      "🚀 Rocket web framework initialized with {} routes and {} catchers",
      self.routes.len(),
      self.catchers.len()
    )
  }

  #[napi]
  pub fn routes_summary(&self) -> Vec<RouteMetadata> {
    self.routes.clone()
  }

  #[napi]
  pub fn catchers_summary(&self) -> Vec<CatcherMetadata> {
    self.catchers.clone()
  }
}

// --- Helper Functions ---

#[napi]
pub fn routes(route_list: Vec<&RouteCodegen>) -> Vec<RouteMetadata> {
  route_list.iter().map(|r| r.generate()).collect()
}

#[napi]
pub fn catchers(catcher_list: Vec<&CatcherCodegen>) -> Vec<CatcherMetadata> {
  catcher_list.iter().map(|c| c.generate()).collect()
}

#[napi]
pub fn generate_route(
  method: String,
  uri: String,
  name: String,
  options: Option<RouteOptions>,
) -> RouteMetadata {
  RouteCodegen::new(Some(method), uri, name, options).generate()
}

#[napi]
pub fn generate_catcher(code: Option<u32>, name: String) -> CatcherMetadata {
  CatcherCodegen::new(code, name).generate()
}

#[napi]
pub fn generate_uri(
  uri_pattern: String,
  params: HashMap<String, String>,
  options: Option<UriOptions>,
) -> Result<String> {
  UriCodegen::new(uri_pattern).build(params, options)
}

#[napi]
pub fn is_codegen_debug_enabled() -> bool {
  if std::env::var_os("ROCKET_CODEGEN_DEBUG").is_some() {
    return true;
  }
  CODEGEN_DEBUG.load(Ordering::Relaxed)
}

#[napi]
pub fn set_codegen_debug(enabled: bool) {
  CODEGEN_DEBUG.store(enabled, Ordering::Relaxed);
}
