use bytes::buf::Reader as InnerReader;
use bytes::Bytes as InnerBytes;
use napi_derive::napi;

#[napi]
pub struct Reader {
  inner: InnerReader<InnerBytes>,
}

impl Reader {
  pub(crate) fn from_inner(bytes: InnerBytes) -> Self {
    use bytes::Buf;
    Self {
      inner: bytes.reader(),
    }
  }
}
