use bytes::Bytes as InnerBytes;
use napi_derive::napi;

#[napi]
pub struct Reader {
  inner: InnerBytes,
}

impl Reader {
  pub(crate) fn from_inner(inner: InnerBytes) -> Self {
    Self { inner }
  }
}

#[napi]
impl Reader {
  #[napi(getter)]
  pub fn length(&self) -> u32 {
    self.inner.len() as u32
  }
}
