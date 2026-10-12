//! NAPI bindings for `bytes::Bytes`.

use bytes::Bytes as InnerBytes;
use napi::bindgen_prelude::Buffer;
use napi::{Error, Result, Status};
use napi_derive::napi;

#[napi]
pub struct Bytes {
  inner: InnerBytes,
}

#[napi]
impl Bytes {
  /// Creates a new empty `Bytes`. This does not allocate.
  #[napi(constructor)]
  pub fn new() -> Self {
    Self { inner: InnerBytes::new() }
  }

  /// Creates a `Bytes` by copying the contents of the given buffer.
  #[napi(factory)]
  pub fn from_buffer(buffer: Buffer) -> Self {
    Self { inner: InnerBytes::copy_from_slice(buffer.as_ref()) }
  }

  /// Returns the number of bytes contained in this `Bytes`.
  #[napi]
  pub fn len(&self) -> u32 {
    self.inner.len() as u32
  }

  /// Returns true if the `Bytes` has a length of 0.
  #[napi]
  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }

  /// Returns true if this is the only reference to the data.
  #[napi]
  pub fn is_unique(&self) -> bool {
    self.inner.is_unique()
  }

  /// Returns a slice of self for the half-open range `[start, end)`.
  #[napi]
  pub fn slice(&self, start: u32, end: Option<u32>) -> Result<Bytes> {
    let len = self.inner.len();
    let start = start as usize;
    let end = end.unwrap_or(len as u32) as usize;
    if start > end || end > len {
      return Err(Error::new(
        Status::GenericFailure,
        format!("slice index out of bounds: the range {start}..{end} is not within 0..{len}"),
      ));
    }
    Ok(Bytes { inner: self.inner.slice(start..end) })
  }

  /// Splits the bytes into two at `at`.
  #[napi]
  pub fn split_off(&mut self, at: u32) -> Result<Bytes> {
    let at = at as usize;
    if at > self.inner.len() {
      return Err(Error::new(
        Status::GenericFailure,
        format!("split_off index {at} is greater than length {}", self.inner.len()),
      ));
    }
    Ok(Bytes { inner: self.inner.split_off(at) })
  }

  /// Splits the bytes into two at `at`.
  #[napi]
  pub fn split_to(&mut self, at: u32) -> Result<Bytes> {
    let at = at as usize;
    if at > self.inner.len() {
      return Err(Error::new(
        Status::GenericFailure,
        format!("split_to index {at} is greater than length {}", self.inner.len()),
      ));
    }
    Ok(Bytes { inner: self.inner.split_to(at) })
  }

  /// Shortens the buffer, keeping the first `len` bytes.
  #[napi]
  pub fn truncate(&mut self, len: u32) {
    self.inner.truncate(len as usize);
  }

  /// Clears the buffer, removing all data.
  #[napi]
  pub fn clear(&mut self) {
    self.inner.clear();
  }

  /// Attempts to obtain a uniquely-owned view of the data.
  #[napi]
  pub fn try_into_mut(&self) -> Option<Bytes> {
    if self.inner.is_unique() {
      Some(Bytes { inner: self.inner.clone() })
    } else {
      None
    }
  }

  /// Returns a copy of the visible bytes as a Node `Buffer`.
  #[napi]
  pub fn as_slice(&self) -> Buffer {
    Buffer::from(self.inner.as_ref())
  }

  /// Returns a node:buffer-compatible `Buffer` (a copy) over this `Bytes`.
  #[napi]
  pub fn buf(&self) -> crate::buffer::Buffer {
    crate::buffer::Buffer::from_buffer(self.as_slice())
  }
}

impl Bytes {
  pub(crate) fn from_inner(inner: InnerBytes) -> Self {
    Self { inner }
  }
}
