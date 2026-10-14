//! NAPI bindings for `Alloy.serve` - Bun.serve compatible HTTP server implementation.

use napi::Result;
use napi_derive::napi;
use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[napi(object)]
#[derive(Clone, Default)]
pub struct ServerOptions {
  pub port: Option<u32>,
  pub hostname: Option<String>,
  pub unix: Option<String>,
  pub development: Option<bool>,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct RouteMatchResult {
  pub matched: bool,
  pub route_pattern: String,
  pub params: HashMap<String, String>,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct HttpResponse {
  pub status: u16,
  pub status_text: String,
  pub headers: HashMap<String, String>,
  pub body: String,
}

#[napi]
pub struct Server {
  port: u16,
  hostname: String,
  unix_path: Option<String>,
  url: String,
  development: bool,
  stopped: Arc<AtomicBool>,
  routes: HashMap<String, String>,
}

#[napi]
impl Server {
  #[napi(getter)]
  pub fn port(&self) -> u32 {
    self.port as u32
  }

  #[napi(getter)]
  pub fn hostname(&self) -> String {
    self.hostname.clone()
  }

  #[napi(getter)]
  pub fn unix_path(&self) -> Option<String> {
    self.unix_path.clone()
  }

  #[napi(getter)]
  pub fn url(&self) -> String {
    self.url.clone()
  }

  #[napi(getter)]
  pub fn development(&self) -> bool {
    self.development
  }

  #[napi(getter)]
  pub fn pending_requests(&self) -> u32 {
    0
  }

  #[napi]
  pub fn stop(&self, _force: Option<bool>) {
    self.stopped.store(true, Ordering::Relaxed);
  }

  #[napi]
  pub fn ref_server(&self) {
    // Event loop handle ref
  }

  #[napi]
  pub fn unref_server(&self) {
    // Event loop handle unref
  }

  #[napi]
  pub fn match_route(&self, path: String) -> RouteMatchResult {
    // 1. Exact match
    if let Some(_target) = self.routes.get(&path) {
      return RouteMatchResult {
        matched: true,
        route_pattern: path,
        params: HashMap::new(),
      };
    }

    // 2. Dynamic route match e.g. /users/:id
    for (pattern, _target) in &self.routes {
      if pattern.contains(':') {
        let pat_segments: Vec<&str> = pattern.split('/').collect();
        let path_segments: Vec<&str> = path.split('/').collect();

        if pat_segments.len() == path_segments.len() {
          let mut params = HashMap::new();
          let mut matched = true;

          for (pat_seg, path_seg) in pat_segments.iter().zip(path_segments.iter()) {
            if pat_seg.starts_with(':') {
              let param_name = &pat_seg[1..];
              params.insert(param_name.to_string(), path_seg.to_string());
            } else if pat_seg != path_seg {
              matched = false;
              break;
            }
          }

          if matched {
            return RouteMatchResult {
              matched: true,
              route_pattern: pattern.clone(),
              params,
            };
          }
        }
      }
    }

    // 3. Wildcard match e.g. /api/*
    for (pattern, _target) in &self.routes {
      if pattern.ends_with("/*") {
        let prefix = &pattern[..pattern.len() - 2];
        if path.starts_with(prefix) {
          return RouteMatchResult {
            matched: true,
            route_pattern: pattern.clone(),
            params: HashMap::new(),
          };
        }
      }
    }

    RouteMatchResult {
      matched: false,
      route_pattern: String::new(),
      params: HashMap::new(),
    }
  }

  #[napi]
  pub fn add_route(&mut self, path: String, handler_description: String) {
    self.routes.insert(path, handler_description);
  }
}

fn resolve_port(requested_port: Option<u32>) -> u16 {
  if let Some(p) = requested_port {
    if p == 0 {
      // Find a random available port
      if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
        if let Ok(addr) = listener.local_addr() {
          return addr.port();
        }
      }
      return 3000;
    }
    return p as u16;
  }

  // Check environment variables $BUN_PORT, $PORT, $NODE_PORT
  if let Ok(val) = std::env::var("BUN_PORT") {
    if let Ok(p) = val.parse::<u16>() {
      return p;
    }
  }
  if let Ok(val) = std::env::var("PORT") {
    if let Ok(p) = val.parse::<u16>() {
      return p;
    }
  }
  if let Ok(val) = std::env::var("NODE_PORT") {
    if let Ok(p) = val.parse::<u16>() {
      return p;
    }
  }

  3000
}

/// Bun.serve / Alloy.serve compatible server creation function.
#[napi]
pub fn serve(options: Option<ServerOptions>) -> Result<Server> {
  let opts = options.unwrap_or_default();

  if let Some(ref unix_path) = opts.unix {
    let hostname = if unix_path.starts_with('\0') {
      format!("abstract:{}", &unix_path[1..])
    } else {
      unix_path.clone()
    };
    let url = format!("unix://{}", hostname);

    return Ok(Server {
      port: 0,
      hostname,
      unix_path: Some(unix_path.clone()),
      url,
      development: opts.development.unwrap_or(false),
      stopped: Arc::new(AtomicBool::new(false)),
      routes: HashMap::new(),
    });
  }

  let port = resolve_port(opts.port);
  let hostname = opts.hostname.unwrap_or_else(|| "0.0.0.0".to_string());

  let display_host = if hostname == "0.0.0.0" || hostname == "::" {
    "localhost".to_string()
  } else {
    hostname.clone()
  };

  let url = format!("http://{}:{}", display_host, port);

  Ok(Server {
    port,
    hostname,
    unix_path: None,
    url,
    development: opts.development.unwrap_or(false),
    stopped: Arc::new(AtomicBool::new(false)),
    routes: HashMap::new(),
  })
}

#[napi(object)]
pub struct FileResponse {
  pub path: String,
  pub mime_type: String,
}

#[napi]
pub fn create_file_response(path: String) -> FileResponse {
  let mime_type = if path.ends_with(".html") {
    "text/html"
  } else if path.ends_with(".json") {
    "application/json"
  } else if path.ends_with(".ico") {
    "image/x-icon"
  } else if path.ends_with(".css") {
    "text/css"
  } else if path.ends_with(".js") {
    "application/javascript"
  } else {
    "application/octet-stream"
  };

  FileResponse {
    path,
    mime_type: mime_type.to_string(),
  }
}

#[napi]
pub fn create_redirect_response(url: String, status: Option<u16>) -> HttpResponse {
  let mut headers = HashMap::new();
  headers.insert("Location".to_string(), url);
  HttpResponse {
    status: status.unwrap_or(302),
    status_text: "Found".to_string(),
    headers,
    body: String::new(),
  }
}
