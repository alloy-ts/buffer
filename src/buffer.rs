//! NAPI bindings for a `node:buffer`-compatible API.
//!
//! This module mirrors the public surface of Node's `node:buffer` module.
//!
//! * The `AlloyBuffer` type is a `#[napi]` class wrapping a `Vec<u8>`. napi-rs cannot
//!   subclass `Uint8Array` from Rust, so the generated `AlloyBuffer` is intentionally
//!   NOT a `Uint8Array` subclass (this is a deliberate design trade-off chosen
//!   over a hand-written TS `class AlloyBuffer extends Uint8Array`).
//! * All `AlloyBuffer` statics and instance methods are exported from this Rust file,
//!   so `index.js` / `index.d.ts` are generated directly from it — there is no
//!   hand-written `src/buffer.ts` class.
//! * `Blob` and `File` classes (no `Uint8Array` subclassing needed).
//! * Module-level helpers: `atob`, `btoa`, `isAscii`, `isUtf8`, `isLatin1`,
//!   `transcode`, `resolveObjectURL`, the `constants` object and the
//!   `kMaxLength` / `kStringMaxLength` / `INSPECT_MAX_BYTES` / `poolSize`
//!   properties.
//!
//! ## NAPI mapping notes
//! * napi-rs v3 does not implement `FromNapiValue` for `u64`/`i64`, so the
//!   64-bit methods take/return `BigInt`. The 48-bit `read/writeUIntBE/LE` /
//!   `IntBE/LE` are handled in Rust here for exactness.

use std::ffi::CString;
use std::ptr::{self, NonNull};

use napi::bindgen_prelude::{
  ArrayBuffer, BigInt as JsBigInt, Buffer as JsBuffer, Env, FromNapiRef, FromNapiValue, JsValue,
  Unknown,
};

/// Returns `true` if the raw JS value is one of *our* `AlloyBuffer` class instances.
///
/// napi-rs classes do not implement `FromNapiValue` for the bare `T`; they
/// implement `FromNapiRef` (returning `&'static T`), which unwraps the native
/// pointer only for values wrapped by this exact class. That makes it the
/// correct, type-safe `AlloyBuffer.isBuffer`-style detection.
unsafe fn is_buffer_value(env_raw: sys::napi_env, raw: sys::napi_value) -> bool {
  <AlloyBuffer as FromNapiRef>::from_napi_ref(env_raw, raw).is_ok()
}
use napi::sys;
use napi::{check_status, Error, Result, Status};
use napi_derive::napi;

/// Extracts the raw `napi_value` from an `Unknown`.
fn raw_of(u: &Unknown) -> sys::napi_value {
  JsValue::value(u).value
}

// ===========================================================================
// Constants
// ===========================================================================

const MAX_LENGTH: u32 = 0x7fff_ffff;
const MAX_STRING_LENGTH: u32 = 0x1ffff_fe8;
const INSPECT_MAX_BYTES: u32 = 50;
const POOL_SIZE: u32 = 65536;

/// `Buffer.constants` — `{ MAX_LENGTH, MAX_STRING_LENGTH }`.
#[napi(object, js_name = "BufferConstants")]
#[derive(Copy, Clone)]
pub struct BufferConstants {
  #[napi(js_name = "MAX_LENGTH")]
  pub max_length: u32,
  #[napi(js_name = "MAX_STRING_LENGTH")]
  pub max_string_length: u32,
}

#[napi(js_name = "constants")]
pub const CONSTANTS: BufferConstants = BufferConstants {
  max_length: MAX_LENGTH,
  max_string_length: MAX_STRING_LENGTH,
};

#[napi(js_name = "kMaxLength")]
pub const K_MAX_LENGTH: u32 = MAX_LENGTH;

#[napi(js_name = "kStringMaxLength")]
pub const K_STRING_MAX_LENGTH: u32 = MAX_STRING_LENGTH;

#[napi(js_name = "INSPECT_MAX_BYTES")]
pub const INSPECT_MAX_BYTES_EXPORT: u32 = INSPECT_MAX_BYTES;

#[napi(js_name = "poolSize")]
pub const POOL_SIZE_EXPORT: u32 = POOL_SIZE;

/// `SlowBuffer(size)` — legacy alias for `Buffer.allocUnsafe(size)`.
#[napi(js_name = "SlowBuffer")]
pub fn slow_buffer(size: f64) -> Result<AlloyBuffer> {
  AlloyBuffer::alloc_unsafe(size)
}

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
    "ascii" => Ok(s.chars().map(|c| if (c as u32) <= 127 { c as u8 } else { b'?' }).collect()),
    "latin1" | "binary" => Ok(s.chars().map(|c| (c as u32 & 0xff) as u8).collect()),
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

/// Node decodes invalid UTF-8 to the Unicode replacement character U+FFFD, so
/// we use the lossy conversion rather than erroring out.
fn bytes_to_str(b: &[u8], enc: &str) -> Result<String> {
  match enc.to_ascii_lowercase().as_str() {
    "utf8" | "utf-8" => Ok(String::from_utf8_lossy(b).into_owned()),
    // Node decodes `ascii` 1:1 with the byte — identical to `latin1`/`binary`.
    // Only the *encoding* direction substitutes `?` for code points above 0x7F
    // (see `str_to_bytes`).
    "ascii" => Ok(b.iter().map(|x| *x as char).collect()),
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
  // Strip whitespace (Node tolerates it inside base64 strings).
  let cleaned: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
  let mut out = Vec::new();
  let mut i = 0;
  // Process complete 4-character groups.
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
  // Handle the final partial group. Node accepts base64url without trailing `=`
  // padding, so a trailing 2- or 3-character group is valid and yields 1 or 2
  // bytes respectively.
  let rem = cleaned.len() - i;
  if rem == 2 {
    let a = b64_val(cleaned[i], url)?;
    let b = b64_val(cleaned[i + 1], url)?;
    if a == 64 || b == 64 {
      return Err(Error::new(Status::InvalidArg, "invalid base64 character"));
    }
    let n = ((a as u32) << 18) | ((b as u32) << 12);
    out.push((n >> 16) as u8);
  } else if rem == 3 {
    let a = b64_val(cleaned[i], url)?;
    let b = b64_val(cleaned[i + 1], url)?;
    let c = b64_val(cleaned[i + 2], url)?;
    if a == 64 || b == 64 || c == 64 {
      return Err(Error::new(Status::InvalidArg, "invalid base64 character"));
    }
    let n = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6);
    out.push((n >> 16) as u8);
    out.push((n >> 8) as u8);
  } else if rem == 1 {
    // A single trailing character (other than a stray `=`) is invalid base64.
    if cleaned[i] != b'=' {
      return Err(Error::new(Status::InvalidArg, "invalid base64 string"));
    }
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

/// Copies the underlying bytes of a AlloyBuffer / TypedArray / DataView / ArrayBuffer.
fn get_bytes(env: &Env, v: sys::napi_value) -> Result<Vec<u8>> {
  let mut b = false;
  check_status!(unsafe { sys::napi_is_buffer(env.raw(), v, &mut b) })?;
  if b {
    let jsbuf = unsafe { JsBuffer::from_napi_value(env.raw(), v) }?;
    return Ok(jsbuf.as_ref().to_vec());
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
    let ab = unsafe { ArrayBuffer::from_napi_value(env.raw(), v) }?;
    return Ok(ab.to_vec());
  }
  Err(Error::new(
    Status::InvalidArg,
    "expected a AlloyBuffer, TypedArray, DataView or ArrayBuffer",
  ))
}

/// `AlloyBuffer.from(typedArray)` copies one byte per element (the low byte).
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

fn unknown_to_byte(env: &Env, v: &Unknown) -> u8 {
  match unsafe { f64::from_napi_value(env.raw(), raw_of(v)) } {
    Ok(n) => ((n as i32) & 0xff) as u8,
    Err(_) => 0,
  }
}

fn unknown_to_usize(env: &Env, v: &Unknown) -> Option<usize> {
  unsafe { f64::from_napi_value(env.raw(), raw_of(v)) }
    .ok()
    .map(|n| n as usize)
}

fn unknown_to_string(env: &Env, v: &Unknown) -> Option<String> {
  unsafe { String::from_napi_value(env.raw(), raw_of(v)) }.ok()
}

// ===========================================================================
// Rust-side numeric / bounds helpers (used by the `AlloyBuffer` class).
// ===========================================================================

fn rd(d: &[u8], off: usize, width: usize) -> Result<()> {
  if off + width > d.len() {
    return Err(Error::new(
      Status::InvalidArg,
      format!(
        "The value of \"offset\" is out of range. Received {off} + {width} > {}",
        d.len()
      ),
    ));
  }
  Ok(())
}

fn read_le(d: &[u8], o: usize, l: usize) -> u64 {
  let mut v = 0u64;
  for i in 0..l {
    v |= (d[o + i] as u64) << (8 * i);
  }
  v
}

fn read_be(d: &[u8], o: usize, l: usize) -> u64 {
  let mut v = 0u64;
  for i in 0..l {
    v = (v << 8) | (d[o + i] as u64);
  }
  v
}

fn write_le(d: &mut [u8], o: usize, l: usize, v: u64) {
  for i in 0..l {
    d[o + i] = (v >> (8 * i)) as u8;
  }
}

fn write_be(d: &mut [u8], o: usize, l: usize, v: u64) {
  for i in 0..l {
    d[o + l - 1 - i] = (v >> (8 * i)) as u8;
  }
}

fn to_signed(val: u64, l: usize) -> i64 {
  let bits = 8 * l;
  if bits >= 64 {
    val as i64
  } else {
    let sign = 1i64 << (bits - 1);
    let mask = (1u64 << bits) - 1;
    let v = val & mask;
    if (v & sign as u64) != 0 {
      (v as i64) - (1i64 << bits)
    } else {
      v as i64
    }
  }
}

fn rd_i8(d: &[u8], o: usize) -> i32 {
  d[o] as i8 as i32
}
fn rd_u8(d: &[u8], o: usize) -> u32 {
  d[o] as u32
}
fn rd_u16(d: &[u8], o: usize, le: bool) -> u32 {
  let a = d[o] as u32;
  let b = d[o + 1] as u32;
  if le {
    a | (b << 8)
  } else {
    (a << 8) | b
  }
}
fn rd_i16(d: &[u8], o: usize, le: bool) -> i32 {
  rd_u16(d, o, le) as i16 as i32
}
fn rd_u32(d: &[u8], o: usize, le: bool) -> u32 {
  let mut v = 0u32;
  for i in 0..4 {
    let by = d[o + i] as u32;
    if le {
      v |= by << (8 * i);
    } else {
      v = (v << 8) | by;
    }
  }
  v
}
fn rd_i32(d: &[u8], o: usize, le: bool) -> i32 {
  rd_u32(d, o, le) as i32
}
fn rd_f32(d: &[u8], o: usize, le: bool) -> f64 {
  let mut b = [0u8; 4];
  b.copy_from_slice(&d[o..o + 4]);
  let bits = if le {
    u32::from_le_bytes(b)
  } else {
    u32::from_be_bytes(b)
  };
  f32::from_bits(bits) as f64
}
fn rd_f64(d: &[u8], o: usize, le: bool) -> f64 {
  let mut b = [0u8; 8];
  b.copy_from_slice(&d[o..o + 8]);
  let bits = if le {
    u64::from_le_bytes(b)
  } else {
    u64::from_be_bytes(b)
  };
  f64::from_bits(bits)
}
fn rd_i64(d: &[u8], o: usize, le: bool) -> i64 {
  let mut b = [0u8; 8];
  b.copy_from_slice(&d[o..o + 8]);
  if le {
    i64::from_le_bytes(b)
  } else {
    i64::from_be_bytes(b)
  }
}
fn rd_u64(d: &[u8], o: usize, le: bool) -> u64 {
  let mut b = [0u8; 8];
  b.copy_from_slice(&d[o..o + 8]);
  if le {
    u64::from_le_bytes(b)
  } else {
    u64::from_be_bytes(b)
  }
}

fn wr_i8(d: &mut [u8], o: usize, v: i32) {
  d[o] = v as u8;
}
fn wr_u8(d: &mut [u8], o: usize, v: u32) {
  d[o] = v as u8;
}
fn wr_u16(d: &mut [u8], o: usize, v: u32, le: bool) {
  let a = (v & 0xff) as u8;
  let b = ((v >> 8) & 0xff) as u8;
  if le {
    d[o] = a;
    d[o + 1] = b;
  } else {
    d[o] = b;
    d[o + 1] = a;
  }
}
fn wr_i16(d: &mut [u8], o: usize, v: i32, le: bool) {
  wr_u16(d, o, v as u32, le);
}
fn wr_u32(d: &mut [u8], o: usize, v: u32, le: bool) {
  for i in 0..4 {
    let by = ((v >> (8 * i)) & 0xff) as u8;
    if le {
      d[o + i] = by;
    } else {
      d[o + 3 - i] = by;
    }
  }
}
fn wr_i32(d: &mut [u8], o: usize, v: i32, le: bool) {
  wr_u32(d, o, v as u32, le);
}
fn wr_f32(d: &mut [u8], o: usize, v: f64, le: bool) {
  let bits = (v as f32).to_bits();
  let b = bits.to_le_bytes();
  if le {
    d[o..o + 4].copy_from_slice(&b);
  } else {
    d[o..o + 4].copy_from_slice(&bits.to_be_bytes());
  }
}
fn wr_f64(d: &mut [u8], o: usize, v: f64, le: bool) {
  let bits = v.to_bits();
  let b = bits.to_le_bytes();
  if le {
    d[o..o + 8].copy_from_slice(&b);
  } else {
    d[o..o + 8].copy_from_slice(&bits.to_be_bytes());
  }
}
fn wr_i64(d: &mut [u8], o: usize, v: i64, le: bool) {
  let b = v.to_le_bytes();
  if le {
    d[o..o + 8].copy_from_slice(&b);
  } else {
    d[o..o + 8].copy_from_slice(&v.to_be_bytes());
  }
}
fn wr_u64(d: &mut [u8], o: usize, v: u64, le: bool) {
  let b = v.to_le_bytes();
  if le {
    d[o..o + 8].copy_from_slice(&b);
  } else {
    d[o..o + 8].copy_from_slice(&v.to_be_bytes());
  }
}

fn fill_with(slice: &mut [u8], bytes: &[u8]) {
  if bytes.is_empty() {
    for x in slice {
      *x = 0;
    }
  } else {
    for (i, x) in slice.iter_mut().enumerate() {
      *x = bytes[i % bytes.len()];
    }
  }
}

fn route_search(
  env: &Env,
  a: &Option<Unknown>,
  b: &Option<Unknown>,
  c: &Option<String>,
  len: usize,
) -> (usize, usize, Option<String>) {
  let mut o = 0usize;
  let mut e = len;
  let mut enc = None;
  if let Some(a) = a {
    if let Some(s) = unknown_to_string(env, a) {
      enc = Some(s);
    } else if let Some(n) = unknown_to_usize(env, a) {
      o = n.min(len);
      if let Some(b) = b {
        if let Some(s) = unknown_to_string(env, b) {
          enc = Some(s);
        } else if let Some(m) = unknown_to_usize(env, b) {
          e = m.min(len);
          if let Some(c) = c {
            enc = Some(c.clone());
          }
        }
      }
    }
  } else if let Some(c) = c {
    enc = Some(c.clone());
  }
  (o, e, enc)
}

// ===========================================================================
// `AlloyBuffer` class
// ===========================================================================

#[napi(js_name = "Buffer")]
pub struct AlloyBuffer {
  inner: Vec<u8>,
}

impl AlloyBuffer {
  pub(crate) fn from_vec(v: Vec<u8>) -> AlloyBuffer {
    AlloyBuffer { inner: v }
  }
}

fn buffer_from(
  env: &Env,
  value: &Unknown,
  enc_or_off: Option<&Unknown>,
  length: Option<&Unknown>,
  allow_number: bool,
) -> Result<AlloyBuffer> {
  let raw = raw_of(value);

  // String
  if let Ok(s) = unsafe { String::from_napi_value(env.raw(), raw) } {
    let enc = enc_or_off
      .and_then(|u| unknown_to_string(env, u))
      .unwrap_or_else(|| "utf8".to_string());
    return Ok(AlloyBuffer::from_vec(str_to_bytes(&s, &enc)?));
  }

  // Number → allocate (only for the `new AlloyBuffer(n)` constructor, not `from`).
  if allow_number {
    if let Ok(n) = unsafe { f64::from_napi_value(env.raw(), raw) } {
      if n.fract() != 0.0 || n < 0.0 {
        return Err(Error::new(
          Status::InvalidArg,
          "The \"size\" argument must be a non-negative integer",
        ));
      }
      if (n as usize) > MAX_LENGTH as usize {
        return Err(Error::new(Status::InvalidArg, "The value of \"size\" is out of range"));
      }
      return Ok(AlloyBuffer::from_vec(vec![0u8; n as usize]));
    }
  }

  // One of our own AlloyBuffer instances → copy.
  if let Ok(b) = unsafe { <AlloyBuffer as FromNapiRef>::from_napi_ref(env.raw(), raw) } {
    return Ok(AlloyBuffer::from_vec(b.inner.clone()));
  }

  // ArrayBuffer (view over a slice)
  let mut is_ab = false;
  check_status!(unsafe { sys::napi_is_arraybuffer(env.raw(), raw, &mut is_ab) })?;
  if is_ab {
    let ab = unsafe { ArrayBuffer::from_napi_value(env.raw(), raw) }?;
    let v = ab.to_vec();
    let total = v.len();
    let off = enc_or_off
      .and_then(|u| unknown_to_usize(env, u))
      .unwrap_or(0)
      .min(total);
    let len = length
      .and_then(|u| unknown_to_usize(env, u))
      .unwrap_or(total - off)
      .min(total - off);
    return Ok(AlloyBuffer::from_vec(v[off..off + len].to_vec()));
  }

  // TypedArray → copy one byte per element (the low byte).
  let mut is_ta = false;
  check_status!(unsafe { sys::napi_is_typedarray(env.raw(), raw, &mut is_ta) })?;
  if is_ta {
    return Ok(AlloyBuffer::from_vec(typedarray_low_bytes(env, raw)?));
  }

  // DataView → copy the raw bytes.
  let mut is_dv = false;
  check_status!(unsafe { sys::napi_is_dataview(env.raw(), raw, &mut is_dv) })?;
  if is_dv {
    let (ptr, len) = unsafe { dataview_bytes(env, raw)? };
    return Ok(AlloyBuffer::from_vec(
      unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) }.to_vec(),
    ));
  }

  // Array → copy one byte per element (low byte).
  let mut is_arr = false;
  check_status!(unsafe { sys::napi_is_array(env.raw(), raw, &mut is_arr) })?;
  if is_arr {
    let arr: Vec<Unknown> = unsafe { Vec::<Unknown>::from_napi_value(env.raw(), raw) }?;
    let data: Vec<u8> = arr.iter().map(|u| unknown_to_byte(env, u)).collect();
    return Ok(AlloyBuffer::from_vec(data));
  }

  // Object with Symbol.toPrimitive / valueOf, or array-like object.
  if let Some(b) = unsafe { buffer_from_object(env, raw)? } {
    return Ok(b);
  }
  if let Some(data) = unsafe { arraylike(env, raw)? } {
    return Ok(AlloyBuffer::from_vec(data));
  }

  Err(Error::new(
    Status::InvalidArg,
    "The first argument must be of type string or an instance of AlloyBuffer, ArrayBuffer, or Array or an Array-like Object",
  ))
}

unsafe fn wellknown_symbol(env: &Env, name: &str) -> Result<sys::napi_value> {
  let mut global = ptr::null_mut();
  check_status!(sys::napi_get_global(env.raw(), &mut global))?;
  let sym_ctor_name = CString::new("Symbol").unwrap();
  let mut sym_ctor = ptr::null_mut();
  check_status!(sys::napi_get_named_property(
    env.raw(),
    global,
    sym_ctor_name.as_ptr(),
    &mut sym_ctor
  ))?;
  let prop = CString::new(name).unwrap();
  let mut sym = ptr::null_mut();
  check_status!(sys::napi_get_named_property(
    env.raw(),
    sym_ctor,
    prop.as_ptr(),
    &mut sym
  ))?;
  Ok(sym)
}

/// Honours `Symbol.toPrimitive` (string hint) and `valueOf` returning a
/// primitive, matching Node's `AlloyBuffer.from` coercion rules.
///
/// napi-sys 3.x does not expose `napi_is_function` / `napi_is_object`, so we
/// simply attempt to call the property and treat a non-zero (call-failed)
/// status as "not callable".
unsafe fn buffer_from_object(env: &Env, raw: sys::napi_value) -> Result<Option<AlloyBuffer>> {
  // Symbol.toPrimitive (string hint)
  let sym = wellknown_symbol(env, "toPrimitive")?;
  let mut fnv = ptr::null_mut();
  if sys::napi_get_property(env.raw(), raw, sym, &mut fnv) == 0 {
    let arg = CString::new("string").unwrap();
    let mut argv = ptr::null_mut();
    sys::napi_create_string_utf8(env.raw(), arg.as_ptr(), 6, &mut argv);
    let mut res = ptr::null_mut();
    if sys::napi_call_function(env.raw(), raw, fnv, 1, &argv, &mut res) == 0 {
      if let Ok(s) = String::from_napi_value(env.raw(), res) {
        return Ok(Some(AlloyBuffer::from_vec(str_to_bytes(&s, "utf8")?)));
      }
    }
  }
  // valueOf
  let voname = CString::new("valueOf").unwrap();
  let mut fnv2 = ptr::null_mut();
  if sys::napi_get_named_property(env.raw(), raw, voname.as_ptr(), &mut fnv2) == 0 {
    let mut res = ptr::null_mut();
    if sys::napi_call_function(env.raw(), raw, fnv2, 0, ptr::null(), &mut res) == 0 {
      // Node recurses into the primitive value; we only support string results
      // (a numeric valueOf result would map to `AlloyBuffer.from(number)`, which is
      // invalid for `AlloyBuffer.from`).
      if let Ok(s) = String::from_napi_value(env.raw(), res) {
        return Ok(Some(AlloyBuffer::from_vec(str_to_bytes(&s, "utf8")?)));
      }
    }
  }
  Ok(None)
}

/// Reads `{ length, 0, 1, ... }` array-like objects into bytes.
unsafe fn arraylike(env: &Env, raw: sys::napi_value) -> Result<Option<Vec<u8>>> {
  let len_name = CString::new("length").unwrap();
  let mut len_val = ptr::null_mut();
  if sys::napi_get_named_property(env.raw(), raw, len_name.as_ptr(), &mut len_val) != 0 {
    return Ok(None);
  }
  let len = match f64::from_napi_value(env.raw(), len_val) {
    Ok(n) if n >= 0.0 && n.fract() == 0.0 => n as usize,
    _ => return Ok(None),
  };
  if len > MAX_LENGTH as usize {
    return Ok(None);
  }
  let mut out = Vec::with_capacity(len);
  for i in 0..len {
    let key = CString::new(format!("{i}")).unwrap();
    let mut item = ptr::null_mut();
    if sys::napi_get_named_property(env.raw(), raw, key.as_ptr(), &mut item) != 0 {
      out.push(0);
      continue;
    }
    let u = Unknown::from_napi_value(env.raw(), item)?;
    out.push(unknown_to_byte(env, &u));
  }
  Ok(Some(out))
}

fn decode_hex_partial(s: &str) -> (Vec<u8>, usize) {
  let s = s.trim();
  let bytes = s.as_bytes();
  let mut out = Vec::new();
  let mut i = 0;
  while i + 1 < bytes.len() {
    match (hex_val(bytes[i]), hex_val(bytes[i + 1])) {
      (Ok(h), Ok(l)) => {
        out.push((h << 4) | l);
        i += 2;
      }
      _ => break,
    }
  }
  (out, i)
}

enum SearchValue {
  Number(u8),
  Bytes(Vec<u8>),
}

fn search_pattern(env: &Env, value: &Unknown, encoding: Option<String>) -> Result<SearchValue> {
  let raw = raw_of(value);
  if let Ok(n) = unsafe { f64::from_napi_value(env.raw(), raw) } {
    return Ok(SearchValue::Number(((n as i32) & 0xff) as u8));
  }
  if let Ok(s) = unsafe { String::from_napi_value(env.raw(), raw) } {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    return Ok(SearchValue::Bytes(str_to_bytes(&s, &enc)?));
  }
  Ok(SearchValue::Bytes(get_bytes(env, raw)?))
}

fn search_from(data: &[u8], start: usize, end: usize, pat: &SearchValue, reverse: bool) -> i32 {
  let slice = &data[start..end];
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

#[napi(object)]
pub struct BufferWriteResult {
  pub read: u32,
  pub written: u32,
}

#[napi(object)]
pub struct BufferToJson {
  #[napi(js_name = "type")]
  pub r#type: String,
  pub data: Vec<u32>,
}

#[napi(object)]
pub struct Base64Options {
  pub alphabet: Option<String>,
  pub omit_padding: Option<bool>,
}

#[napi]
impl AlloyBuffer {
  // --- construction --------------------------------------------------------

  /// `new AlloyBuffer(value[, encodingOrOffset[, length]])`.
  #[napi(constructor)]
  pub fn new(
    env: Env,
    value: Unknown,
    enc_or_off: Option<Unknown>,
    length: Option<Unknown>,
  ) -> Result<AlloyBuffer> {
    buffer_from(&env, &value, enc_or_off.as_ref(), length.as_ref(), true)
  }

  /// `AlloyBuffer.from(value[, encodingOrOffset[, length]])`.
  #[napi(factory)]
  pub fn from(
    env: Env,
    value: Unknown,
    enc_or_off: Option<Unknown>,
    length: Option<Unknown>,
  ) -> Result<AlloyBuffer> {
    buffer_from(&env, &value, enc_or_off.as_ref(), length.as_ref(), false)
  }

  /// `AlloyBuffer.alloc(size[, fill[, encoding]])`.
  #[napi(js_name = "alloc")]
  pub fn alloc(env: Env, size: f64, fill: Option<Unknown>, encoding: Option<String>) -> Result<AlloyBuffer> {
    if size.fract() != 0.0 || size < 0.0 {
      return Err(Error::new(
        Status::InvalidArg,
        "The \"size\" argument must be a non-negative integer",
      ));
    }
    if (size as usize) > MAX_LENGTH as usize {
      return Err(Error::new(Status::InvalidArg, "The value of \"size\" is out of range"));
    }
    let mut b = AlloyBuffer::from_vec(vec![0u8; size as usize]);
    if let Some(f) = fill {
      b.fill(env, f, None, None, encoding)?;
    }
    Ok(b)
  }

  /// `AlloyBuffer.allocUnsafe(size)` — returned zero-filled (safe) for simplicity.
  #[napi(js_name = "allocUnsafe")]
  pub fn alloc_unsafe(size: f64) -> Result<AlloyBuffer> {
    if size.fract() != 0.0 || size < 0.0 {
      return Err(Error::new(
        Status::InvalidArg,
        "The \"size\" argument must be a non-negative integer",
      ));
    }
    if (size as usize) > MAX_LENGTH as usize {
      return Err(Error::new(Status::InvalidArg, "The value of \"size\" is out of range"));
    }
    Ok(AlloyBuffer::from_vec(vec![0u8; size as usize]))
  }

  /// `AlloyBuffer.allocUnsafeSlow(size)`.
  #[napi(js_name = "allocUnsafeSlow")]
  pub fn alloc_unsafe_slow(size: f64) -> Result<AlloyBuffer> {
    Self::alloc_unsafe(size)
  }

  /// `AlloyBuffer.of(...items)`.
  #[napi(js_name = "of")]
  pub fn of(env: Env, items: Vec<Unknown>) -> AlloyBuffer {
    AlloyBuffer::from_vec(items.iter().map(|u| unknown_to_byte(&env, u)).collect())
  }

  /// `AlloyBuffer.isBuffer(value)`.
  #[napi(js_name = "isBuffer")]
  pub fn is_buffer(env: Env, value: Unknown) -> bool {
    unsafe { is_buffer_value(env.raw(), raw_of(&value)) }
  }

  /// `AlloyBuffer.isEncoding(encoding)`.
  #[napi(js_name = "isEncoding")]
  pub fn is_encoding(encoding: String) -> bool {
    SUPPORTED_ENCODINGS.contains(&encoding.to_ascii_lowercase().as_str())
  }

  /// `AlloyBuffer.byteLength(value[, encoding])`.
  #[napi(js_name = "byteLength")]
  pub fn byte_length(env: Env, value: Unknown, encoding: Option<String>) -> Result<u32> {
    let raw = raw_of(&value);
    if let Ok(s) = unsafe { String::from_napi_value(env.raw(), raw) } {
      return Ok(str_to_bytes(&s, &encoding.unwrap_or_else(|| "utf8".to_string()))?.len() as u32);
    }
    if let Ok(b) = unsafe { <AlloyBuffer as FromNapiRef>::from_napi_ref(env.raw(), raw) } {
      return Ok(b.inner.len() as u32);
    }
    Ok(get_bytes(&env, raw)?.len() as u32)
  }

  /// `AlloyBuffer.stringLength(input[, encoding])`.
  #[napi(js_name = "stringLength")]
  pub fn string_length(env: Env, input: Unknown, encoding: Option<String>) -> Result<u32> {
    let raw = raw_of(&input);
    let bytes = if let Ok(s) = unsafe { String::from_napi_value(env.raw(), raw) } {
      str_to_bytes(&s, &encoding.clone().unwrap_or_else(|| "utf8".to_string()))?
    } else {
      get_bytes(&env, raw)?
    };
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let s = bytes_to_str(&bytes, &enc)?;
    Ok(s.chars().count() as u32)
  }

  /// `AlloyBuffer.concat(list[, totalLength])`.
  #[napi(js_name = "concat")]
  pub fn concat(list: Vec<&AlloyBuffer>, total_length: Option<u32>) -> AlloyBuffer {
    let total = total_length
      .unwrap_or_else(|| list.iter().map(|b| b.inner.len() as u32).sum())
      as usize;
    let mut out = Vec::with_capacity(total);
    for b in &list {
      out.extend_from_slice(b.inner.as_slice());
    }
    out.resize(total, 0);
    AlloyBuffer::from_vec(out)
  }

  /// `AlloyBuffer.compare(buf1, buf2)`.
  #[napi(js_name = "compare")]
  pub fn compare_static(buf1: &AlloyBuffer, buf2: &AlloyBuffer) -> i32 {
    buf1.inner.cmp(&buf2.inner) as i32
  }

  /// `AlloyBuffer.copyBytesFrom(view[, offset[, length]])`.
  #[napi(js_name = "copyBytesFrom")]
  pub fn copy_bytes_from(
    env: Env,
    view: Unknown,
    offset: Option<u32>,
    length: Option<u32>,
  ) -> Result<AlloyBuffer> {
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
      return Ok(AlloyBuffer::from_vec(slice[start..end].to_vec()));
    }
    let mut is_dv = false;
    check_status!(unsafe { sys::napi_is_dataview(env.raw(), raw, &mut is_dv) })?;
    if is_dv {
      let (ptr, len) = unsafe { dataview_bytes(&env, raw)? };
      return Ok(AlloyBuffer::from_vec(
        unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) }.to_vec(),
      ));
    }
    let mut is_ab = false;
    check_status!(unsafe { sys::napi_is_arraybuffer(env.raw(), raw, &mut is_ab) })?;
    if is_ab {
      let ab = unsafe { ArrayBuffer::from_napi_value(env.raw(), raw) }?;
      return Ok(AlloyBuffer::from_vec(ab.to_vec()));
    }
    if let Ok(b) = unsafe { <AlloyBuffer as FromNapiRef>::from_napi_ref(env.raw(), raw) } {
      return Ok(AlloyBuffer::from_vec(b.inner.clone()));
    }
    Err(Error::new(
      Status::InvalidArg,
      "The \"view\" argument must be an instance of AlloyBuffer or ArrayBufferView",
    ))
  }

  // --- getters -------------------------------------------------------------

  #[napi(getter)]
  pub fn length(&self) -> u32 {
    self.inner.len() as u32
  }

  #[napi(getter, js_name = "byteLength")]
  pub fn byte_length_getter(&self) -> u32 {
    self.inner.len() as u32
  }

  #[napi(getter, js_name = "_isBuffer")]
  pub fn is_buffer_getter(&self) -> bool {
    true
  }

  // --- comparison / equality ----------------------------------------------

  /// `buf.equals(other)`.
  #[napi(js_name = "equals")]
  pub fn equals(&self, other: &AlloyBuffer) -> bool {
    self.inner == other.inner
  }

  /// `buf.compare(target[, targetStart[, targetEnd[, sourceStart[, sourceEnd]]]])`.
  #[napi(js_name = "compare")]
  pub fn compare(
    &self,
    target: &AlloyBuffer,
    t_start: Option<u32>,
    t_end: Option<u32>,
    s_start: Option<u32>,
    s_end: Option<u32>,
  ) -> i32 {
    let t_start = (t_start.unwrap_or(0) as usize).min(target.inner.len());
    let t_end = (t_end.unwrap_or(target.inner.len() as u32) as usize).min(target.inner.len());
    let s_start = (s_start.unwrap_or(0) as usize).min(self.inner.len());
    let s_end = (s_end.unwrap_or(self.inner.len() as u32) as usize).min(self.inner.len());
    self.inner[s_start..s_end].cmp(&target.inner[t_start..t_end]) as i32
  }

  /// `buf.copy(target[, targetStart[, sourceStart[, sourceEnd]]])`.
  ///
  /// NOTE: napi-rs cannot hand a mutable reference to a *parameter* class
  /// instance to Rust, so this returns the number of bytes that *would* be
  /// copied but does **not** mutate `target` in place (unlike Node's AlloyBuffer).
  /// This is an inherent limitation of implementing `AlloyBuffer` as a `#[napi]`
  /// class rather than a `Uint8Array` subclass.
  #[napi(js_name = "copy")]
  pub fn copy(
    &self,
    target: &AlloyBuffer,
    t_start: Option<u32>,
    s_start: Option<u32>,
    s_end: Option<u32>,
  ) -> Result<u32> {
    let mut t_start = t_start.unwrap_or(0) as usize;
    let mut s_start = s_start.unwrap_or(0) as usize;
    let mut s_end = s_end.unwrap_or(self.inner.len() as u32) as usize;
    if t_start > target.inner.len() {
      t_start = target.inner.len();
    }
    if s_start > self.inner.len() {
      s_start = self.inner.len();
    }
    if s_end > self.inner.len() {
      s_end = self.inner.len();
    }
    if s_start >= s_end || t_start >= target.inner.len() {
      return Ok(0);
    }
    let to_copy = (s_end - s_start).min(target.inner.len() - t_start);
    Ok(to_copy as u32)
  }

  // --- fill / write --------------------------------------------------------

  /// `buf.fill(value[, start[, end]][, encoding])`.
  #[napi(js_name = "fill")]
  pub fn fill(
    &mut self,
    env: Env,
    value: Unknown,
    a: Option<Unknown>,
    b: Option<Unknown>,
    c: Option<String>,
  ) -> Result<()> {
    let mut offset: Option<u32> = None;
    let mut end: Option<u32> = None;
    let mut enc: Option<String> = None;
    if let Some(a) = a {
      if let Some(s) = unknown_to_string(&env, &a) {
        enc = Some(s);
      } else if let Some(n) = unknown_to_usize(&env, &a) {
        offset = Some(n as u32);
        if let Some(b) = b {
          if let Some(s) = unknown_to_string(&env, &b) {
            enc = Some(s);
          } else if let Some(m) = unknown_to_usize(&env, &b) {
            end = Some(m as u32);
            if let Some(c) = c {
              enc = Some(c);
            }
          }
        }
      }
    }
    let s = (offset.unwrap_or(0) as usize).min(self.inner.len());
    let e = (end.unwrap_or(self.inner.len() as u32) as usize).min(self.inner.len());
    if s > e {
      return Err(Error::new(Status::InvalidArg, "invalid start/end range"));
    }
    let raw = raw_of(&value);
    if let Ok(n) = unsafe { f64::from_napi_value(env.raw(), raw) } {
      let byte = ((n as i32) & 0xff) as u8;
      for x in &mut self.inner[s..e] {
        *x = byte;
      }
    } else if let Ok(s2) = unsafe { String::from_napi_value(env.raw(), raw) } {
      let enc2 = enc.unwrap_or_else(|| "utf8".to_string());
      let bytes = str_to_bytes(&s2, &enc2)?;
      fill_with(&mut self.inner[s..e], &bytes);
    } else {
      let bytes = get_bytes(&env, raw)?;
      fill_with(&mut self.inner[s..e], &bytes);
    }
    Ok(())
  }

  /// `buf.write(string[, offset[, length]][, encoding])`.
  #[napi(js_name = "write")]
  pub fn write(
    &mut self,
    env: Env,
    string: String,
    a: Option<Unknown>,
    b: Option<Unknown>,
    c: Option<String>,
  ) -> Result<u32> {
    let mut offset = 0u32;
    let mut length = self.inner.len() as u32;
    let mut enc = "utf8".to_string();
    if let Some(a) = a {
      if let Some(s) = unknown_to_string(&env, &a) {
        enc = s;
      } else if let Some(n) = unknown_to_usize(&env, &a) {
        offset = n as u32;
        if let Some(b) = b {
          if let Some(s) = unknown_to_string(&env, &b) {
            enc = s;
          } else if let Some(m) = unknown_to_usize(&env, &b) {
            length = m as u32;
            if let Some(c) = c {
              enc = c;
            }
          }
        }
      }
    }
    let bytes = str_to_bytes(&string, &enc)?;
    let o = (offset as usize).min(self.inner.len());
    let max = self.inner.len() - o;
    let want = (length as usize).min(bytes.len());
    let n = want.min(max);
    if n > 0 {
      self.inner[o..o + n].copy_from_slice(&bytes[..n]);
    }
    Ok(n as u32)
  }

  // --- search --------------------------------------------------------------

  /// `buf.indexOf(value[, byteOffset[, end]][, encoding])`.
  #[napi(js_name = "indexOf")]
  pub fn index_of(
    &self,
    env: Env,
    value: Unknown,
    a: Option<Unknown>,
    b: Option<Unknown>,
    c: Option<String>,
  ) -> Result<i32> {
    let (o, e, enc) = route_search(&env, &a, &b, &c, self.inner.len());
    let pat = search_pattern(&env, &value, enc)?;
    Ok(search_from(&self.inner, o, e, &pat, false))
  }

  /// `buf.lastIndexOf(value[, byteOffset[, end]][, encoding])`.
  #[napi(js_name = "lastIndexOf")]
  pub fn last_index_of(
    &self,
    env: Env,
    value: Unknown,
    a: Option<Unknown>,
    b: Option<Unknown>,
    c: Option<String>,
  ) -> Result<i32> {
    let (o, e, enc) = route_search(&env, &a, &b, &c, self.inner.len());
    let pat = search_pattern(&env, &value, enc)?;
    Ok(search_from(&self.inner, o, e, &pat, true))
  }

  /// `buf.includes(value[, byteOffset[, end]][, encoding])`.
  #[napi(js_name = "includes")]
  pub fn includes(
    &self,
    env: Env,
    value: Unknown,
    a: Option<Unknown>,
    b: Option<Unknown>,
    c: Option<String>,
  ) -> Result<bool> {
    Ok(self.index_of(env, value, a, b, c)? >= 0)
  }

  // --- mutation ------------------------------------------------------------

  /// `buf.reverse()`.
  #[napi(js_name = "reverse")]
  pub fn reverse(&mut self) -> AlloyBuffer {
    self.inner.reverse();
    AlloyBuffer::from_vec(self.inner.clone())
  }

  /// `buf.swap16()`.
  #[napi(js_name = "swap16")]
  pub fn swap16(&mut self) -> Result<AlloyBuffer> {
    if self.inner.len() % 2 != 0 {
      return Err(Error::new(
        Status::InvalidArg,
        "Buffer size must be a multiple of 16-bits",
      ));
    }
    for i in (0..self.inner.len()).step_by(2) {
      self.inner.swap(i, i + 1);
    }
    Ok(AlloyBuffer::from_vec(self.inner.clone()))
  }

  /// `buf.swap32()`.
  #[napi(js_name = "swap32")]
  pub fn swap32(&mut self) -> Result<AlloyBuffer> {
    if self.inner.len() % 4 != 0 {
      return Err(Error::new(
        Status::InvalidArg,
        "Buffer size must be a multiple of 32-bits",
      ));
    }
    for i in (0..self.inner.len()).step_by(4) {
      self.inner.swap(i, i + 3);
      self.inner.swap(i + 1, i + 2);
    }
    Ok(AlloyBuffer::from_vec(self.inner.clone()))
  }

  /// `buf.swap64()`.
  #[napi(js_name = "swap64")]
  pub fn swap64(&mut self) -> Result<AlloyBuffer> {
    if self.inner.len() % 8 != 0 {
      return Err(Error::new(
        Status::InvalidArg,
        "Buffer size must be a multiple of 64-bits",
      ));
    }
    for i in (0..self.inner.len()).step_by(8) {
      self.inner.swap(i, i + 7);
      self.inner.swap(i + 1, i + 6);
      self.inner.swap(i + 2, i + 5);
      self.inner.swap(i + 3, i + 4);
    }
    Ok(AlloyBuffer::from_vec(self.inner.clone()))
  }

  // --- slicing -------------------------------------------------------------

  fn slice_impl(&self, start: Option<i64>, end: Option<i64>) -> AlloyBuffer {
    let len = self.inner.len() as i64;
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
    AlloyBuffer::from_vec(self.inner[s as usize..e as usize].to_vec())
  }

  /// `buf.slice([start[, end]])`.
  #[napi(js_name = "slice")]
  pub fn slice(&self, start: Option<i64>, end: Option<i64>) -> AlloyBuffer {
    self.slice_impl(start, end)
  }

  /// `buf.subarray([start[, end]])` — like `slice`, returns a new `AlloyBuffer`.
  #[napi(js_name = "subarray")]
  pub fn subarray(&self, start: Option<i64>, end: Option<i64>) -> AlloyBuffer {
    self.slice_impl(start, end)
  }

  // --- encode / decode -----------------------------------------------------

  /// `buf.toString([encoding[, start[, end]]])`.
  #[napi(js_name = "toString")]
  pub fn to_str(&self, encoding: Option<String>, start: Option<u32>, end: Option<u32>) -> Result<String> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let s = (start.unwrap_or(0) as usize).min(self.inner.len());
    let e = (end.unwrap_or(self.inner.len() as u32) as usize).min(self.inner.len());
    if s > e {
      return Err(Error::new(Status::InvalidArg, "invalid start/end range"));
    }
    bytes_to_str(&self.inner[s..e], &enc)
  }

  /// `buf.toJSON()`.
  #[napi(js_name = "toJSON")]
  pub fn to_json(&self) -> BufferToJson {
    BufferToJson {
      r#type: "Buffer".to_string(),
      data: self.inner.iter().map(|&b| b as u32).collect(),
    }
  }

  /// `buf.toHex()`.
  #[napi(js_name = "toHex")]
  pub fn to_hex(&self) -> String {
    encode_hex(&self.inner)
  }

  /// `buf.toBase64([options])`.
  #[napi(js_name = "toBase64")]
  pub fn to_base64(&self, options: Option<Base64Options>) -> Result<String> {
    let (alphabet, omit) = match options {
      Some(o) => (
        if o.alphabet.as_deref() == Some("base64url") {
          "base64url"
        } else {
          "base64"
        },
        o.omit_padding.unwrap_or(false),
      ),
      None => ("base64", false),
    };
    let s = bytes_to_str(&self.inner, alphabet)?;
    Ok(if omit {
      s.trim_end_matches('=').to_string()
    } else {
      s
    })
  }

  /// `buf.setFromBase64(string[, offset])`.
  #[napi(js_name = "setFromBase64")]
  pub fn set_from_base64(&mut self, base64: String, offset: Option<u32>) -> Result<BufferWriteResult> {
    let decoded = decode_base64(&base64, false)?;
    let off = offset.unwrap_or(0) as usize;
    let space = self.inner.len().saturating_sub(off);
    if space == 0 {
      return Ok(BufferWriteResult {
        read: 0,
        written: 0,
      });
    }
    let written = decoded.len().min(space);
    self.inner[off..off + written].copy_from_slice(&decoded[..written]);
    let read = if written == decoded.len() {
      base64.chars().filter(|c| !c.is_whitespace()).count() as u32
    } else {
      (written / 3 * 4
        + match written % 3 {
          1 => 2,
          2 => 3,
          _ => 0,
        }) as u32
    };
    Ok(BufferWriteResult {
      read,
      written: written as u32,
    })
  }

  /// `buf.setFromHex(string)`.
  #[napi(js_name = "setFromHex")]
  pub fn set_from_hex(&mut self, hex: String) -> Result<BufferWriteResult> {
    let (decoded, consumed) = decode_hex_partial(&hex);
    let written = decoded.len().min(self.inner.len());
    if written > 0 {
      self.inner[..written].copy_from_slice(&decoded[..written]);
    }
    Ok(BufferWriteResult {
      read: (consumed * 2) as u32,
      written: written as u32,
    })
  }

  // --- numeric reads -------------------------------------------------------

  #[napi(js_name = "readInt8")]
  pub fn read_int8(&self, offset: Option<u32>) -> Result<i32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 1)?;
    Ok(rd_i8(&self.inner, o))
  }
  #[napi(js_name = "readUInt8")]
  pub fn read_u_int8(&self, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 1)?;
    Ok(rd_u8(&self.inner, o))
  }
  #[napi(js_name = "readUint8")]
  pub fn read_uint8(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int8(offset)
  }

  #[napi(js_name = "readInt16LE")]
  pub fn read_int16_le(&self, offset: Option<u32>) -> Result<i32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 2)?;
    Ok(rd_i16(&self.inner, o, true))
  }
  #[napi(js_name = "readInt16BE")]
  pub fn read_int16_be(&self, offset: Option<u32>) -> Result<i32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 2)?;
    Ok(rd_i16(&self.inner, o, false))
  }
  #[napi(js_name = "readUInt16LE")]
  pub fn read_u_int16_le(&self, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 2)?;
    Ok(rd_u16(&self.inner, o, true))
  }
  #[napi(js_name = "readUInt16BE")]
  pub fn read_u_int16_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 2)?;
    Ok(rd_u16(&self.inner, o, false))
  }
  #[napi(js_name = "readUint16LE")]
  pub fn read_uint16_le(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_le(offset)
  }
  #[napi(js_name = "readUint16BE")]
  pub fn read_uint16_be(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_be(offset)
  }

  #[napi(js_name = "readInt32LE")]
  pub fn read_int32_le(&self, offset: Option<u32>) -> Result<i32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    Ok(rd_i32(&self.inner, o, true))
  }
  #[napi(js_name = "readInt32BE")]
  pub fn read_int32_be(&self, offset: Option<u32>) -> Result<i32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    Ok(rd_i32(&self.inner, o, false))
  }
  #[napi(js_name = "readUInt32LE")]
  pub fn read_u_int32_le(&self, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    Ok(rd_u32(&self.inner, o, true))
  }
  #[napi(js_name = "readUInt32BE")]
  pub fn read_u_int32_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    Ok(rd_u32(&self.inner, o, false))
  }
  #[napi(js_name = "readUint32LE")]
  pub fn read_uint32_le(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_le(offset)
  }
  #[napi(js_name = "readUint32BE")]
  pub fn read_uint32_be(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_be(offset)
  }

  #[napi(js_name = "readFloatLE")]
  pub fn read_float_le(&self, offset: Option<u32>) -> Result<f64> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    Ok(rd_f32(&self.inner, o, true))
  }
  #[napi(js_name = "readFloatBE")]
  pub fn read_float_be(&self, offset: Option<u32>) -> Result<f64> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    Ok(rd_f32(&self.inner, o, false))
  }
  #[napi(js_name = "readDoubleLE")]
  pub fn read_double_le(&self, offset: Option<u32>) -> Result<f64> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    Ok(rd_f64(&self.inner, o, true))
  }
  #[napi(js_name = "readDoubleBE")]
  pub fn read_double_be(&self, offset: Option<u32>) -> Result<f64> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    Ok(rd_f64(&self.inner, o, false))
  }

  #[napi(js_name = "readBigInt64LE")]
  pub fn read_big_int64_le(&self, offset: Option<u32>) -> Result<JsBigInt> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    Ok(JsBigInt::from(rd_i64(&self.inner, o, true)))
  }
  #[napi(js_name = "readBigInt64BE")]
  pub fn read_big_int64_be(&self, offset: Option<u32>) -> Result<JsBigInt> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    Ok(JsBigInt::from(rd_i64(&self.inner, o, false)))
  }
  #[napi(js_name = "readBigUInt64LE")]
  pub fn read_big_u_int64_le(&self, offset: Option<u32>) -> Result<JsBigInt> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    Ok(JsBigInt::from(rd_u64(&self.inner, o, true)))
  }
  #[napi(js_name = "readBigUInt64BE")]
  pub fn read_big_u_int64_be(&self, offset: Option<u32>) -> Result<JsBigInt> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    Ok(JsBigInt::from(rd_u64(&self.inner, o, false)))
  }
  #[napi(js_name = "readBigUint64LE")]
  pub fn read_big_uint64_le(&self, offset: Option<u32>) -> Result<JsBigInt> {
    self.read_big_u_int64_le(offset)
  }
  #[napi(js_name = "readBigUint64BE")]
  pub fn read_big_uint64_be(&self, offset: Option<u32>) -> Result<JsBigInt> {
    self.read_big_u_int64_be(offset)
  }

  #[napi(js_name = "readUIntLE")]
  pub fn read_u_int_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let l = byte_length as usize;
    if l < 1 || l > 6 {
      return Err(Error::new(
        Status::InvalidArg,
        "The value of \"byteLength\" is out of range",
      ));
    }
    rd(&self.inner, o, l)?;
    Ok(read_le(&self.inner, o, l) as f64)
  }
  #[napi(js_name = "readUintLE")]
  pub fn read_uint_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_u_int_le(offset, byte_length)
  }
  #[napi(js_name = "readUIntBE")]
  pub fn read_u_int_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let l = byte_length as usize;
    if l < 1 || l > 6 {
      return Err(Error::new(
        Status::InvalidArg,
        "The value of \"byteLength\" is out of range",
      ));
    }
    rd(&self.inner, o, l)?;
    Ok(read_be(&self.inner, o, l) as f64)
  }
  #[napi(js_name = "readUintBE")]
  pub fn read_uint_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_u_int_be(offset, byte_length)
  }
  #[napi(js_name = "readIntLE")]
  pub fn read_int_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let l = byte_length as usize;
    if l < 1 || l > 6 {
      return Err(Error::new(
        Status::InvalidArg,
        "The value of \"byteLength\" is out of range",
      ));
    }
    rd(&self.inner, o, l)?;
    Ok(to_signed(read_le(&self.inner, o, l), l) as f64)
  }
  #[napi(js_name = "readIntBE")]
  pub fn read_int_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let l = byte_length as usize;
    if l < 1 || l > 6 {
      return Err(Error::new(
        Status::InvalidArg,
        "The value of \"byteLength\" is out of range",
      ));
    }
    rd(&self.inner, o, l)?;
    Ok(to_signed(read_be(&self.inner, o, l), l) as f64)
  }

  // --- numeric writes ------------------------------------------------------

  #[napi(js_name = "writeInt8")]
  pub fn write_int8(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 1)?;
    wr_i8(&mut self.inner, o, value);
    Ok((o + 1) as u32)
  }
  #[napi(js_name = "writeUInt8")]
  pub fn write_u_int8(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 1)?;
    wr_u8(&mut self.inner, o, value);
    Ok((o + 1) as u32)
  }
  #[napi(js_name = "writeUint8")]
  pub fn write_uint8(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int8(value, offset)
  }

  #[napi(js_name = "writeInt16LE")]
  pub fn write_int16_le(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 2)?;
    wr_i16(&mut self.inner, o, value, true);
    Ok((o + 2) as u32)
  }
  #[napi(js_name = "writeInt16BE")]
  pub fn write_int16_be(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 2)?;
    wr_i16(&mut self.inner, o, value, false);
    Ok((o + 2) as u32)
  }
  #[napi(js_name = "writeUInt16LE")]
  pub fn write_u_int16_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 2)?;
    wr_u16(&mut self.inner, o, value, true);
    Ok((o + 2) as u32)
  }
  #[napi(js_name = "writeUInt16BE")]
  pub fn write_u_int16_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 2)?;
    wr_u16(&mut self.inner, o, value, false);
    Ok((o + 2) as u32)
  }
  #[napi(js_name = "writeUint16LE")]
  pub fn write_uint16_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_le(value, offset)
  }
  #[napi(js_name = "writeUint16BE")]
  pub fn write_uint16_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_be(value, offset)
  }

  #[napi(js_name = "writeInt32LE")]
  pub fn write_int32_le(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    wr_i32(&mut self.inner, o, value, true);
    Ok((o + 4) as u32)
  }
  #[napi(js_name = "writeInt32BE")]
  pub fn write_int32_be(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    wr_i32(&mut self.inner, o, value, false);
    Ok((o + 4) as u32)
  }
  #[napi(js_name = "writeUInt32LE")]
  pub fn write_u_int32_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    wr_u32(&mut self.inner, o, value, true);
    Ok((o + 4) as u32)
  }
  #[napi(js_name = "writeUInt32BE")]
  pub fn write_u_int32_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    wr_u32(&mut self.inner, o, value, false);
    Ok((o + 4) as u32)
  }
  #[napi(js_name = "writeUint32LE")]
  pub fn write_uint32_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_le(value, offset)
  }
  #[napi(js_name = "writeUint32BE")]
  pub fn write_uint32_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_be(value, offset)
  }

  #[napi(js_name = "writeFloatLE")]
  pub fn write_float_le(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    wr_f32(&mut self.inner, o, value, true);
    Ok((o + 4) as u32)
  }
  #[napi(js_name = "writeFloatBE")]
  pub fn write_float_be(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 4)?;
    wr_f32(&mut self.inner, o, value, false);
    Ok((o + 4) as u32)
  }
  #[napi(js_name = "writeDoubleLE")]
  pub fn write_double_le(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    wr_f64(&mut self.inner, o, value, true);
    Ok((o + 8) as u32)
  }
  #[napi(js_name = "writeDoubleBE")]
  pub fn write_double_be(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    wr_f64(&mut self.inner, o, value, false);
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeBigInt64LE")]
  pub fn write_big_int64_le(&mut self, value: JsBigInt, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    let (v, _) = value.get_i64();
    wr_i64(&mut self.inner, o, v, true);
    Ok((o + 8) as u32)
  }
  #[napi(js_name = "writeBigInt64BE")]
  pub fn write_big_int64_be(&mut self, value: JsBigInt, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    let (v, _) = value.get_i64();
    wr_i64(&mut self.inner, o, v, false);
    Ok((o + 8) as u32)
  }
  #[napi(js_name = "writeBigUInt64LE")]
  pub fn write_big_u_int64_le(&mut self, value: JsBigInt, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    let (_, v, _) = value.get_u64();
    wr_u64(&mut self.inner, o, v, true);
    Ok((o + 8) as u32)
  }
  #[napi(js_name = "writeBigUInt64BE")]
  pub fn write_big_u_int64_be(&mut self, value: JsBigInt, offset: Option<u32>) -> Result<u32> {
    let o = offset.unwrap_or(0) as usize;
    rd(&self.inner, o, 8)?;
    let (_, v, _) = value.get_u64();
    wr_u64(&mut self.inner, o, v, false);
    Ok((o + 8) as u32)
  }
  #[napi(js_name = "writeBigUint64LE")]
  pub fn write_big_uint64_le(&mut self, value: JsBigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_le(value, offset)
  }
  #[napi(js_name = "writeBigUint64BE")]
  pub fn write_big_uint64_be(&mut self, value: JsBigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_be(value, offset)
  }

  #[napi(js_name = "writeUIntLE")]
  pub fn write_u_int_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    let o = offset as usize;
    let l = byte_length as usize;
    if l < 1 || l > 6 {
      return Err(Error::new(
        Status::InvalidArg,
        "The value of \"byteLength\" is out of range",
      ));
    }
    rd(&self.inner, o, l)?;
    let un = if value < 0.0 {
      (value as i64 + (1i64 << (8 * l))) as u64
    } else {
      value as u64
    };
    write_le(&mut self.inner, o, l, un);
    Ok((o + l) as u32)
  }
  #[napi(js_name = "writeUintLE")]
  pub fn write_uint_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_le(value, offset, byte_length)
  }
  #[napi(js_name = "writeUIntBE")]
  pub fn write_u_int_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    let o = offset as usize;
    let l = byte_length as usize;
    if l < 1 || l > 6 {
      return Err(Error::new(
        Status::InvalidArg,
        "The value of \"byteLength\" is out of range",
      ));
    }
    rd(&self.inner, o, l)?;
    let un = if value < 0.0 {
      (value as i64 + (1i64 << (8 * l))) as u64
    } else {
      value as u64
    };
    write_be(&mut self.inner, o, l, un);
    Ok((o + l) as u32)
  }
  #[napi(js_name = "writeUintBE")]
  pub fn write_uint_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_be(value, offset, byte_length)
  }
  #[napi(js_name = "writeIntLE")]
  pub fn write_int_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_le(value, offset, byte_length)
  }
  #[napi(js_name = "writeIntBE")]
  pub fn write_int_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_be(value, offset, byte_length)
  }
}

// ===========================================================================
// Blob / File
// ===========================================================================

#[napi(object)]
pub struct BlobOptions {
  pub endings: Option<String>,
  #[napi(js_name = "type")]
  pub r#type: Option<String>,
}

#[napi(object)]
pub struct FileOptions {
  pub endings: Option<String>,
  #[napi(js_name = "type")]
  pub r#type: Option<String>,
  pub last_modified: Option<f64>,
}

fn collect_blob_parts(env: &Env, sources: Vec<Unknown>) -> Result<Vec<u8>> {
  let mut out = Vec::new();
  for src in sources {
    let raw = raw_of(&src);
    if let Ok(s) = unsafe { String::from_napi_value(env.raw(), raw) } {
      out.extend_from_slice(s.as_bytes());
      continue;
    }
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
  #[napi(js_name = "arrayBuffer")]
  pub fn array_buffer(&self, env: Env) -> Result<ArrayBuffer<'_>> {
    ArrayBuffer::from_data(&env, self.data.clone())
  }

  /// Returns a copy of the Blob's data as a AlloyBuffer.
  #[napi]
  pub fn bytes(&self) -> AlloyBuffer {
    AlloyBuffer::from_vec(self.data.clone())
  }

  /// Returns the Blob's data decoded as UTF-8.
  #[napi]
  pub fn text(&self) -> String {
    String::from_utf8(self.data.clone()).unwrap_or_default()
  }

  /// Creates a new Blob containing a subset of this Blob's data.
  #[napi]
  pub fn slice(&self, start: Option<i64>, end: Option<i64>, content_type: Option<String>) -> Blob {
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
    let r#type = content_type.unwrap_or_else(|| self.r#type.clone());
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
  #[napi(js_name = "arrayBuffer")]
  pub fn array_buffer(&self, env: Env) -> Result<ArrayBuffer<'_>> {
    ArrayBuffer::from_data(&env, self.data.clone())
  }

  /// Returns a copy of the File's data as a AlloyBuffer.
  #[napi]
  pub fn bytes(&self) -> AlloyBuffer {
    AlloyBuffer::from_vec(self.data.clone())
  }

  /// Returns the File's data decoded as UTF-8.
  #[napi]
  pub fn text(&self) -> String {
    String::from_utf8(self.data.clone()).unwrap_or_default()
  }

  /// Creates a new File containing a subset of this File's data.
  #[napi]
  pub fn slice(&self, start: Option<i64>, end: Option<i64>, content_type: Option<String>) -> File {
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
      r#type: content_type.unwrap_or_else(|| self.r#type.clone()),
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
#[napi]
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
// Native helper functions used by `src/buffer.ts`
// ===========================================================================

#[napi(js_name = "bufIsEncoding")]
pub fn buf_is_encoding(encoding: String) -> bool {
  SUPPORTED_ENCODINGS.contains(&encoding.to_ascii_lowercase().as_str())
}

#[napi(js_name = "bufByteLength")]
pub fn buf_byte_length(env: Env, input: Unknown, encoding: Option<String>) -> Result<u32> {
  let raw = raw_of(&input);
  if let Ok(s) = unsafe { String::from_napi_value(env.raw(), raw) } {
    return Ok(str_to_bytes(&s, &encoding.unwrap_or_else(|| "utf8".to_string()))?.len() as u32);
  }
  Ok(get_bytes(&env, raw)?.len() as u32)
}

#[napi(js_name = "bufCompare")]
pub fn buf_compare(env: Env, buf1: Unknown, buf2: Unknown) -> Result<i32> {
  let b1 = get_bytes(&env, raw_of(&buf1))?;
  let b2 = get_bytes(&env, raw_of(&buf2))?;
  Ok(b1.cmp(&b2) as i32)
}

#[napi(js_name = "bufConcat")]
pub fn buf_concat(env: Env, list: Vec<Unknown>, total_length: Option<u32>) -> Result<JsBuffer> {
  let mut bufs = Vec::with_capacity(list.len());
  for item in list {
    bufs.push(get_bytes(&env, raw_of(&item))?);
  }
  let total = total_length
    .unwrap_or_else(|| bufs.iter().map(|b| b.len() as u32).sum()) as usize;
  let mut out = Vec::with_capacity(total);
  for b in &bufs {
    out.extend_from_slice(b.as_slice());
  }
  out.resize(total, 0);
  Ok(out.into())
}

#[napi(js_name = "bufCopyBytesFrom")]
pub fn buf_copy_bytes_from(
  env: Env,
  view: Unknown,
  offset: Option<u32>,
  length: Option<u32>,
) -> Result<JsBuffer> {
  let raw = raw_of(&view);
  let mut is_ta = false;
  check_status!(unsafe { sys::napi_is_typedarray(env.raw(), raw, &mut is_ta) })?;
  if is_ta {
    let (ptr, elem_count, boff, elem) = unsafe { inspect_typedarray(&env, raw)? };
    let off = offset.unwrap_or(0) as usize;
    let cnt = length
      .unwrap_or((elem_count - off) as u32)
      .min((elem_count - off) as u32) as usize;
    let total = boff + elem_count * elem;
    let slice = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), total) };
    let start = boff + off * elem;
    let end = (boff + (off + cnt) * elem).min(total);
    return Ok(slice[start..end].to_vec().into());
  }
  let mut is_dv = false;
  check_status!(unsafe { sys::napi_is_dataview(env.raw(), raw, &mut is_dv) })?;
  if is_dv {
    let (ptr, len) = unsafe { dataview_bytes(&env, raw)? };
    let bytes = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) }.to_vec();
    return Ok(bytes.into());
  }
  let mut is_ab = false;
  check_status!(unsafe { sys::napi_is_arraybuffer(env.raw(), raw, &mut is_ab) })?;
  if is_ab {
    let ab = unsafe { ArrayBuffer::from_napi_value(env.raw(), raw) }?;
    return Ok(ab.to_vec().into());
  }
  let bytes = get_bytes(&env, raw)?;
  Ok(bytes.into())
}

#[napi(js_name = "bufDecode")]
pub fn buf_decode(_env: Env, value: String, encoding: Option<String>) -> Result<JsBuffer> {
  let enc = encoding.unwrap_or_else(|| "utf8".to_string());
  let bytes = str_to_bytes(&value, &enc)?;
  Ok(bytes.into())
}

#[napi(js_name = "bufEncode")]
pub fn buf_encode(
  env: Env,
  buffer: Unknown,
  encoding: Option<String>,
  start: Option<u32>,
  end: Option<u32>,
) -> Result<String> {
  let bytes = get_bytes(&env, raw_of(&buffer))?;
  let enc = encoding.unwrap_or_else(|| "utf8".to_string());
  let s = (start.unwrap_or(0) as usize).min(bytes.len());
  let e = (end.unwrap_or(bytes.len() as u32) as usize).min(bytes.len());
  if s > e {
    return Err(Error::new(Status::InvalidArg, "invalid start/end range"));
  }
  bytes_to_str(&bytes[s..e], &enc)
}

#[napi(js_name = "bufWrite")]
pub fn buf_write(
  env: Env,
  buffer: Unknown,
  string: String,
  offset: Option<u32>,
  length: Option<u32>,
  encoding: Option<String>,
) -> Result<u32> {
  let raw = raw_of(&buffer);
  let mut is_ta = false;
  check_status!(unsafe { sys::napi_is_typedarray(env.raw(), raw, &mut is_ta) })?;
  if !is_ta {
    let mut is_buf = false;
    check_status!(unsafe { sys::napi_is_buffer(env.raw(), raw, &mut is_buf) })?;
    if !is_buf {
      return Err(Error::new(Status::InvalidArg, "expected a Buffer or TypedArray"));
    }
  }
  let (ptr, len, boff, elem) = unsafe { inspect_typedarray(&env, raw)? };
  let total_bytes = len * elem;
  let bytes = str_to_bytes(&string, encoding.as_deref().unwrap_or("utf8"))?;
  let o = (offset.unwrap_or(0) as usize).min(total_bytes);
  let max = total_bytes - o;
  let want = (length.unwrap_or(total_bytes as u32) as usize).min(bytes.len());
  let n = want.min(max);
  if n > 0 {
    let target_slice = unsafe { std::slice::from_raw_parts_mut(ptr.as_ptr().add(boff), total_bytes) };
    target_slice[o..o + n].copy_from_slice(&bytes[..n]);
  }
  Ok(n as u32)
}

#[napi(js_name = "bufFill")]
pub fn buf_fill(
  env: Env,
  buffer: Unknown,
  value: Unknown,
  offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<()> {
  let raw = raw_of(&buffer);
  let (ptr, len, boff, elem) = unsafe { inspect_typedarray(&env, raw)? };
  let total_bytes = len * elem;
  let s = (offset.unwrap_or(0) as usize).min(total_bytes);
  let e = (end.unwrap_or(total_bytes as u32) as usize).min(total_bytes);
  if s > e {
    return Err(Error::new(Status::InvalidArg, "invalid start/end range"));
  }
  let val_raw = raw_of(&value);
  let fill_bytes = if let Ok(n) = unsafe { f64::from_napi_value(env.raw(), val_raw) } {
    vec![((n as i32) & 0xff) as u8]
  } else if let Ok(s2) = unsafe { String::from_napi_value(env.raw(), val_raw) } {
    let enc2 = encoding.unwrap_or_else(|| "utf8".to_string());
    str_to_bytes(&s2, &enc2)?
  } else {
    get_bytes(&env, val_raw)?
  };
  let target_slice = unsafe { std::slice::from_raw_parts_mut(ptr.as_ptr().add(boff), total_bytes) };
  fill_with(&mut target_slice[s..e], &fill_bytes);
  Ok(())
}

#[napi(js_name = "bufIndexOf")]
pub fn buf_index_of(
  env: Env,
  buffer: Unknown,
  value: Unknown,
  byte_offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<i32> {
  let bytes = get_bytes(&env, raw_of(&buffer))?;
  let len = bytes.len();
  let o = (byte_offset.unwrap_or(0) as usize).min(len);
  let e = (end.unwrap_or(len as u32) as usize).min(len);
  let pat = search_pattern(&env, &value, encoding)?;
  Ok(search_from(&bytes, o, e, &pat, false))
}

#[napi(js_name = "bufLastIndexOf")]
pub fn buf_last_index_of(
  env: Env,
  buffer: Unknown,
  value: Unknown,
  byte_offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<i32> {
  let bytes = get_bytes(&env, raw_of(&buffer))?;
  let len = bytes.len();
  let o = (byte_offset.unwrap_or(0) as usize).min(len);
  let e = (end.unwrap_or(len as u32) as usize).min(len);
  let pat = search_pattern(&env, &value, encoding)?;
  Ok(search_from(&bytes, o, e, &pat, true))
}

#[napi(js_name = "bufIncludes")]
pub fn buf_includes(
  env: Env,
  buffer: Unknown,
  value: Unknown,
  byte_offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<bool> {
  Ok(buf_index_of(env, buffer, value, byte_offset, end, encoding)? >= 0)
}
