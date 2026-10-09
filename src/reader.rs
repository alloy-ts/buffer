use bytes::Bytes as InnerBytes;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;

#[napi]
pub struct Reader {
  inner: bytes::buf::Reader<InnerBytes>,
}

impl Reader {
  pub(crate) fn from_inner(inner: InnerBytes) -> Self {
    use bytes::Buf;
    Self {
      inner: inner.reader(),
    }
  }
}

#[napi]
impl Reader {
  #[napi]
  pub fn get_ref(&self) -> crate::buffer::Buffer {
    crate::buffer::Buffer::from_buffer(Buffer::from(self.inner.get_ref().as_ref()))
  }
}
