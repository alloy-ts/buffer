use napi_derive::napi;

#[napi]
pub struct RocketServer {
  port: u16,
}

#[napi]
impl RocketServer {
  #[napi(constructor)]
  pub fn new(port: Option<u32>) -> Self {
    Self {
      port: port.unwrap_or(8000) as u16,
    }
  }

  #[napi]
  pub fn get_port(&self) -> u32 {
    self.port as u32
  }

  #[napi]
  pub fn is_running(&self) -> bool {
    false
  }
}
