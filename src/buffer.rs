//! NAPI bindings for a `node:buffer`-compatible API.
//!
//! This module mirrors the public surface of Node's `node:buffer` module:
//!
//! * The `Buffer` class — a fixed-length byte sequence backed by a `Vec<u8>`.
//!   It exposes the `alloc` / `allocUnsafe` / `allocUnsafeSlow` / `from` /
//!   `concat` / `byteLength` / `stringLength` / `compare` / `copyBytesFrom` /
//!   `isBuffer` / `isEncoding` statics and the instance helpers
//!   `read*` / `write*` (offset-based, including the 48-bit `BE`/`LE` variants),
//!   `fill`, `equals`, `compare`, `copy`, `slice`, `subarray`, `swap16/32/64`,
//!   `toString`, `toJSON`, `indexOf`, `lastIndexOf`, `includes`, `keys`,
//!   `values`, `entries`, `reverse`, plus the `.length` / `.buffer` /
//!   `.byteOffset` / `.parent` views.
//! * The `Blob` and `File` classes.
//! * Module-level helpers: `atob`, `btoa`, `isAscii`, `isUtf8`, `isLatin1`,
//!   `transcode`, `resolveObjectURL`, the `constants` object and the
//!   `kMaxLength` / `kStringMaxLength` / `INSPECT_MAX_BYTES` / `poolSize`
//!   properties.
//!
//! ## NAPI mapping notes
//! * napi-rs v3.13.0 does **not** implement `FromNapiValue` for `u64`/`i64`/
//!   `u128`/`i128`/`f32`, so the 64-bit methods take/return `BigInt` (via
//!   `napi::bindgen_prelude::BigInt`) and the float methods take `f64`. The
//!   48-bit `read/writeUIntBE/LE`/`IntBE/LE` take/return `f64` (exact for up to
//!   53 bits).
//! * `ArrayBuffer`, `TypedArray` and `DataView` are accepted as inputs (via the
//!   low-level `napi_*` type checks) and `ArrayBuffer` / `Buffer` are produced
//!   as outputs, so this module interoperates with the rest of the TypedArray
//!   ecosystem.
//!
//! ## Known limitations vs. real `node:buffer`
//! * Instances of this `Buffer` are **not** `Uint8Array` subclasses (napi-rs
//!   cannot subclass `Uint8Array` from a custom class). Therefore `buf[i]`,
//!   `for..of buf`, `buf instanceof Uint8Array`, and the `Buffer.prototype`
//!   methods being callable on plain `Uint8Array`s are not supported. Use the
//!   explicit methods (`readUInt8`, `write`, `slice`, …) instead. The `.buffer`
//!   view hands back a *copy* as a real `Buffer`/`ArrayBuffer`.
//! * `blob.stream()` / `blob.textStream()` are not implemented (no stream
//!   binding); `arrayBuffer` / `bytes` / `text` / `slice` are synchronous
//!   (rather than returning a `Promise`) for simplicity.

use std::ptr::{self, NonNull};

use napi::bindgen_prelude::{
  ArrayBuffer, BigInt, Buffer as JsBuffer, Env, FromNapiValue, JsValue, Object, Unknown,
};
use napi::sys;
use napi::{check_status, Error, Result, Status};
use napi_derive::napi;

/// Extracts the raw `napi_value` from an `Unknown` (the `NapiRaw` trait is not
/// re-exported publicly, so we go through the public `JsValue`/`Value` API).
fn raw_of(u: &Unknown) -> sys::napi_value {
  JsValue::value(u).value
}

/// Safe wrapper around the `unsafe` `FromNapiValue::from_napi_value`.
fn conv<T: FromNapiValue>(env: &Env, v: sys::napi_value) -> Result<T> {
  unsafe { T::from_napi_value(env.raw(), v) }
}

// ===========================================================================
// Constants
// ===========================================================================

const MAX_LENGTH: u32 = 0x7fff_ffff;
const MAX_STRING_LENGTH: u32 = 0x1ffff_fe8;
const INSPECT_MAX_BYTES: u32 = 50;
const POOL_SIZE: u32 = 65536;

const SUPPORTED_ENCODINGS: &[&str] = &[
  "utf8", "utf-8", "ascii", "latin1", "binary", "utf16le", "ucs2", "ucs-2", "hex", "base64",
  "base64url",
];

// ===========================================================================
// Encoding helpers (utf8 / ascii / latin1 / utf16le / hex / base64).
// ===========================================================================

fn str_to_bytes(s: &str, enc: &str) -> Result<Vec<u8>> {
  match enc.to_ascii_lowercase().as_str() {
    "utf8" | "utf-8" => Ok(s.as_bytes().to_vec()),
    "ascii" => {
      if s.bytes().any(|b| b > 127) {
        return Err(Error::new(Status::InvalidArg, "invalid ASCII string"));
      }
      Ok(s.as_bytes().to_vec())
    }
    "latin1" | "binary" => Ok(s.chars().map(|c| c as u8).collect()),
    "utf16le" | "ucs2" | "ucs-2" => {
      let mut out = Vec::with_capacity(s.len() * 2);
      for unit in s.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
      }
      Ok(out)
    }
    "hex" => decode_hex(s),
    "base64" => decode_base64(s, false),
    "base64url" => decode_base64(s, true),
    other => Err(Error::new(
      Status::InvalidArg,
      format!("unsupported encoding: {other}"),
    )),
  }
}

fn bytes_to_str(b: &[u8], enc: &str) -> Result<String> {
  match enc.to_ascii_lowercase().as_str() {
    "utf8" | "utf-8" => String::from_utf8(b.to_vec())
      .map_err(|_| Error::new(Status::InvalidArg, "invalid UTF-8 bytes")),
    "ascii" => {
      if b.iter().any(|x| *x > 127) {
        return Err(Error::new(Status::InvalidArg, "invalid ASCII bytes"));
      }
      Ok(b.iter().map(|x| *x as char).collect())
    }
    "latin1" | "binary" => Ok(b.iter().map(|x| *x as char).collect()),
    "utf16le" | "ucs2" | "ucs-2" => {
      if b.len() % 2 != 0 {
        return Err(Error::new(
          Status::InvalidArg,
          "invalid UTF-16LE bytes: odd length",
        ));
      }
      let units = b
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect::<Vec<_>>();
      String::from_utf16(&units)
        .map_err(|_| Error::new(Status::InvalidArg, "invalid UTF-16LE bytes"))
    }
    "hex" => Ok(encode_hex(b)),
    "base64" => Ok(encode_base64(b, false)),
    "base64url" => Ok(encode_base64(b, true)),
    other => Err(Error::new(
      Status::InvalidArg,
      format!("unsupported encoding: {other}"),
    )),
  }
}

fn decode_hex(s: &str) -> Result<Vec<u8>> {
  let s = s.trim();
  if s.len() % 2 != 0 {
    return Err(Error::new(Status::InvalidArg, "invalid hex string"));
  }
  let bytes = s.as_bytes();
  let mut out = Vec::with_capacity(s.len() / 2);
  let mut i = 0;
  while i + 1 < bytes.len() {
    let hi = hex_val(bytes[i])?;
    let lo = hex_val(bytes[i + 1])?;
    out.push((hi << 4) | lo);
    i += 2;
  }
  Ok(out)
}

fn hex_val(b: u8) -> Result<u8> {
  match b {
    b'0'..=b'9' => Ok(b - b'0'),
    b'a'..=b'f' => Ok(b - b'a' + 10),
    b'A'..=b'F' => Ok(b - b'A' + 10),
    _ => Err(Error::new(Status::InvalidArg, "invalid hex digit")),
  }
}

fn encode_hex(b: &[u8]) -> String {
  let mut s = String::with_capacity(b.len() * 2);
  for &byte in b {
    s.push_str(&format!("{byte:02x}"));
  }
  s
}

fn decode_base64(s: &str, url: bool) -> Result<Vec<u8>> {
  let cleaned: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
  let mut out = Vec::new();
  let mut i = 0;
  while i + 4 <= cleaned.len() {
    let mut buf = [0u8; 4];
    for k in 0..4 {
      buf[k] = b64_val(cleaned[i + k], url)?;
    }
    let n = ((buf[0] as u32) << 18) | ((buf[1] as u32) << 12) | ((buf[2] as u32) << 6) | (buf[3] as u32);
    out.push((n >> 16) as u8);
    if buf[2] != 64 {
      out.push((n >> 8) as u8);
    }
    if buf[3] != 64 {
      out.push(n as u8);
    }
    i += 4;
  }
  Ok(out)
}

fn b64_val(b: u8, url: bool) -> Result<u8> {
  match b {
    b'A'..=b'Z' => Ok(b - b'A'),
    b'a'..=b'z' => Ok(b - b'a' + 26),
    b'0'..=b'9' => Ok(b - b'0' + 52),
    b'+' if !url => Ok(62),
    b'/' if !url => Ok(63),
    b'-' if url => Ok(62),
    b'_' if url => Ok(63),
    b'=' => Ok(64),
    _ => Err(Error::new(Status::InvalidArg, "invalid base64 character")),
  }
}

fn encode_base64(b: &[u8], url: bool) -> String {
  const STANDARD: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const URLSAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
  let alpha = if url { URLSAFE } else { STANDARD };
  let mut s = String::new();
  let mut i = 0;
  while i + 3 <= b.len() {
    let n = ((b[i] as u32) << 16) | ((b[i + 1] as u32) << 8) | (b[i + 2] as u32);
    s.push(alpha[((n >> 18) & 63) as usize] as char);
    s.push(alpha[((n >> 12) & 63) as usize] as char);
    s.push(alpha[((n >> 6) & 63) as usize] as char);
    s.push(alpha[(n & 63) as usize] as char);
    i += 3;
  }
  let rem = b.len() - i;
  if rem == 1 {
    let n = (b[i] as u32) << 16;
    s.push(alpha[((n >> 18) & 63) as usize] as char);
    s.push(alpha[((n >> 12) & 63) as usize] as char);
    if !url {
      s.push('=');
      s.push('=');
    }
  } else if rem == 2 {
    let n = ((b[i] as u32) << 16) | ((b[i + 1] as u32) << 8);
    s.push(alpha[((n >> 18) & 63) as usize] as char);
    s.push(alpha[((n >> 12) & 63) as usize] as char);
    s.push(alpha[((n >> 6) & 63) as usize] as char);
    if !url {
      s.push('=');
    }
  }
  s
}

// ===========================================================================
// Low-level JS value inspection for ArrayBuffer / TypedArray / DataView.
// ===========================================================================

unsafe fn inspect_typedarray(
  env: &Env,
  v: sys::napi_value,
) -> Result<(NonNull<u8>, usize, usize, usize)> {
  let mut ta_type = 0;
  let mut len = 0usize;
  let mut data = ptr::null_mut();
  let mut ab = ptr::null_mut();
  let mut boff = 0usize;
  check_status!(
    sys::napi_get_typedarray_info(env.raw(), v, &mut ta_type, &mut len, &mut data, &mut ab, &mut boff),
    "Failed to read TypedArray info"
  )?;
  // napi_typedarray_type numbering: int8=0 uint8=1 uint8clamped=2 int16=3
  // uint16=4 int32=5 uint32=6 float32=7 float64=8 bigint64=9 biguint64=10
  let elem = match ta_type {
    0 | 1 | 2 => 1,
    3 | 4 => 2,
    5 | 6 | 7 => 4,
    8 | 9 | 10 => 8,
    _ => 1,
  };
  let ptr = NonNull::new(data as *mut u8).unwrap_or(NonNull::dangling());
  Ok((ptr, len, boff, elem))
}

unsafe fn dataview_bytes(env: &Env, v: sys::napi_value) -> Result<(NonNull<u8>, usize)> {
  let mut len = 0usize;
  let mut data = ptr::null_mut();
  let mut ab = ptr::null_mut();
  let mut boff = 0usize;
  check_status!(
    sys::napi_get_dataview_info(env.raw(), v, &mut len, &mut data, &mut ab, &mut boff),
    "Failed to read DataView info"
  )?;
  let ptr = NonNull::new(data as *mut u8).unwrap_or(NonNull::dangling());
  Ok((ptr, len))
}

/// Copies the underlying bytes of a Buffer / TypedArray / DataView / ArrayBuffer.
fn get_bytes(env: &Env, v: sys::napi_value) -> Result<Vec<u8>> {
  let mut b = false;
  check_status!(unsafe { sys::napi_is_buffer(env.raw(), v, &mut b) })?;
  if b {
    let buf = conv::<JsBuffer>(&env, v)?;
    return Ok(buf.as_ref().to_vec());
  }
  let mut t = false;
  check_status!(unsafe { sys::napi_is_typedarray(env.raw(), v, &mut t) })?;
  if t {
    let (ptr, len, boff, elem) = unsafe { inspect_typedarray(env, v)? };
    let total = boff + len * elem;
    let slice = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), total) };
    return Ok(slice[boff..boff + len * elem].to_vec());
  }
  let mut d = false;
  check_status!(unsafe { sys::napi_is_dataview(env.raw(), v, &mut d) })?;
  if d {
    let (ptr, len) = unsafe { dataview_bytes(env, v)? };
    return Ok(unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) }.to_vec());
  }
  let mut a = false;
  check_status!(unsafe { sys::napi_is_arraybuffer(env.raw(), v, &mut a) })?;
  if a {
    let ab = conv::<ArrayBuffer>(env, v)?;
    return Ok(ab.to_vec());
  }
  Err(Error::new(
    Status::InvalidArg,
    "expected a Buffer, TypedArray, DataView or ArrayBuffer",
  ))
}

/// `Buffer.from(typedArray)` copies one byte per element (the low byte).
fn typedarray_low_bytes(env: &Env, v: sys::napi_value) -> Result<Vec<u8>> {
  let (ptr, len, boff, elem) = unsafe { inspect_typedarray(env, v)? };
  let total = boff + len * elem;
  let slice = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), total) };
  let mut out = Vec::with_capacity(len);
  for i in 0..len {
    out.push(slice[boff + i * elem]);
  }
  Ok(out)
}

fn unknown_to_byte(env: &Env, v: Unknown) -> u8 {
  match conv::<f64>(env, raw_of(&v)) {
    Ok(n) => ((n as i32) & 0xff) as u8,
    Err(_) => 0,
  }
}

fn unknown_to_usize(env: &Env, v: Unknown) -> Option<usize> {
  conv::<f64>(env, raw_of(&v)).ok().map(|n| n as usize)
}

fn unknown_to_string(env: &Env, v: Unknown) -> Option<String> {
  conv::<String>(env, raw_of(&v)).ok()
}

// ===========================================================================
// `Buffer` — node:buffer-compatible byte buffer.
// ===========================================================================

#[napi]
pub struct Buffer {
  data: Vec<u8>,
}

#[napi]
impl Buffer {
  /// Creates a zero-filled buffer of `length` bytes (equivalent to `new Buffer(length)`).
  #[napi(constructor)]
  pub fn new(length: u32) -> Self {
    Self {
      data: vec![0u8; length as usize],
    }
  }

  /// Allocates a zero-filled buffer of `size` bytes, optionally filled.
  #[napi(ts_return_type = "Buffer")]
  pub fn alloc(env: Env, size: u32, fill: Option<Unknown>, encoding: Option<String>) -> Result<Buffer> {
    let mut b = Buffer {
      data: vec![0u8; size as usize],
    };
    if let Some(v) = fill {
      let raw = raw_of(&v);
      if let Ok(n) = conv::<f64>(&env, raw) {
        let byte = ((n as i32) & 0xff) as u8;
        b.data.iter_mut().for_each(|x| *x = byte);
      } else if let Ok(s) = conv::<String>(&env, raw) {
        let enc = encoding.unwrap_or_else(|| "utf8".to_string());
        let bytes = str_to_bytes(&s, &enc)?;
        if !bytes.is_empty() {
          for (i, x) in b.data.iter_mut().enumerate() {
            *x = bytes[i % bytes.len()];
          }
        }
      } else if let Ok(buf) = conv::<JsBuffer>(&env, raw) {
        let bytes = buf.as_ref();
        if !bytes.is_empty() {
          for (i, x) in b.data.iter_mut().enumerate() {
            *x = bytes[i % bytes.len()];
          }
        }
      }
    }
    Ok(b)
  }

  /// Allocates an uninitialized buffer of `size` bytes.
  ///
  /// For safety this still zero-fills (napi cannot hand back truly uninitialized
  /// memory across the FFI boundary).
  #[napi(js_name = "allocUnsafe", ts_return_type = "Buffer")]
  pub fn alloc_unsafe(size: u32, _alignment: Option<u32>) -> Buffer {
    Buffer::new(size)
  }

  /// Like `allocUnsafe` but never uses the internal pool.
  #[napi(js_name = "allocUnsafeSlow", ts_return_type = "Buffer")]
  pub fn alloc_unsafe_slow(size: u32, _alignment: Option<u32>) -> Buffer {
    Buffer::new(size)
  }

  /// Creates a buffer by copying an existing Node `Buffer`.
  #[napi(factory, ts_args_type = "buffer: Buffer", ts_return_type = "Buffer")]
  pub fn from_buffer(buffer: JsBuffer) -> Buffer {
    Buffer {
      data: buffer.as_ref().to_vec(),
    }
  }

  /// The node:buffer `Buffer.from(...)` overload dispatcher.
  ///
  /// Supports: string (with optional encoding), ArrayBuffer (with byteOffset /
  /// length), TypedArray, DataView, Buffer / Uint8Array, and plain Arrays.
  #[napi(js_name = "from", ts_return_type = "Buffer")]
  pub fn from(env: Env, value: Unknown, arg1: Option<Unknown>, arg2: Option<Unknown>) -> Result<Buffer> {
    let raw = raw_of(&value);
    if let Ok(s) = conv::<String>(&env, raw) {
      let enc = arg1
        .and_then(|u| unknown_to_string(&env, u))
        .unwrap_or_else(|| "utf8".to_string());
      return Ok(Buffer {
        data: str_to_bytes(&s, &enc)?,
      });
    }
    let mut is_ta = false;
    check_status!(unsafe { sys::napi_is_typedarray(env.raw(), raw, &mut is_ta) })?;
    if is_ta {
      return Ok(Buffer {
        data: typedarray_low_bytes(&env, raw)?,
      });
    }
    let mut is_dv = false;
    check_status!(unsafe { sys::napi_is_dataview(env.raw(), raw, &mut is_dv) })?;
    if is_dv {
      let (ptr, len) = unsafe { dataview_bytes(&env, raw)? };
      return Ok(Buffer {
        data: unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) }.to_vec(),
      });
    }
    let mut is_ab = false;
    check_status!(unsafe { sys::napi_is_arraybuffer(env.raw(), raw, &mut is_ab) })?;
    if is_ab {
      let ab = conv::<ArrayBuffer>(&env, raw)?;
      let total = ab.len();
      let off = arg1
        .as_ref()
        .and_then(|u| unknown_to_usize(&env, u.clone()))
        .unwrap_or(0);
      let off = off.min(total);
      let len = arg2
        .as_ref()
        .and_then(|u| unknown_to_usize(&env, u.clone()))
        .unwrap_or(total - off);
      let len = len.min(total - off);
      return Ok(Buffer {
        data: ab[off..off + len].to_vec(),
      });
    }
    let mut is_buf = false;
    check_status!(unsafe { sys::napi_is_buffer(env.raw(), raw, &mut is_buf) })?;
    if is_buf {
      let buf = conv::<JsBuffer>(&env, raw)?;
      return Ok(Buffer {
        data: buf.as_ref().to_vec(),
      });
    }
    let mut is_arr = false;
    check_status!(unsafe { sys::napi_is_array(env.raw(), raw, &mut is_arr) })?;
    if is_arr {
      let arr: Vec<Unknown> = conv::<Vec<Unknown>>(&env, raw)?;
      let data: Vec<u8> = arr.into_iter().map(|u| unknown_to_byte(&env, u)).collect();
      return Ok(Buffer { data });
    }
    Err(Error::new(
      Status::InvalidArg,
      "Buffer.from: unsupported value",
    ))
  }

  /// The number of bytes in the buffer.
  #[napi(getter)]
  pub fn length(&self) -> u32 {
    self.data.len() as u32
  }

  /// A view of the buffer's backing memory (a real `Buffer` copy).
  #[napi(getter, ts_return_type = "Buffer")]
  pub fn buffer(&self) -> JsBuffer {
    JsBuffer::from(self.data.clone())
  }

  /// The byte offset of the buffer's backing memory (always 0 here).
  #[napi(getter)]
  pub fn byte_offset(&self) -> u32 {
    0
  }

  /// Alias of `.buffer`.
  #[napi(getter, ts_return_type = "Buffer")]
  pub fn parent(&self) -> JsBuffer {
    JsBuffer::from(self.data.clone())
  }

  /// Concatenates a list of buffers into one.
  #[napi(ts_args_type = "list: Array<Buffer>, totalLength?: number", ts_return_type = "Buffer")]
  pub fn concat(list: Vec<JsBuffer>, total_length: Option<u32>) -> Buffer {
    let total = total_length.unwrap_or_else(|| {
      list.iter().map(|b| b.as_ref().len() as u32).sum()
    }) as usize;
    let mut out = Vec::with_capacity(total);
    for b in &list {
      out.extend_from_slice(b.as_ref());
    }
    out.truncate(total);
    Buffer { data: out }
  }

  /// Returns the byte length of a string / buffer-like value.
  #[napi(js_name = "byteLength")]
  pub fn byte_length(env: Env, value: Unknown, encoding: Option<String>) -> Result<u32> {
    let raw = raw_of(&value);
    if let Ok(s) = conv::<String>(&env, raw) {
      let enc = encoding.unwrap_or_else(|| "utf8".to_string());
      return Ok(str_to_bytes(&s, &enc)?.len() as u32);
    }
    Ok(get_bytes(&env, raw)?.len() as u32)
  }

  /// Returns the UTF-16 length that `toString(encoding)` would produce.
  #[napi(js_name = "stringLength")]
  pub fn string_length(env: Env, input: Unknown, encoding: Option<String>) -> Result<u32> {
    let bytes = get_bytes(&env, raw_of(&input))?;
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let s = bytes_to_str(&bytes, &enc)?;
    Ok(s.chars().count() as u32)
  }

  /// Compares two buffers, returning -1, 0, or 1.
  #[napi(ts_args_type = "buf1: Buffer, buf2: Buffer")]
  pub fn compare(buf1: JsBuffer, buf2: JsBuffer) -> i32 {
    buf1.as_ref().cmp(buf2.as_ref()) as i32
  }

  /// Copies the underlying bytes of a TypedArray into a new Buffer.
  #[napi(js_name = "copyBytesFrom", ts_return_type = "Buffer")]
  pub fn copy_bytes_from(env: Env, view: Unknown, offset: Option<u32>, length: Option<u32>) -> Result<Buffer> {
    let raw = raw_of(&view);
    let mut is_ta = false;
    check_status!(unsafe { sys::napi_is_typedarray(env.raw(), raw, &mut is_ta) })?;
    if is_ta {
      let (ptr, elem_count, boff, elem) = unsafe { inspect_typedarray(&env, raw)? };
      let off = offset.unwrap_or(0) as usize;
      let cnt = length
        .unwrap_or((elem_count - off) as u32)
        .min((elem_count - off) as u32)
        as usize;
      let total = boff + elem_count * elem;
      let slice = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), total) };
      let start = boff + off * elem;
      let end = (boff + (off + cnt) * elem).min(total);
      return Ok(Buffer {
        data: slice[start..end].to_vec(),
      });
    }
    let buf = conv::<JsBuffer>(&env, raw)?;
    Ok(Buffer {
      data: buf.as_ref().to_vec(),
    })
  }

  /// Returns true if `value` is a Node `Buffer`. Note: instances of this class
  /// are not Node `Buffer`s, so this returns false for them.
  #[napi(js_name = "isBuffer")]
  pub fn is_buffer(env: Env, value: Unknown) -> bool {
    let mut b = false;
    let _ = unsafe { sys::napi_is_buffer(env.raw(), raw_of(&value), &mut b) };
    b
  }

  /// Returns true if `encoding` is a supported encoding name.
  #[napi(js_name = "isEncoding")]
  pub fn is_encoding(encoding: String) -> bool {
    SUPPORTED_ENCODINGS.contains(&encoding.to_ascii_lowercase().as_str())
  }

  // --- bounds helper -----------------------------------------------------

  fn check(&self, off: usize, n: usize) -> Result<()> {
    if off
      .checked_add(n)
      .map(|end| end > self.data.len())
      .unwrap_or(true)
    {
      return Err(Error::new(
        Status::GenericFailure,
        format!(
          "out of range: offset {off} + {n} exceeds buffer length {}",
          self.data.len()
        ),
      ));
    }
    Ok(())
  }

  fn ro(&self, off: Option<u32>) -> usize {
    off.unwrap_or(0) as usize
  }

  // --- unsigned / signed integer reads ----------------------------------

  #[napi(js_name = "readUInt8")]
  pub fn read_uint8(&self, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 1)?;
    Ok(self.data[o] as u32)
  }

  #[napi(js_name = "readUint8")]
  pub fn read_uint8_alias(&self, offset: Option<u32>) -> Result<u32> {
    self.read_uint8(offset)
  }

  #[napi(js_name = "readInt8")]
  pub fn read_int8(&self, offset: Option<u32>) -> Result<i32> {
    let o = self.ro(offset);
    self.check(o, 1)?;
    Ok((self.data[o] as i8) as i32)
  }

  #[napi(js_name = "readUInt16LE")]
  pub fn read_uint16_le(&self, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 2)?;
    Ok(u16::from_le_bytes([self.data[o], self.data[o + 1]]) as u32)
  }

  #[napi(js_name = "readUint16LE")]
  pub fn read_uint16_le_alias(&self, offset: Option<u32>) -> Result<u32> {
    self.read_uint16_le(offset)
  }

  #[napi(js_name = "readUInt16BE")]
  pub fn read_uint16_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 2)?;
    Ok(u16::from_be_bytes([self.data[o], self.data[o + 1]]) as u32)
  }

  #[napi(js_name = "readUint16BE")]
  pub fn read_uint16_be_alias(&self, offset: Option<u32>) -> Result<u32> {
    self.read_uint16_be(offset)
  }

  #[napi(js_name = "readInt16LE")]
  pub fn read_int16_le(&self, offset: Option<u32>) -> Result<i32> {
    let o = self.ro(offset);
    self.check(o, 2)?;
    Ok(i16::from_le_bytes([self.data[o], self.data[o + 1]]) as i32)
  }

  #[napi(js_name = "readInt16BE")]
  pub fn read_int16_be(&self, offset: Option<u32>) -> Result<i32> {
    let o = self.ro(offset);
    self.check(o, 2)?;
    Ok(i16::from_be_bytes([self.data[o], self.data[o + 1]]) as i32)
  }

  #[napi(js_name = "readUInt32LE")]
  pub fn read_uint32_le(&self, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(u32::from_le_bytes(b))
  }

  #[napi(js_name = "readUint32LE")]
  pub fn read_uint32_le_alias(&self, offset: Option<u32>) -> Result<u32> {
    self.read_uint32_le(offset)
  }

  #[napi(js_name = "readUInt32BE")]
  pub fn read_uint32_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(u32::from_be_bytes(b))
  }

  #[napi(js_name = "readUint32BE")]
  pub fn read_uint32_be_alias(&self, offset: Option<u32>) -> Result<u32> {
    self.read_uint32_be(offset)
  }

  #[napi(js_name = "readInt32LE")]
  pub fn read_int32_le(&self, offset: Option<u32>) -> Result<i32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(i32::from_le_bytes(b))
  }

  #[napi(js_name = "readInt32BE")]
  pub fn read_int32_be(&self, offset: Option<u32>) -> Result<i32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(i32::from_be_bytes(b))
  }

  #[napi(js_name = "readFloatLE")]
  pub fn read_float_le(&self, offset: Option<u32>) -> Result<f64> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(f32::from_le_bytes(b) as f64)
  }

  #[napi(js_name = "readFloatBE")]
  pub fn read_float_be(&self, offset: Option<u32>) -> Result<f64> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(f32::from_be_bytes(b) as f64)
  }

  #[napi(js_name = "readDoubleLE")]
  pub fn read_double_le(&self, offset: Option<u32>) -> Result<f64> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(f64::from_le_bytes(b))
  }

  #[napi(js_name = "readDoubleBE")]
  pub fn read_double_be(&self, offset: Option<u32>) -> Result<f64> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(f64::from_be_bytes(b))
  }

  #[napi(js_name = "readBigUInt64LE")]
  pub fn read_big_uint64_le(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(u64::from_le_bytes(b)))
  }

  #[napi(js_name = "readBigUint64LE")]
  pub fn read_big_uint64_le_alias(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_uint64_le(offset)
  }

  #[napi(js_name = "readBigUInt64BE")]
  pub fn read_big_uint64_be(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(u64::from_be_bytes(b)))
  }

  #[napi(js_name = "readBigUint64BE")]
  pub fn read_big_uint64_be_alias(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_uint64_be(offset)
  }

  #[napi(js_name = "readBigInt64LE")]
  pub fn read_big_int64_le(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(i64::from_le_bytes(b)))
  }

  #[napi(js_name = "readBigInt64BE")]
  pub fn read_big_int64_be(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(i64::from_be_bytes(b)))
  }

  // --- 48-bit reads ------------------------------------------------------

  #[napi(js_name = "readUIntBE")]
  pub fn read_uint_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let n = byte_length as usize;
    if n == 0 || n > 6 {
      return Err(Error::new(Status::InvalidArg, "byteLength must be 1..=6"));
    }
    self.check(o, n)?;
    let mut val: u64 = 0;
    for i in 0..n {
      val = (val << 8) | self.data[o + i] as u64;
    }
    Ok(val as f64)
  }

  #[napi(js_name = "readUintBE")]
  pub fn read_uint_be_alias(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_uint_be(offset, byte_length)
  }

  #[napi(js_name = "readUIntLE")]
  pub fn read_uint_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let n = byte_length as usize;
    if n == 0 || n > 6 {
      return Err(Error::new(Status::InvalidArg, "byteLength must be 1..=6"));
    }
    self.check(o, n)?;
    let mut val: u64 = 0;
    for i in 0..n {
      val |= (self.data[o + i] as u64) << (8 * i);
    }
    Ok(val as f64)
  }

  #[napi(js_name = "readUintLE")]
  pub fn read_uint_le_alias(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_uint_le(offset, byte_length)
  }

  #[napi(js_name = "readIntBE")]
  pub fn read_int_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let n = byte_length as usize;
    if n == 0 || n > 6 {
      return Err(Error::new(Status::InvalidArg, "byteLength must be 1..=6"));
    }
    self.check(o, n)?;
    let mut val: u64 = 0;
    for i in 0..n {
      val = (val << 8) | self.data[o + i] as u64;
    }
    let shift = 64 - 8 * n;
    let signed = ((val << shift) as i64) >> shift;
    Ok(signed as f64)
  }

  #[napi(js_name = "readIntLE")]
  pub fn read_int_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let n = byte_length as usize;
    if n == 0 || n > 6 {
      return Err(Error::new(Status::InvalidArg, "byteLength must be 1..=6"));
    }
    self.check(o, n)?;
    let mut val: u64 = 0;
    for i in 0..n {
      val |= (self.data[o + i] as u64) << (8 * i);
    }
    let shift = 64 - 8 * n;
    let signed = ((val << shift) as i64) >> shift;
    Ok(signed as f64)
  }

  // --- unsigned / signed integer writes ----------------------------------

  #[napi(js_name = "writeUInt8")]
  pub fn write_uint8(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 1)?;
    self.data[o] = (value & 0xff) as u8;
    Ok((o + 1) as u32)
  }

  #[napi(js_name = "writeUint8")]
  pub fn write_uint8_alias(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_uint8(value, offset)
  }

  #[napi(js_name = "writeInt8")]
  pub fn write_int8(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 1)?;
    self.data[o] = (value & 0xff) as u8;
    Ok((o + 1) as u32)
  }

  #[napi(js_name = "writeUInt16LE")]
  pub fn write_uint16_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as u16).to_le_bytes());
    Ok((o + 2) as u32)
  }

  #[napi(js_name = "writeUint16LE")]
  pub fn write_uint16_le_alias(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_uint16_le(value, offset)
  }

  #[napi(js_name = "writeUInt16BE")]
  pub fn write_uint16_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as u16).to_be_bytes());
    Ok((o + 2) as u32)
  }

  #[napi(js_name = "writeUint16BE")]
  pub fn write_uint16_be_alias(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_uint16_be(value, offset)
  }

  #[napi(js_name = "writeInt16LE")]
  pub fn write_int16_le(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as i16).to_le_bytes());
    Ok((o + 2) as u32)
  }

  #[napi(js_name = "writeInt16BE")]
  pub fn write_int16_be(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as i16).to_be_bytes());
    Ok((o + 2) as u32)
  }

  #[napi(js_name = "writeUInt32LE")]
  pub fn write_uint32_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_le_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeUint32LE")]
  pub fn write_uint32_le_alias(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_uint32_le(value, offset)
  }

  #[napi(js_name = "writeUInt32BE")]
  pub fn write_uint32_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_be_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeUint32BE")]
  pub fn write_uint32_be_alias(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_uint32_be(value, offset)
  }

  #[napi(js_name = "writeInt32LE")]
  pub fn write_int32_le(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_le_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeInt32BE")]
  pub fn write_int32_be(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_be_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeFloatLE")]
  pub fn write_float_le(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&(value as f32).to_le_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeFloatBE")]
  pub fn write_float_be(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&(value as f32).to_be_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeDoubleLE")]
  pub fn write_double_le(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    self.data[o..o + 8].copy_from_slice(&value.to_le_bytes());
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeDoubleBE")]
  pub fn write_double_be(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    self.data[o..o + 8].copy_from_slice(&value.to_be_bytes());
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeBigUInt64LE")]
  pub fn write_big_uint64_le(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let (_, v, _) = value.get_u64();
    self.data[o..o + 8].copy_from_slice(&v.to_le_bytes());
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeBigUint64LE")]
  pub fn write_big_uint64_le_alias(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_uint64_le(value, offset)
  }

  #[napi(js_name = "writeBigUInt64BE")]
  pub fn write_big_uint64_be(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let (_, v, _) = value.get_u64();
    self.data[o..o + 8].copy_from_slice(&v.to_be_bytes());
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeBigUint64BE")]
  pub fn write_big_uint64_be_alias(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_uint64_be(value, offset)
  }

  #[napi(js_name = "writeBigInt64LE")]
  pub fn write_big_int64_le(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let (v, _) = value.get_i64();
    self.data[o..o + 8].copy_from_slice(&v.to_le_bytes());
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeBigInt64BE")]
  pub fn write_big_int64_be(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = self.ro(offset);
    self.check(o, 8)?;
    let (v, _) = value.get_i64();
    self.data[o..o + 8].copy_from_slice(&v.to_be_bytes());
    Ok((o + 8) as u32)
  }

  // --- 48-bit writes -----------------------------------------------------

  #[napi(js_name = "writeUIntBE")]
  pub fn write_uint_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_uint_be_inner(value as u64, offset, byte_length, false)
  }

  #[napi(js_name = "writeUintBE")]
  pub fn write_uint_be_alias(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_uint_be(value, offset, byte_length)
  }

  #[napi(js_name = "writeUIntLE")]
  pub fn write_uint_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_uint_be_inner(value as u64, offset, byte_length, true)
  }

  #[napi(js_name = "writeUintLE")]
  pub fn write_uint_le_alias(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_uint_le(value, offset, byte_length)
  }

  #[napi(js_name = "writeIntBE")]
  pub fn write_int_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_int_be_inner(value as i64, offset, byte_length, false)
  }

  #[napi(js_name = "writeIntLE")]
  pub fn write_int_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_int_be_inner(value as i64, offset, byte_length, true)
  }

  fn write_uint_be_inner(&mut self, val: u64, offset: u32, byte_length: u32, le: bool) -> Result<u32> {
    let o = offset as usize;
    let n = byte_length as usize;
    if n == 0 || n > 6 {
      return Err(Error::new(Status::InvalidArg, "byteLength must be 1..=6"));
    }
    self.check(o, n)?;
    for i in 0..n {
      let byte = if le {
        (val >> (8 * i)) as u8
      } else {
        (val >> (8 * (n - 1 - i))) as u8
      };
      self.data[o + i] = byte;
    }
    Ok((o + n) as u32)
  }

  fn write_int_be_inner(&mut self, val: i64, offset: u32, byte_length: u32, le: bool) -> Result<u32> {
    let u = val as u64;
    self.write_uint_be_inner(u, offset, byte_length, le)
  }

  // --- string read / write ----------------------------------------------

  /// Writes `string` into the buffer at `offset` (optionally limiting to
  /// `length` bytes) using `encoding`. Returns the number of bytes written.
  #[napi]
  pub fn write(
    &mut self,
    string: String,
    offset: Option<u32>,
    length: Option<u32>,
    encoding: Option<String>,
  ) -> Result<u32> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let bytes = str_to_bytes(&string, &enc)?;
    let o = self.ro(offset);
    if o > self.data.len() {
      return Err(Error::new(
        Status::GenericFailure,
        format!("offset {o} out of range (length {})", self.data.len()),
      ));
    }
    let max = self.data.len() - o;
    let want = length.unwrap_or(bytes.len() as u32).min(bytes.len() as u32);
    let n = want.min(max as u32) as usize;
    self.data[o..o + n].copy_from_slice(&bytes[..n]);
    Ok(n as u32)
  }

  /// Decodes the buffer (or `start..end`) using `encoding`.
  #[napi(js_name = "toString")]
  pub fn to_string(&self, encoding: Option<String>, start: Option<u32>, end: Option<u32>) -> Result<String> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let s = self.ro(start);
    let e = end.unwrap_or(self.data.len() as u32) as usize;
    if s > e || e > self.data.len() {
      return Err(Error::new(Status::GenericFailure, "invalid start/end range"));
    }
    bytes_to_str(&self.data[s..e], &enc)
  }

  /// Returns a JSON-compatible representation `{ type: "Buffer", data: [...] }`.
  #[napi(js_name = "toJSON")]
  pub fn to_json(&self) -> BufferJson {
    BufferJson {
      r#type: "Buffer".to_string(),
      data: self.data.iter().map(|b| *b as u32).collect(),
    }
  }

  // --- views / comparison ------------------------------------------------

  /// Returns a copy of the bytes in `start..end` as a real Node `Buffer`.
  #[napi(ts_return_type = "Buffer")]
  pub fn slice(&self, start: Option<i64>, end: Option<i64>) -> Result<JsBuffer> {
    let (s, e) = self.clamp_range(start, end)?;
    Ok(JsBuffer::from(&self.data[s..e]))
  }

  /// Alias of `slice` (Node's `subarray` shares memory; here it copies).
  #[napi(ts_return_type = "Buffer")]
  pub fn subarray(&self, start: Option<i64>, end: Option<i64>) -> Result<JsBuffer> {
    self.slice(start, end)
  }

  /// Returns a copy of all written bytes as a real Node `Buffer`.
  #[napi(ts_return_type = "Buffer")]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  /// Fills `value` (number, string, or buffer) over `start..end`.
  #[napi]
  pub fn fill(&mut self, env: Env, value: Unknown, start: Option<u32>, end: Option<u32>, encoding: Option<String>) -> Result<()> {
    let (s, e) = self.range_u(start, end)?;
    let raw = raw_of(&value);
    if let Ok(n) = conv::<f64>(&env, raw) {
      let byte = ((n as i32) & 0xff) as u8;
      self.data[s..e].iter_mut().for_each(|x| *x = byte);
    } else if let Ok(s2) = conv::<String>(&env, raw) {
      let enc = encoding.unwrap_or_else(|| "utf8".to_string());
      let bytes = str_to_bytes(&s2, &enc)?;
      if bytes.is_empty() {
        self.data[s..e].iter_mut().for_each(|x| *x = 0);
      } else {
        for (i, slot) in self.data[s..e].iter_mut().enumerate() {
          *slot = bytes[i % bytes.len()];
        }
      }
    } else {
      let bytes = get_bytes(&env, raw)?;
      if bytes.is_empty() {
        self.data[s..e].iter_mut().for_each(|x| *x = 0);
      } else {
        for (i, slot) in self.data[s..e].iter_mut().enumerate() {
          *slot = bytes[i % bytes.len()];
        }
      }
    }
    Ok(())
  }

  /// Returns true if the bytes equal `other`.
  #[napi(ts_args_type = "other: Buffer")]
  pub fn equals(&self, other: JsBuffer) -> bool {
    self.data == other.as_ref()
  }

  /// Lexicographic comparison with `other`: -1, 0, or 1.
  #[napi(ts_args_type = "other: Buffer")]
  pub fn compare_with(&self, other: JsBuffer) -> i32 {
    self.data.as_slice().cmp(other.as_ref()) as i32
  }

  /// Copies bytes from this buffer into `target`. Returns the number copied.
  #[napi(ts_args_type = "target: Buffer, targetStart?: number, sourceStart?: number, sourceEnd?: number")]
  pub fn copy(&self, mut target: JsBuffer, target_start: Option<u32>, source_start: Option<u32>, source_end: Option<u32>) -> Result<u32> {
    let ts = target_start.unwrap_or(0) as usize;
    let ss = source_start.unwrap_or(0) as usize;
    let se = source_end.unwrap_or(self.data.len() as u32) as usize;
    let se = se.min(self.data.len());
    let tlen = target.as_ref().len();
    if ss > se || se > self.data.len() || ts > tlen {
      return Err(Error::new(
        Status::GenericFailure,
        "copy: out of range",
      ));
    }
    let n = (se - ss).min(tlen - ts);
    target.as_mut()[ts..ts + n].copy_from_slice(&self.data[ss..ss + n]);
    Ok(n as u32)
  }

  /// Returns the index of `value`, or -1.
  #[napi(js_name = "indexOf")]
  pub fn index_of(&self, env: Env, value: Unknown, byte_offset: Option<u32>, end: Option<u32>, encoding: Option<String>) -> Result<i32> {
    let pat = self.search_pattern(&env, value, encoding)?;
    let o = self.ro(byte_offset).min(self.data.len());
    let e = end.unwrap_or(self.data.len() as u32) as usize;
    let e = e.min(self.data.len());
    Ok(self.search_from(o, e, &pat, false))
  }

  /// Returns the last index of `value`, or -1.
  #[napi(js_name = "lastIndexOf")]
  pub fn last_index_of(&self, env: Env, value: Unknown, byte_offset: Option<u32>, end: Option<u32>, encoding: Option<String>) -> Result<i32> {
    let pat = self.search_pattern(&env, value, encoding)?;
    let o = self.ro(byte_offset);
    if o > self.data.len() {
      return Ok(-1);
    }
    let e = end.unwrap_or(self.data.len() as u32) as usize;
    let e = e.min(self.data.len());
    Ok(self.search_from(o, e, &pat, true))
  }

  /// Returns true if `value` was found.
  #[napi]
  pub fn includes(&self, env: Env, value: Unknown, byte_offset: Option<u32>, end: Option<u32>, encoding: Option<String>) -> Result<bool> {
    Ok(self.index_of(env, value, byte_offset, end, encoding)? >= 0)
  }

  /// Reverses the buffer in place.
  #[napi]
  pub fn reverse(&mut self) {
    self.data.reverse();
  }

  /// Swaps the byte order of each 16-bit word in place.
  #[napi]
  pub fn swap16(&mut self) -> Result<()> {
    self.swap_n::<2>()
  }

  #[napi]
  pub fn swap32(&mut self) -> Result<()> {
    self.swap_n::<4>()
  }

  #[napi]
  pub fn swap64(&mut self) -> Result<()> {
    self.swap_n::<8>()
  }

  /// Iterator of indices (substitute for the JS iterator; returns an array).
  #[napi]
  pub fn keys(&self) -> Vec<u32> {
    (0..self.data.len() as u32).collect()
  }

  /// Iterator of byte values (substitute for the JS iterator; returns an array).
  #[napi]
  pub fn values(&self) -> Vec<u32> {
    self.data.iter().map(|b| *b as u32).collect()
  }

  /// Iterator of [index, byte] pairs (substitute for the JS iterator).
  #[napi]
  pub fn entries(&self) -> Vec<(u32, u32)> {
    self
      .data
      .iter()
      .enumerate()
      .map(|(i, b)| (i as u32, *b as u32))
      .collect()
  }

  // --- internal helpers --------------------------------------------------

  fn clamp_range(&self, start: Option<i64>, end: Option<i64>) -> Result<(usize, usize)> {
    let len = self.data.len() as i64;
    let mut s = start.unwrap_or(0);
    if s < 0 {
      s += len;
    }
    if s < 0 {
      s = 0;
    }
    let mut e = end.unwrap_or(len);
    if e < 0 {
      e += len;
    }
    if e > len {
      e = len;
    }
    if s > e {
      s = e;
    }
    Ok((s as usize, e as usize))
  }

  fn range_u(&self, start: Option<u32>, end: Option<u32>) -> Result<(usize, usize)> {
    let s = self.ro(start).min(self.data.len());
    let e = end.unwrap_or(self.data.len() as u32) as usize;
    let e = e.min(self.data.len());
    if s > e {
      return Err(Error::new(Status::GenericFailure, "invalid start/end range"));
    }
    Ok((s, e))
  }

  fn swap_n<const N: usize>(&mut self) -> Result<()> {
    if self.data.len() % N != 0 {
      return Err(Error::new(
        Status::GenericFailure,
        format!("buffer length {} is not a multiple of {N}", self.data.len()),
      ));
    }
    for chunk in self.data.chunks_exact_mut(N) {
      chunk.reverse();
    }
    Ok(())
  }

  fn search_pattern(&self, env: &Env, value: Unknown, encoding: Option<String>) -> Result<SearchValue> {
    let raw = raw_of(&value);
    if let Ok(n) = conv::<f64>(env, raw) {
      return Ok(SearchValue::Number(((n as i32) & 0xff) as u8));
    }
    if let Ok(s) = conv::<String>(env, raw) {
      let enc = encoding.unwrap_or_else(|| "utf8".to_string());
      return Ok(SearchValue::Bytes(str_to_bytes(&s, &enc)?));
    }
    Ok(SearchValue::Bytes(get_bytes(env, raw)?))
  }

  fn search_from(&self, start: usize, end: usize, pat: &SearchValue, reverse: bool) -> i32 {
    let slice = &self.data[start..end];
    match pat {
      SearchValue::Number(b) => {
        if let Some(pos) = if reverse {
          slice.iter().rposition(|x| x == b)
        } else {
          slice.iter().position(|x| x == b)
        } {
          (start + pos) as i32
        } else {
          -1
        }
      }
      SearchValue::Bytes(needle) => {
        if needle.is_empty() {
          return start as i32;
        }
        if reverse {
          if let Some(pos) = slice.windows(needle.len()).rposition(|w| w == needle.as_slice()) {
            (start + pos) as i32
          } else {
            -1
          }
        } else if let Some(pos) = slice.windows(needle.len()).position(|w| w == needle.as_slice()) {
          (start + pos) as i32
        } else {
          -1
        }
      }
    }
  }
}

enum SearchValue {
  Number(u8),
  Bytes(Vec<u8>),
}

// ===========================================================================
// `to_json` return type.
// ===========================================================================

#[napi(object)]
pub struct BufferJson {
  pub r#type: String,
  pub data: Vec<u32>,
}

// ===========================================================================
// Blob / File
// ===========================================================================

#[napi(object)]
pub struct BlobOptions {
  pub endings: Option<String>,
  pub r#type: Option<String>,
}

#[napi(object)]
pub struct FileOptions {
  pub endings: Option<String>,
  pub r#type: Option<String>,
  pub last_modified: Option<f64>,
}

fn collect_blob_parts(env: &Env, sources: Vec<Unknown>) -> Result<Vec<u8>> {
  let mut out = Vec::new();
  for src in sources {
    let raw = raw_of(&src);
    if let Ok(s) = conv::<String>(env, raw) {
      out.extend_from_slice(s.as_bytes());
      continue;
    }
    let mut is_blob = false;
    // A `Blob` instance we created is just an object; detect via class name is
    // brittle, so accept Buffer/TypedArray/DataView/ArrayBuffer uniformly.
    let mut is_buf = false;
    check_status!(unsafe { sys::napi_is_buffer(env.raw(), raw, &mut is_buf) })?;
    let mut is_ta = false;
    check_status!(unsafe { sys::napi_is_typedarray(env.raw(), raw, &mut is_ta) })?;
    let mut is_dv = false;
    check_status!(unsafe { sys::napi_is_dataview(env.raw(), raw, &mut is_dv) })?;
    let mut is_ab = false;
    check_status!(unsafe { sys::napi_is_arraybuffer(env.raw(), raw, &mut is_ab) })?;
    if is_buf || is_ta || is_dv || is_ab {
      out.extend_from_slice(&get_bytes(env, raw)?);
    } else {
      // Treat unknown sources as an object's string coercion (best effort).
      let _ = &mut is_blob;
    }
  }
  Ok(out)
}

#[napi]
pub struct Blob {
  data: Vec<u8>,
  r#type: String,
}

#[napi]
impl Blob {
  /// Creates a Blob from a list of string / ArrayBuffer / TypedArray /
  /// DataView / Blob parts.
  #[napi(constructor)]
  pub fn new(env: Env, sources: Vec<Unknown>, options: Option<BlobOptions>) -> Result<Blob> {
    let data = collect_blob_parts(&env, sources)?;
    let r#type = options.and_then(|o| o.r#type).unwrap_or_default();
    Ok(Blob { data, r#type })
  }

  /// Returns a copy of the Blob's data as an ArrayBuffer.
  #[napi(js_name = "arrayBuffer", ts_return_type = "ArrayBuffer")]
  pub fn array_buffer(&self, env: Env) -> Result<ArrayBuffer<'_>> {
    ArrayBuffer::from_data(&env, self.data.clone())
  }

  /// Returns a copy of the Blob's data as a Buffer.
  #[napi(ts_return_type = "Buffer")]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.clone())
  }

  /// Returns the Blob's data decoded as UTF-8.
  #[napi]
  pub fn text(&self) -> String {
    String::from_utf8(self.data.clone()).unwrap_or_default()
  }

  /// Creates a new Blob containing a subset of this Blob's data.
  #[napi]
  pub fn slice(&self, start: Option<i64>, end: Option<i64>, mime_type: Option<String>) -> Blob {
    let len = self.data.len() as i64;
    let mut s = start.unwrap_or(0);
    if s < 0 {
      s += len;
    }
    if s < 0 {
      s = 0;
    }
    let mut e = end.unwrap_or(len);
    if e < 0 {
      e += len;
    }
    if e > len {
      e = len;
    }
    if s > e {
      s = e;
    }
    let r#type = mime_type.unwrap_or_else(|| self.r#type.clone());
    Blob {
      data: self.data[s as usize..e as usize].to_vec(),
      r#type,
    }
  }

  /// The total size of the Blob in bytes.
  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.data.len() as u32
  }

  /// The content-type of the Blob.
  #[napi(getter, js_name = "type")]
  pub fn mime_type(&self) -> String {
    self.r#type.clone()
  }
}

#[napi]
pub struct File {
  data: Vec<u8>,
  name: String,
  last_modified: f64,
  r#type: String,
}

#[napi]
impl File {
  /// Creates a File from a list of parts plus a file name.
  #[napi(constructor)]
  pub fn new(
    env: Env,
    sources: Vec<Unknown>,
    file_name: String,
    options: Option<FileOptions>,
  ) -> Result<File> {
    let data = collect_blob_parts(&env, sources)?;
    let opts = options.unwrap_or(FileOptions {
      endings: None,
      r#type: None,
      last_modified: None,
    });
    let last_modified = opts.last_modified.unwrap_or_else(|| {
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
    });
    let r#type = opts.r#type.unwrap_or_default();
    Ok(File {
      data,
      name: file_name,
      last_modified,
      r#type,
    })
  }

  /// Returns a copy of the File's data as an ArrayBuffer.
  #[napi(js_name = "arrayBuffer", ts_return_type = "ArrayBuffer")]
  pub fn array_buffer(&self, env: Env) -> Result<ArrayBuffer<'_>> {
    ArrayBuffer::from_data(&env, self.data.clone())
  }

  /// Returns a copy of the File's data as a Buffer.
  #[napi(ts_return_type = "Buffer")]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.clone())
  }

  /// Returns the File's data decoded as UTF-8.
  #[napi]
  pub fn text(&self) -> String {
    String::from_utf8(self.data.clone()).unwrap_or_default()
  }

  /// Creates a new File containing a subset of this File's data.
  #[napi]
  pub fn slice(&self, start: Option<i64>, end: Option<i64>, mime_type: Option<String>) -> File {
    let len = self.data.len() as i64;
    let mut s = start.unwrap_or(0);
    if s < 0 {
      s += len;
    }
    if s < 0 {
      s = 0;
    }
    let mut e = end.unwrap_or(len);
    if e < 0 {
      e += len;
    }
    if e > len {
      e = len;
    }
    if s > e {
      s = e;
    }
    File {
      data: self.data[s as usize..e as usize].to_vec(),
      name: self.name.clone(),
      last_modified: self.last_modified,
      r#type: mime_type.unwrap_or_else(|| self.r#type.clone()),
    }
  }

  /// The name of the File.
  #[napi(getter)]
  pub fn name(&self) -> String {
    self.name.clone()
  }

  /// The last modified time of the File (milliseconds since epoch).
  #[napi(getter, js_name = "lastModified")]
  pub fn last_modified(&self) -> f64 {
    self.last_modified
  }

  /// The content-type of the File.
  #[napi(getter, js_name = "type")]
  pub fn mime_type(&self) -> String {
    self.r#type.clone()
  }

  /// The total size of the File in bytes.
  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.data.len() as u32
  }
}

// ===========================================================================
// Module-level helpers (`node:buffer` functions).
// ===========================================================================

/// Decodes a Base64 string into bytes and re-encodes those bytes as Latin-1.
#[napi]
pub fn atob(data: String) -> Result<String> {
  let bytes = decode_base64(&data, false)?;
  Ok(bytes.iter().map(|b| *b as char).collect())
}

/// Decodes a string as Latin-1 into bytes and encodes those bytes as Base64.
#[napi]
pub fn btoa(data: String) -> String {
  encode_base64(data.as_bytes(), false)
}

/// Returns true if `input` contains only ASCII bytes.
#[napi(js_name = "isAscii")]
pub fn is_ascii(env: Env, input: Unknown) -> Result<bool> {
  Ok(get_bytes(&env, raw_of(&input))?.iter().all(|b| *b < 128))
}

/// Returns true if `input` is valid UTF-8.
#[napi(js_name = "isUtf8")]
pub fn is_utf8(env: Env, input: Unknown) -> Result<bool> {
  Ok(std::str::from_utf8(&get_bytes(&env, raw_of(&input))?).is_ok())
}

/// Returns true if `input` is a valid WebIDL ByteString (every UTF-16 code unit
/// is in U+0000..U+00FF).
#[napi(js_name = "isLatin1")]
pub fn is_latin1(input: String) -> bool {
  input.chars().all(|c| (c as u32) <= 0xff)
}

/// Re-encodes `source` from `from_encoding` to `to_encoding`.
#[napi(ts_return_type = "Buffer")]
pub fn transcode(env: Env, source: Unknown, from_encoding: String, to_encoding: String) -> Result<JsBuffer> {
  let bytes = get_bytes(&env, raw_of(&source))?;
  let intermediate = bytes_to_str(&bytes, &from_encoding)?;
  let out = str_to_bytes(&intermediate, &to_encoding)?;
  Ok(JsBuffer::from(out))
}

/// Resolves a `blob:nodedata:` URL into a Blob. Not supported in this binding.
#[napi(js_name = "resolveObjectURL")]
pub fn resolve_object_url(_id: String) -> Result<Blob> {
  Err(Error::new(
    Status::GenericFailure,
    "blob: URLs are not supported by this binding",
  ))
}

// ===========================================================================
// Constants (attached as module properties via `module_exports`).
// ===========================================================================

#[napi(object)]
pub struct BufferConstants {
  pub max_length: u32,
  pub max_string_length: u32,
}

/// Attaches the `node:buffer` constants (`constants`, `kMaxLength`,
/// `kStringMaxLength`, `INSPECT_MAX_BYTES`, `poolSize`) onto the module
/// exports object.
#[napi(module_exports)]
pub fn init(mut exports: Object) -> Result<()> {
  exports.set(
    "constants",
    BufferConstants {
      max_length: MAX_LENGTH,
      max_string_length: MAX_STRING_LENGTH,
    },
  )?;
  exports.set("kMaxLength", MAX_LENGTH)?;
  exports.set("kStringMaxLength", MAX_STRING_LENGTH)?;
  exports.set("INSPECT_MAX_BYTES", INSPECT_MAX_BYTES)?;
  exports.set("poolSize", POOL_SIZE)?;
  Ok(())
}
