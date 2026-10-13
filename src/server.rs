//! NAPI bindings for a Rocket-style HTTP Web Server.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct Request {
  pub method: String,
  pub uri: String,
  pub headers: HashMap<String, String>,
  pub body: String,
}

#[napi(object)]
pub struct Response {
  pub status: u32,
  pub headers: Option<HashMap<String, String>>,
  pub body: Option<String>,
}

#[napi(object)]
pub struct Route {
  pub method: String,
  pub path: String,
}

#[napi]
pub struct Server {
  port: u16,
  address: String,
  routes: Arc<Mutex<HashMap<String, String>>>,
  is_running: Arc<Mutex<bool>>,
}

#[napi]
impl Server {
  #[napi(constructor)]
  pub fn new(port: Option<u32>, address: Option<String>) -> Self {
    Self {
      port: port.unwrap_or(8000) as u16,
      address: address.unwrap_or_else(|| "127.0.0.1".to_string()),
      routes: Arc::new(Mutex::new(HashMap::new())),
      is_running: Arc::new(Mutex::new(false)),
    }
  }

  #[napi]
  pub fn get_port(&self) -> u32 {
    self.port as u32
  }

  #[napi]
  pub fn get_address(&self) -> String {
    self.address.clone()
  }

  #[napi]
  pub fn mount(&self, base_path: String, routes: Vec<Route>) {
    let mut guard = self.routes.lock().unwrap();
    for route in routes {
      let key = format!("{} {}{}", route.method.to_uppercase(), base_path, route.path);
      guard.insert(key, "Rocket route mounted".to_string());
    }
  }

  #[napi]
  pub fn route(&self, method: String, path: String, response_body: String) {
    let mut guard = self.routes.lock().unwrap();
    let key = format!("{} {}", method.to_uppercase(), path);
    guard.insert(key, response_body);
  }

  #[napi]
  pub fn launch(&self) -> Result<String> {
    let addr = format!("{}:{}", self.address, self.port);
    let listener = TcpListener::bind(&addr)
      .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to bind to {addr}: {e}")))?;

    *self.is_running.lock().unwrap() = true;
    let routes = self.routes.clone();
    let is_running = self.is_running.clone();

    thread::spawn(move || {
      listener.set_nonblocking(true).ok();
      while *is_running.lock().unwrap() {
        if let Ok((mut stream, _)) = listener.accept() {
          let mut buffer = [0u8; 1024];
          if let Ok(n) = stream.read(&mut buffer) {
            if n > 0 {
              let req_str = String::from_utf8_lossy(&buffer[..n]);
              let mut lines = req_str.lines();
              let first_line = lines.next().unwrap_or("");
              let parts: Vec<&str> = first_line.split_whitespace().collect();

              let body = if parts.len() >= 2 {
                let key = format!("{} {}", parts[0].to_uppercase(), parts[1]);
                let guard = routes.lock().unwrap();
                guard.get(&key).cloned().unwrap_or_else(|| "404 Not Found".to_string())
              } else {
                "400 Bad Request".to_string()
              };

              let res_status = if body == "404 Not Found" { "404 Not Found" } else { "200 OK" };
              let http_response = format!(
                "HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                res_status,
                body.len(),
                body
              );
              stream.write_all(http_response.as_bytes()).ok();
            }
          }
        }
        thread::sleep(std::time::Duration::from_millis(10));
      }
    });

    Ok(format!("Rocket server launching on http://{addr}"))
  }

  #[napi]
  pub fn stop(&self) {
    *self.is_running.lock().unwrap() = false;
  }
}
