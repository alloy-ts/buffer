//! NAPI bindings for a `node:buffer`-compatible API.
//!
//! This module mirrors the public surface of Node's `node:buffer` module, but
//! it does **not** export a `Buffer` *class*. A `Buffer` must be a `Uint8Array`
//! subclass to be faithful to Node, and napi-rs cannot subclass `Uint8Array`
//! from Rust. Instead the JS layer (`src/buffer.ts`) defines
//! `class Buffer extends Uint8Array` and calls into the stateless helpers below,
//! which operate on a `Uint8Array`/`Buffer`'s underlying bytes.
//!
//! * `bufDecode` / `bufEncode` — `Buffer.from(string)` and `buf.toString(enc)`.
//! * `bufConcat`, `bufCompare`, `bufByteLength`, `bufStringLength`,
//!   `bufIsEncoding`, `bufCopyBytesFrom` — the remaining `Buffer` statics.
//! * `bufFill`, `bufWrite`, `bufIndexOf`, `bufLastIndexOf`, `bufIncludes` —
//!   the instance methods whose encoding/truncation rules are easier in Rust.
//! * The `Blob` and `File` classes (no `Uint8Array` subclassing needed).
//! * Module-level helpers: `atob`, `btoa`, `isAscii`, `isUtf8`, `isLatin1`,
//!   `transcode`, `resolveObjectURL`, the `constants` object and the
//!   `kMaxLength` / `kStringMaxLength` / `INSPECT_MAX_BYTES` / `poolSize`
//!   properties.
//!
//! ## NAPI mapping notes
//! * napi-rs v3 does not implement `FromNapiValue` for `u64`/`i64`, so the
//!   64-bit methods take/return `BigInt`. The 48-bit `read/writeUIntBE/LE` /
//!   `IntBE/LE` are implemented in JS (see `src/buffer.ts`) for exactness.
//! * `ArrayBuffer`, `TypedArray` and `DataView` are accepted as inputs and
//!   `ArrayBuffer` / `Buffer` are produced as outputs, so this module
//!   interoperates with the rest of the TypedArray ecosystem.

use std::ptr::{self, NonNull};

use napi::bindgen_prelude::{
  ArrayBuffer, Env, FromNapiValue, JsValue, Object, Unknown,
};
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

/// Node decodes invalid UTF-8 to the Unicode replacement character U+FFFD, so
/// we use the lossy conversion rather than erroring out.
fn bytes_to_str(b: &[u8], enc: &str) -> Result<String> {
  match enc.to_ascii_lowercase().as_str() {
    "utf8" | "utf-8" => Ok(String::from_utf8_lossy(b).into_owned()),
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
    let buf = unsafe { napi::bindgen_prelude::JsBuffer::from_napi_value(env.raw(), v) }?;
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
    let ab = unsafe { ArrayBuffer::from_napi_value(env.raw(), v) }?;
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
// `Buffer` statics / instance helpers (operate on `JsBuffer`).
// ===========================================================================

use napi::bindgen_prelude::JsBuffer;

/// `Buffer.from(value[, arg1[, arg2]])` — the string overload returns a
/// `JsBuffer`; the JS class handles ArrayBuffer / TypedArray / Buffer / array
/// overloads itself (they only need to share or copy bytes).
#[napi(js_name = "bufDecode")]
pub fn buf_decode(env: Env, value: Unknown, arg1: Option<Unknown>, arg2: Option<Unknown>) -> Result<JsBuffer> {
  let raw = raw_of(&value);
  if let Ok(s) = unsafe { String::from_napi_value(env.raw(), raw) } {
    let enc = arg1
      .and_then(|u| unknown_to_string(&env, &u))
      .unwrap_or_else(|| "utf8".to_string());
    return Ok(JsBuffer::from(str_to_bytes(&s, &enc)?));
  }
  let mut is_ta = false;
  check_status!(unsafe { sys::napi_is_typedarray(env.raw(), raw, &mut is_ta) })?;
  if is_ta {
    return Ok(JsBuffer::from(typedarray_low_bytes(&env, raw)?));
  }
  let mut is_dv = false;
  check_status!(unsafe { sys::napi_is_dataview(env.raw(), raw, &mut is_dv) })?;
  if is_dv {
    let (ptr, len) = unsafe { dataview_bytes(&env, raw)? };
    return Ok(JsBuffer::from(
      unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) }.to_vec(),
    ));
  }
  let mut is_ab = false;
  check_status!(unsafe { sys::napi_is_arraybuffer(env.raw(), raw, &mut is_ab) })?;
  if is_ab {
    let ab = unsafe { ArrayBuffer::from_napi_value(env.raw(), raw) }?;
    let total = ab.len();
    let off = arg1
      .as_ref()
      .and_then(|u| unknown_to_usize(&env, u))
      .unwrap_or(0);
    let off = off.min(total);
    let len = arg2
      .as_ref()
      .and_then(|u| unknown_to_usize(&env, u))
      .unwrap_or(total - off);
    let len = len.min(total - off);
    return Ok(JsBuffer::from(ab[off..off + len].to_vec()));
  }
  let mut is_buf = false;
  check_status!(unsafe { sys::napi_is_buffer(env.raw(), raw, &mut is_buf) })?;
  if is_buf {
    let buf = unsafe { JsBuffer::from_napi_value(env.raw(), raw) }?;
    return Ok(JsBuffer::from(buf.as_ref().to_vec()));
  }
  let mut is_arr = false;
  check_status!(unsafe { sys::napi_is_array(env.raw(), raw, &mut is_arr) })?;
  if is_arr {
    let arr: Vec<Unknown> = unsafe { Vec::<Unknown>::from_napi_value(env.raw(), raw) }?;
    let data: Vec<u8> = arr.iter().map(|u| unknown_to_byte(&env, u)).collect();
    return Ok(JsBuffer::from(data));
  }
  Err(Error::new(
    Status::InvalidArg,
    "Buffer.from: unsupported value",
  ))
}

/// `buf.toString(encoding[, start[, end]])`.
#[napi(js_name = "bufEncode")]
pub fn buf_encode(
  buf: JsBuffer,
  encoding: Option<String>,
  start: Option<u32>,
  end: Option<u32>,
) -> Result<String> {
  let data = buf.as_ref();
  let enc = encoding.unwrap_or_else(|| "utf8".to_string());
  let s = start.unwrap_or(0) as usize;
  let e = end.unwrap_or(data.len() as u32) as usize;
  if s > e || e > data.len() {
    return Err(Error::new(
      Status::InvalidArg,
      "invalid start/end range",
    ));
  }
  bytes_to_str(&data[s..e], &enc)
}

/// `Buffer.concat(list[, totalLength])`.
#[napi(js_name = "bufConcat")]
pub fn buf_concat(list: Vec<JsBuffer>, total_length: Option<u32>) -> JsBuffer {
  let total = total_length.unwrap_or_else(|| {
    list.iter().map(|b| b.as_ref().len() as u32).sum()
  }) as usize;
  let mut out = Vec::with_capacity(total);
  for b in &list {
    out.extend_from_slice(b.as_ref());
  }
  out.truncate(total);
  JsBuffer::from(out)
}

/// `Buffer.compare(buf1, buf2)`.
#[napi(js_name = "bufCompare")]
pub fn buf_compare(buf1: JsBuffer, buf2: JsBuffer) -> i32 {
  buf1.as_ref().cmp(buf2.as_ref()) as i32
}

/// `Buffer.byteLength(value[, encoding])`.
#[napi(js_name = "bufByteLength")]
pub fn buf_byte_length(env: Env, value: Unknown, encoding: Option<String>) -> Result<u32> {
  let raw = raw_of(&value);
  if let Ok(s) = unsafe { String::from_napi_value(env.raw(), raw) } {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    return Ok(str_to_bytes(&s, &enc)?.len() as u32);
  }
  Ok(get_bytes(&env, raw)?.len() as u32)
}

/// `Buffer.stringLength(input[, encoding])`.
#[napi(js_name = "bufStringLength")]
pub fn buf_string_length(env: Env, input: Unknown, encoding: Option<String>) -> Result<u32> {
  let bytes = get_bytes(&env, raw_of(&input))?;
  let enc = encoding.unwrap_or_else(|| "utf8".to_string());
  let s = bytes_to_str(&bytes, &enc)?;
  Ok(s.chars().count() as u32)
}

/// `Buffer.isEncoding(encoding)`.
#[napi(js_name = "bufIsEncoding")]
pub fn buf_is_encoding(encoding: String) -> bool {
  SUPPORTED_ENCODINGS.contains(&encoding.to_ascii_lowercase().as_str())
}

/// `Buffer.copyBytesFrom(view[, offset[, length]])`.
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
      .min((elem_count - off) as u32)
      as usize;
    let total = boff + elem_count * elem;
    let slice = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), total) };
    let start = boff + off * elem;
    let end = (boff + (off + cnt) * elem).min(total);
    return Ok(JsBuffer::from(slice[start..end].to_vec()));
  }
  let buf = unsafe { JsBuffer::from_napi_value(env.raw(), raw) }?;
  Ok(JsBuffer::from(buf.as_ref().to_vec()))
}

// --- fill ----------------------------------------------------------------

/// `buf.fill(value[, start[, end]][, encoding])`.
#[napi(js_name = "bufFill")]
pub fn buf_fill(
  env: Env,
  mut buf: JsBuffer,
  value: Unknown,
  start: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<()> {
  let data = buf.as_mut();
  let s = (start.unwrap_or(0) as usize).min(data.len());
  let e = (end.unwrap_or(data.len() as u32) as usize).min(data.len());
  if s > e {
    return Err(Error::new(Status::InvalidArg, "invalid start/end range"));
  }
  let raw = raw_of(&value);
  if let Ok(n) = unsafe { f64::from_napi_value(env.raw(), raw) } {
    let byte = ((n as i32) & 0xff) as u8;
    data[s..e].iter_mut().for_each(|x| *x = byte);
  } else if let Ok(s2) = unsafe { String::from_napi_value(env.raw(), raw) } {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let bytes = str_to_bytes(&s2, &enc)?;
    if bytes.is_empty() {
      data[s..e].iter_mut().for_each(|x| *x = 0);
    } else {
      for (i, slot) in data[s..e].iter_mut().enumerate() {
        *slot = bytes[i % bytes.len()];
      }
    }
  } else {
    let bytes = get_bytes(&env, raw)?;
    if bytes.is_empty() {
      data[s..e].iter_mut().for_each(|x| *x = 0);
    } else {
      for (i, slot) in data[s..e].iter_mut().enumerate() {
        *slot = bytes[i % bytes.len()];
      }
    }
  }
  Ok(())
}

// --- write ---------------------------------------------------------------

/// `buf.write(string[, offset[, length]][, encoding])`.
#[napi(js_name = "bufWrite")]
pub fn buf_write(
  env: Env,
  mut buf: JsBuffer,
  string: String,
  offset: Option<u32>,
  length: Option<u32>,
  encoding: Option<String>,
) -> Result<u32> {
  let data = buf.as_mut();
  let enc = encoding.unwrap_or_else(|| "utf8".to_string());
  let bytes = str_to_bytes(&string, &enc)?;
  let o = (offset.unwrap_or(0) as usize).min(data.len());
  let max = data.len() - o;
  let want = (length.unwrap_or(bytes.len() as u32) as usize).min(bytes.len());
  let n = want.min(max);
  if n > 0 {
    data[o..o + n].copy_from_slice(&bytes[..n]);
  }
  Ok(n as u32)
}

// --- search --------------------------------------------------------------

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

/// `buf.indexOf(value[, byteOffset[, end]][, encoding])`.
#[napi(js_name = "bufIndexOf")]
pub fn buf_index_of(
  env: Env,
  buf: JsBuffer,
  value: Unknown,
  byte_offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<i32> {
  let data = buf.as_ref();
  let pat = search_pattern(&env, &value, encoding)?;
  let o = (byte_offset.unwrap_or(0) as usize).min(data.len());
  let e = (end.unwrap_or(data.len() as u32) as usize).min(data.len());
  Ok(search_from(data, o, e, &pat, false))
}

/// `buf.lastIndexOf(value[, byteOffset[, end]][, encoding])`.
#[napi(js_name = "bufLastIndexOf")]
pub fn buf_last_index_of(
  env: Env,
  buf: JsBuffer,
  value: Unknown,
  byte_offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<i32> {
  let data = buf.as_ref();
  let pat = search_pattern(&env, &value, encoding)?;
  let o = (byte_offset.unwrap_or(0) as usize).min(data.len());
  let e = (end.unwrap_or(data.len() as u32) as usize).min(data.len());
  Ok(search_from(data, o, e, &pat, true))
}

/// `buf.includes(value[, byteOffset[, end]][, encoding])`.
#[napi(js_name = "bufIncludes")]
pub fn buf_includes(
  env: Env,
  buf: JsBuffer,
  value: Unknown,
  byte_offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<bool> {
  Ok(buf_index_of(env, buf, value, byte_offset, end, encoding)? >= 0)
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
  pub fn array_buffer(&self, env: Env) -> Result<ArrayBuffer> {
    ArrayBuffer::from_data(&env, self.data.clone())
  }

  /// Returns a copy of the Blob's data as a Buffer.
  #[napi]
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
  pub fn slice(&self, start: Option<i64>, end: Option<i64>, r#type: Option<String>) -> Blob {
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
    let r#type = r#type.unwrap_or_else(|| self.r#type.clone());
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
  pub fn array_buffer(&self, env: Env) -> Result<ArrayBuffer> {
    ArrayBuffer::from_data(&env, self.data.clone())
  }

  /// Returns a copy of the File's data as a Buffer.
  #[napi]
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
  pub fn slice(&self, start: Option<i64>, end: Option<i64>, r#type: Option<String>) -> File {
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
      r#type: r#type.unwrap_or_else(|| self.r#type.clone()),
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
