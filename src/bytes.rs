//! NAPI bindings for `bytes::Bytes`.
//!
//! `Bytes` is a cheaply cloneable, sliceable chunk of contiguous memory. In
//! Rust it dispatches through a vtable so multiple handles can share the same
//! backing storage (reference-counted, static, or owned) and point at
//! different sub-views of it. All of the operations below are O(1) because
//! they only bump a reference count and adjust a few indices.
//!
//! Notes on the NAPI mapping:
//! * The JS class is named `Bytes` (the crate type is aliased to `InnerBytes`
//!   to avoid the name clash).
//! * `slice`/`split_*` take plain integers instead of `RangeBounds`, since JS
//!   has no range type. They validate bounds and return a real JS error rather
//!   than panicking across the FFI boundary.
//! * `as_slice()` hands back a *copy* as a `Buffer`. napi-rs cannot safely
//!   expose a zero-copy view that outlives the call, so this is a deliberate
//!   copy.
//! * `from_static` / `from_owner` / `slice_ref` have no natural JS counterpart
//!   (they require `'static` data, a custom `Send + 'static` owner, or a
//!   sub-slice that is physically inside the buffer) and are documented below
//!   rather than bound.

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
  ///
  /// Corresponds to `Bytes::copy_from_slice` / `Bytes::from(Vec<u8>)`.
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

  /// Returns true if this is the only reference to the data and a conversion
  /// into a uniquely-owned buffer would avoid copying the underlying memory.
  ///
  /// Always returns false for static / owner-backed data.
  #[napi]
  pub fn is_unique(&self) -> bool {
    self.inner.is_unique()
  }

  /// Returns a slice of self for the half-open range `[start, end)`.
  ///
  /// If `end` is omitted it defaults to `len()`. This is an O(1) operation
  /// that increments the reference count; the returned `Bytes` shares the
  /// same backing storage as `self`.
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
  ///
  /// After the call `self` contains elements `[0, at)` and the returned
  /// `Bytes` contains `[at, len)`. O(1).
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
  ///
  /// After the call `self` contains elements `[at, len)` and the returned
  /// `Bytes` contains `[0, at)`. O(1).
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

  /// Shortens the buffer, keeping the first `len` bytes and dropping the rest.
  ///
  /// If `len` is greater than the current length this has no effect.
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
  ///
  /// Returns `Some(owned)` when this handle is the unique owner (so the
  /// conversion is cheap, with no copy), and `None` otherwise. The original
  /// `Bytes` is preserved either way.
  ///
  /// Note: a full `try_into_mut` would yield a `BytesMut` for in-place edits,
  /// which cannot be exposed through napi without a dedicated mutable binding;
  /// this is a faithful read-oriented approximation.
  #[napi]
  pub fn try_into_mut(&self) -> Option<Bytes> {
    if self.inner.is_unique() {
      Some(Bytes { inner: self.inner.clone() })
    } else {
      None
    }
  }

  /// Returns a copy of the visible bytes as a Node `Buffer`.
  ///
  /// This is a copy: napi-rs cannot hand out a zero-copy view that outlives
  /// the call. Use `slice`/`split_*` to share storage cheaply inside Rust.
  #[napi]
  pub fn as_slice(&self) -> Buffer {
    Buffer::from(self.inner.as_ref())
  }

  /// Returns a node:buffer-compatible `Buffer` (a copy) over this `Bytes`.
  ///
  /// The returned `Buffer` is an independent, writable copy, so this `Bytes`
  /// remains usable afterwards.
  #[napi]
  pub fn buf(&self) -> Buffer {
    self.as_slice()
  }
}

// Crate-internal accessors so sibling modules (e.g. `reader.rs`) can build a
// `Reader` over the underlying `bytes::Bytes` without reaching into the
// private field.
impl Bytes {
  #[allow(dead_code)]
  pub(crate) fn from_inner(inner: InnerBytes) -> Self {
    Self { inner }
  }
}

// --- Not bound (documented for completeness) -------------------------------
//
// * `Bytes::from_static(&'static [u8])` — requires `&'static` data that JS
//   cannot provide without leaking memory, so it is intentionally omitted.
//
// * `Bytes::from_owner<T>(owner)` where `T: AsRef<[u8]> + Send + 'static` —
//   needs a custom Rust-owned value (e.g. an `mmap`) as the owner; there is no
//   JS-side type that can satisfy `Send + 'static` here.
//
// * `Bytes::slice_ref(&[u8])` — requires the argument to be a physical sub-slice
//   of this buffer; JS `Buffer`s are independent allocations, so the invariant
//   cannot be upheld from the JS side.
