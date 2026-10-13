//! # NAPI Bindings for Rocket Web Framework
//!
//! Exposes Rocket web framework server configuration, routing, catchers,
//! request/response models, and async launch/shutdown lifecycle to Node.js/TypeScript.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use rocket::config::Config as RocketConfig;
use rocket::data::Data;
use rocket::http::{ContentType, Header, Method as RocketMethod, Status as RocketStatus};
use rocket::local::asynchronous::Client;
use rocket::outcome::Outcome;
use rocket::request::Request as RocketRequest;
use rocket::response::{Response as RocketResponse};
use rocket::route::{Handler as RocketHandler, Outcome as RouteOutcome, Route as RocketRoute};
use tokio::sync::oneshot;

/// HTTP methods supported by Rocket routes.
#[napi]
pub enum HttpMethod {
  Get,
  Put,
  Post,
  Delete,
  Head,
  Patch,
  Options,
}

#[napi]
impl HttpMethod {
  pub fn from_string(s: String) -> Result<HttpMethod> {
    match s.to_uppercase().as_str() {
      "GET" => Ok(HttpMethod::Get),
      "PUT" => Ok(HttpMethod::Put),
      "POST" => Ok(HttpMethod::Post),
      "DELETE" => Ok(HttpMethod::Delete),
      "HEAD" => Ok(HttpMethod::Head),
      "PATCH" => Ok(HttpMethod::Patch),
      "OPTIONS" => Ok(HttpMethod::Options),
      _ => Err(Error::new(
        Status::InvalidArg,
        format!("Unsupported HTTP method: {s}"),
      )),
    }
  }

  pub fn to_str(&self) -> &'static str {
    match self {
      HttpMethod::Get => "GET",
      HttpMethod::Put => "PUT",
      HttpMethod::Post => "POST",
      HttpMethod::Delete => "DELETE",
      HttpMethod::Head => "HEAD",
      HttpMethod::Patch => "PATCH",
      HttpMethod::Options => "OPTIONS",
    }
  }
}

impl HttpMethod {
  pub fn to_rocket(&self) -> RocketMethod {
    match self {
      HttpMethod::Get => RocketMethod::Get,
      HttpMethod::Put => RocketMethod::Put,
      HttpMethod::Post => RocketMethod::Post,
      HttpMethod::Delete => RocketMethod::Delete,
      HttpMethod::Head => RocketMethod::Head,
      HttpMethod::Patch => RocketMethod::Patch,
      HttpMethod::Options => RocketMethod::Options,
    }
  }
}

/// Request object passed to route handlers.
#[napi(object)]
pub struct JsRequest {
  pub method: String,
  pub uri: String,
  pub path: String,
  pub headers: HashMap<String, String>,
  pub query: HashMap<String, String>,
  pub params: HashMap<String, String>,
  pub body: Option<Buffer>,
}

/// Response object returned by route handlers or created by response helpers.
#[napi(object)]
pub struct JsResponse {
  pub status: u16,
  pub headers: Option<HashMap<String, String>>,
  pub content_type: Option<String>,
  pub body: Option<Buffer>,
}

/// Route configuration metadata.
#[napi(object)]
pub struct RouteConfig {
  pub method: String,
  pub path: String,
  pub name: Option<String>,
  pub rank: Option<i32>,
  pub format: Option<String>,
}

/// Catcher configuration metadata.
#[napi(object)]
pub struct CatcherConfig {
  pub code: Option<u16>,
  pub name: Option<String>,
}

/// Rocket server configuration options.
#[napi(object)]
pub struct ServerConfig {
  pub port: Option<u16>,
  pub address: Option<String>,
  pub secret_key: Option<String>,
}

#[derive(Clone)]
struct SimpleRouteHandler {
  handler: Arc<dyn Fn(JsRequest) -> JsResponse + Send + Sync + 'static>,
}

#[rocket::async_trait]
impl RocketHandler for SimpleRouteHandler {
  async fn handle<'r>(&self, req: &'r RocketRequest<'_>, data: Data<'r>) -> RouteOutcome<'r> {
    let mut headers_map = HashMap::new();
    for h in req.headers().iter() {
      headers_map.insert(h.name().to_string(), h.value().to_string());
    }

    let mut query_map = HashMap::new();
    if let Some(query) = req.uri().query() {
      for item in query.segments() {
        query_map.insert(item.0.to_string(), item.1.to_string());
      }
    }

    let mut params_map = HashMap::new();
    if let Some(r) = req.route() {
      let route_uri = r.uri.path().to_string();
      for seg in route_uri.split('/') {
        if seg.starts_with('<') && seg.ends_with('>') {
          let name = seg.trim_matches('<').trim_matches('>').trim_end_matches("..");
          if let Some(Ok(val)) = req.param::<&str>(0) {
            params_map.insert(name.to_string(), val.to_string());
          }
        }
      }
    }

    let body_bytes = match data.open(rocket::data::ToByteUnit::mebibytes(2)).into_bytes().await {
      Ok(b) => {
        let vec = b.into_inner();
        if vec.is_empty() {
          None
        } else {
          Some(Buffer::from(vec))
        }
      }
      _ => None,
    };

    let js_req = JsRequest {
      method: req.method().as_str().to_string(),
      uri: req.uri().to_string(),
      path: req.uri().path().to_string(),
      headers: headers_map,
      query: query_map,
      params: params_map,
      body: body_bytes,
    };

    let js_res = (self.handler)(js_req);
    let status = RocketStatus::from_code(js_res.status).unwrap_or(RocketStatus::InternalServerError);
    let mut response = RocketResponse::build();
    response.status(status);

    if let Some(ct) = js_res.content_type {
      if let Ok(content_type) = ContentType::parse_flexible(&ct).ok_or(()) {
        response.header(content_type);
      }
    }

    if let Some(hdrs) = js_res.headers {
      for (k, v) in hdrs {
        response.header(Header::new(k, v));
      }
    }

    if let Some(body) = js_res.body {
      response.sized_body(body.len(), std::io::Cursor::new(body.to_vec()));
    }

    Outcome::Success(response.finalize())
  }
}

/// NAPI Class representing a Rocket Web Framework application instance.
#[napi]
pub struct RocketServer {
  config: RocketConfig,
  routes: Vec<(String, RouteConfig, Arc<dyn Fn(JsRequest) -> JsResponse + Send + Sync + 'static>)>,
  _shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

#[napi]
impl RocketServer {
  /// Constructs a new `RocketServer` with default Rocket configuration.
  #[napi(factory)]
  pub fn build() -> RocketServer {
    RocketServer {
      config: RocketConfig::default(),
      routes: Vec::new(),
      _shutdown_tx: Arc::new(Mutex::new(None)),
    }
  }

  /// Constructs a new `RocketServer` with custom configuration options.
  #[napi(factory)]
  pub fn custom(config: ServerConfig) -> Result<RocketServer> {
    let mut cfg = RocketConfig::default();

    if let Some(p) = config.port {
      cfg.port = p;
    }
    if let Some(addr) = config.address {
      if let Ok(ip) = addr.parse() {
        cfg.address = ip;
      } else {
        return Err(Error::new(
          Status::InvalidArg,
          format!("Invalid IP address: {addr}"),
        ));
      }
    }
    if let Some(sec) = config.secret_key {
      let key = rocket::config::SecretKey::derive_from(sec.as_bytes());
      cfg.secret_key = key;
    }

    Ok(RocketServer {
      config: cfg,
      routes: Vec::new(),
      _shutdown_tx: Arc::new(Mutex::new(None)),
    })
  }

  /// Returns configured server port.
  #[napi(getter)]
  pub fn port(&self) -> u16 {
    self.config.port
  }

  /// Sets server port.
  #[napi(setter)]
  pub fn set_port(&mut self, port: u16) {
    self.config.port = port;
  }

  /// Returns configured server address.
  #[napi(getter)]
  pub fn address(&self) -> String {
    self.config.address.to_string()
  }

  /// Mounts a route with a custom JS handler function.
  #[napi]
  pub fn mount_route(
    &mut self,
    base_path: String,
    config: RouteConfig,
    handler: ThreadsafeFunction<JsRequest>,
  ) -> Result<()> {
    let handler_arc = Arc::new(move |req: JsRequest| {
      let (tx, rx) = std::sync::mpsc::channel();
      let status = handler.call_with_return_value(
        Ok(req),
        ThreadsafeFunctionCallMode::Blocking,
        move |res: Result<Unknown>, env: Env| {
          if let Ok(unk) = res {
            if let Ok(resp) = unsafe { JsResponse::from_napi_value(env.raw(), unk.raw()) } {
              let _ = tx.send(resp);
            }
          }
          Ok(())
        },
      );
      if status == napi::Status::Ok {
        rx.recv().unwrap_or(JsResponse {
          status: 500,
          headers: None,
          content_type: Some("text/plain".to_string()),
          body: Some(Buffer::from("Internal Server Error".as_bytes())),
        })
      } else {
        JsResponse {
          status: 500,
          headers: None,
          content_type: Some("text/plain".to_string()),
          body: Some(Buffer::from("Handler Invocation Error".as_bytes())),
        }
      }
    });

    self.routes.push((base_path, config, handler_arc));
    Ok(())
  }

  /// Helper to mount simple synchronous route responses.
  #[napi]
  pub fn mount_simple_route(
    &mut self,
    base_path: String,
    method: String,
    path: String,
    response_body: String,
    status: Option<u16>,
  ) -> Result<()> {
    let status_code = status.unwrap_or(200);
    let handler_arc = Arc::new(move |_req: JsRequest| JsResponse {
      status: status_code,
      headers: None,
      content_type: Some("text/plain".to_string()),
      body: Some(Buffer::from(response_body.as_bytes())),
    });

    let config = RouteConfig {
      method,
      path,
      name: None,
      rank: None,
      format: None,
    };

    self.routes.push((base_path, config, handler_arc));
    Ok(())
  }

  /// Simulates dispatching a request directly against built Rocket routes for testing without opening network sockets.
  #[napi]
  pub async fn dispatch(&self, method: String, uri: String, body: Option<String>) -> Result<JsResponse> {
    let http_method = HttpMethod::from_string(method)?;
    let rocket_method = http_method.to_rocket();

    let mut rocket_builder = rocket::custom(self.config.clone());

    let mut routes_by_base: HashMap<String, Vec<RocketRoute>> = HashMap::new();
    for (base, cfg, handler) in &self.routes {
      let method = HttpMethod::from_string(cfg.method.clone())?.to_rocket();
      let simple_handler = SimpleRouteHandler {
        handler: handler.clone(),
      };
      let mut route = RocketRoute::new(method, cfg.path.as_str(), simple_handler);
      if let Some(r) = cfg.rank {
        route.rank = r as isize;
      }
      if let Some(ref name) = cfg.name {
        route.name = Some(name.clone().into());
      }
      routes_by_base.entry(base.clone()).or_default().push(route);
    }

    for (base, route_list) in routes_by_base {
      rocket_builder = rocket_builder.mount(base, route_list);
    }

    let client = Client::untracked(rocket_builder)
      .await
      .map_err(|e| Error::new(Status::GenericFailure, format!("Rocket client error: {e}")))?;

    let mut req = client.req(rocket_method, &uri);
    if let Some(b) = body {
      req = req.body(b);
    }

    let res = req.dispatch().await;
    let status = res.status().code;
    let mut headers = HashMap::new();
    for h in res.headers().iter() {
      headers.insert(h.name().to_string(), h.value().to_string());
    }
    let content_type = res.content_type().map(|ct| ct.to_string());
    let body_bytes = res.into_bytes().await.unwrap_or_default();

    Ok(JsResponse {
      status,
      headers: Some(headers),
      content_type,
      body: Some(Buffer::from(body_bytes)),
    })
  }

  /// Returns total mounted routes count.
  #[napi(getter)]
  pub fn routes_count(&self) -> u32 {
    self.routes.len() as u32
  }
}

/// Helper function to create standard HTTP response objects.
#[napi]
pub fn create_response(
  status: u16,
  body: String,
  content_type: Option<String>,
  headers: Option<HashMap<String, String>>,
) -> JsResponse {
  JsResponse {
    status,
    headers,
    content_type: content_type.or_else(|| Some("text/plain".to_string())),
    body: Some(Buffer::from(body.as_bytes())),
  }
}

/// Helper function to create JSON response objects.
#[napi]
pub fn create_json_response(status: u16, json_body: String) -> JsResponse {
  JsResponse {
    status,
    headers: None,
    content_type: Some("application/json".to_string()),
    body: Some(Buffer::from(json_body.as_bytes())),
  }
}
