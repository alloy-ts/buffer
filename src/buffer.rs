//! NAPI bindings for a `node:buffer`-compatible API.
//!
//! This module provides native performance-critical functions for the
//! `node:buffer` implementation in `src/buffer.ts`.

use std::ffi::CString;
use std::ptr::{self, NonNull};

use napi::bindgen_prelude::{
  ArrayBuffer, Buffer as JsBuffer, Env, FromNapiValue, JsValue, Object, Unknown,
};
use napi::sys;
use napi::{check_status, Error, Result, Status};
use napi_derive::napi;

fn raw_of(u: &Unknown) -> sys::napi_value {
  JsValue::value(u).value
}

// ===========================================================================
// Constants
// ===========================================================================

const MAX_LENGTH: u32 = 0x7fff_ffff;
const MAX_STRING_LENGTH: u32 = 0x1ffff_fe8;
const INSPECT_MAX_BYTES_VALUE: u32 = 50;
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

fn bytes_to_str(b: &[u8], enc: &str) -> Result<String> {
  match enc.to_ascii_lowercase().as_str() {
    "utf8" | "utf-8" => Ok(String::from_utf8_lossy(b).into_owned()),
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
    "expected a Buffer, TypedArray, DataView or ArrayBuffer",
  ))
}

unsafe fn get_bytes_mut(env: &Env, v: sys::napi_value) -> Result<(&'static mut [u8], usize)> {
  let mut b = false;
  check_status!(sys::napi_is_buffer(env.raw(), v, &mut b))?;
  if b {
    let mut data = ptr::null_mut();
    let mut len = 0usize;
    check_status!(sys::napi_get_buffer_info(env.raw(), v, &mut data, &mut len))?;
    let ptr = NonNull::new(data as *mut u8).unwrap_or(NonNull::dangling());
    return Ok((std::slice::from_raw_parts_mut(ptr.as_ptr(), len), len));
  }
  let mut t = false;
  check_status!(sys::napi_is_typedarray(env.raw(), v, &mut t))?;
  if t {
    let (ptr, len, boff, elem) = inspect_typedarray(env, v)?;
    let total = boff + len * elem;
    let full_slice = std::slice::from_raw_parts_mut(ptr.as_ptr(), total);
    return Ok((&mut full_slice[boff..boff + len * elem], len * elem));
  }
  let mut d = false;
  check_status!(sys::napi_is_dataview(env.raw(), v, &mut d))?;
  if d {
    let (ptr, len) = dataview_bytes(env, v)?;
    return Ok((std::slice::from_raw_parts_mut(ptr.as_ptr(), len), len));
  }
  let mut a = false;
  check_status!(sys::napi_is_arraybuffer(env.raw(), v, &mut a))?;
  if a {
    let mut data = ptr::null_mut();
    let mut len = 0usize;
    check_status!(sys::napi_get_arraybuffer_info(env.raw(), v, &mut data, &mut len))?;
    let ptr = NonNull::new(data as *mut u8).unwrap_or(NonNull::dangling());
    return Ok((std::slice::from_raw_parts_mut(ptr.as_ptr(), len), len));
  }
  Err(Error::new(
    Status::InvalidArg,
    "expected a Buffer, TypedArray, DataView or ArrayBuffer",
  ))
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

// ===========================================================================
// Exported Native Functions for JS Buffer Wrapper
// ===========================================================================

#[napi(js_name = "bufIsEncoding")]
pub fn buf_is_encoding(encoding: String) -> bool {
  SUPPORTED_ENCODINGS.contains(&encoding.to_ascii_lowercase().as_str())
}

#[napi(js_name = "bufByteLength")]
pub fn buf_byte_length(env: Env, value: Unknown, encoding: Option<String>) -> Result<u32> {
  let raw = raw_of(&value);
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
  let mut bytes_list = Vec::with_capacity(list.len());
  for item in list {
    bytes_list.push(get_bytes(&env, raw_of(&item))?);
  }
  let total = total_length
    .map(|n| n as usize)
    .unwrap_or_else(|| bytes_list.iter().map(|b| b.len()).sum());
  let mut out = Vec::with_capacity(total);
  for b in &bytes_list {
    if out.len() >= total {
      break;
    }
    let take = (total - out.len()).min(b.len());
    out.extend_from_slice(&b[..take]);
  }
  out.resize(total, 0);
  Ok(JsBuffer::from(out))
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
    if off > elem_count {
      return Err(Error::new(Status::InvalidArg, "offset out of bounds"));
    }
    let cnt = length
      .map(|n| n as usize)
      .unwrap_or(elem_count - off)
      .min(elem_count - off);
    let total_bytes = boff + elem_count * elem;
    let slice = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), total_bytes) };
    let start_byte = boff + off * elem;
    let bytes_to_copy = cnt * elem;
    let end_byte = start_byte + bytes_to_copy;
    return Ok(JsBuffer::from(slice[start_byte..end_byte].to_vec()));
  }
  let mut is_dv = false;
  check_status!(unsafe { sys::napi_is_dataview(env.raw(), raw, &mut is_dv) })?;
  if is_dv {
    let (ptr, len) = unsafe { dataview_bytes(&env, raw)? };
    let off = offset.unwrap_or(0) as usize;
    let cnt = length.map(|n| n as usize).unwrap_or(len.saturating_sub(off)).min(len.saturating_sub(off));
    let slice = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), len) };
    return Ok(JsBuffer::from(slice[off..off + cnt].to_vec()));
  }
  let mut is_ab = false;
  check_status!(unsafe { sys::napi_is_arraybuffer(env.raw(), raw, &mut is_ab) })?;
  if is_ab {
    let ab = unsafe { ArrayBuffer::from_napi_value(env.raw(), raw) }?;
    let slice = ab.to_vec();
    let off = offset.unwrap_or(0) as usize;
    let cnt = length.map(|n| n as usize).unwrap_or(slice.len().saturating_sub(off)).min(slice.len().saturating_sub(off));
    return Ok(JsBuffer::from(slice[off..off + cnt].to_vec()));
  }
  let bytes = get_bytes(&env, raw)?;
  let off = offset.unwrap_or(0) as usize;
  let cnt = length.map(|n| n as usize).unwrap_or(bytes.len().saturating_sub(off)).min(bytes.len().saturating_sub(off));
  Ok(JsBuffer::from(bytes[off..off + cnt].to_vec()))
}

#[napi(js_name = "bufDecode")]
pub fn buf_decode(string: String, encoding: Option<String>) -> Result<JsBuffer> {
  let enc = encoding.unwrap_or_else(|| "utf8".to_string());
  let bytes = str_to_bytes(&string, &enc)?;
  Ok(JsBuffer::from(bytes))
}

#[napi(js_name = "bufEncode")]
pub fn buf_encode(
  env: Env,
  buf: Unknown,
  encoding: Option<String>,
  start: Option<u32>,
  end: Option<u32>,
) -> Result<String> {
  let bytes = get_bytes(&env, raw_of(&buf))?;
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
  buf: Unknown,
  string: String,
  offset: Option<u32>,
  length: Option<u32>,
  encoding: Option<String>,
) -> Result<u32> {
  let enc = encoding.unwrap_or_else(|| "utf8".to_string());
  let bytes = str_to_bytes(&string, &enc)?;
  let (slice, buf_len) = unsafe { get_bytes_mut(&env, raw_of(&buf))? };
  let o = (offset.unwrap_or(0) as usize).min(buf_len);
  let max = buf_len - o;
  let want = length.map(|l| l as usize).unwrap_or(bytes.len()).min(bytes.len());
  let mut n = want.min(max);
  if enc.to_ascii_lowercase() == "utf8" || enc.to_ascii_lowercase() == "utf-8" {
    while n > 0 && (bytes[n - 1] & 0xc0) == 0x80 {
      n -= 1;
    }
    if n > 0 && (bytes[n - 1] & 0x80) != 0 {
      n -= 1;
    }
  }
  if n > 0 {
    slice[o..o + n].copy_from_slice(&bytes[..n]);
  }
  Ok(n as u32)
}

#[napi(js_name = "bufFill")]
pub fn buf_fill(
  env: Env,
  buf: Unknown,
  value: Unknown,
  offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<()> {
  let (slice, buf_len) = unsafe { get_bytes_mut(&env, raw_of(&buf))? };
  let s = (offset.unwrap_or(0) as usize).min(buf_len);
  let e = (end.unwrap_or(buf_len as u32) as usize).min(buf_len);
  if s > e {
    return Ok(());
  }
  let raw = raw_of(&value);
  if let Ok(n) = unsafe { f64::from_napi_value(env.raw(), raw) } {
    let byte = ((n as i32) & 0xff) as u8;
    for x in &mut slice[s..e] {
      *x = byte;
    }
  } else if let Ok(s2) = unsafe { String::from_napi_value(env.raw(), raw) } {
    let enc2 = encoding.unwrap_or_else(|| "utf8".to_string());
    let fill_bytes = str_to_bytes(&s2, &enc2)?;
    fill_with(&mut slice[s..e], &fill_bytes);
  } else {
    let fill_bytes = get_bytes(&env, raw)?;
    fill_with(&mut slice[s..e], &fill_bytes);
  }
  Ok(())
}

#[napi(js_name = "bufIndexOf")]
pub fn buf_index_of(
  env: Env,
  buf: Unknown,
  value: Unknown,
  byte_offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<i32> {
  let bytes = get_bytes(&env, raw_of(&buf))?;
  let o = (byte_offset.unwrap_or(0) as usize).min(bytes.len());
  let e = (end.unwrap_or(bytes.len() as u32) as usize).min(bytes.len());
  let pat = search_pattern(&env, &value, encoding)?;
  Ok(search_from(&bytes, o, e, &pat, false))
}

#[napi(js_name = "bufLastIndexOf")]
pub fn buf_last_index_of(
  env: Env,
  buf: Unknown,
  value: Unknown,
  byte_offset: Option<u32>,
  end: Option<u32>,
  encoding: Option<String>,
) -> Result<i32> {
  let bytes = get_bytes(&env, raw_of(&buf))?;
  let o = (byte_offset.unwrap_or(0) as usize).min(bytes.len());
  let e = (end.unwrap_or(bytes.len() as u32) as usize).min(bytes.len());
  let pat = search_pattern(&env, &value, encoding)?;
  Ok(search_from(&bytes, o, e, &pat, true))
}

#[napi(js_name = "bufIncludes")]
pub fn buf_includes(
  env: Env,
  buf: Unknown,
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
  #[napi(constructor)]
  pub fn new(env: Env, sources: Vec<Unknown>, options: Option<BlobOptions>) -> Result<Blob> {
    let data = collect_blob_parts(&env, sources)?;
    let r#type = options.and_then(|o| o.r#type).unwrap_or_default();
    Ok(Blob { data, r#type })
  }

  #[napi(js_name = "arrayBuffer")]
  pub fn array_buffer(&self, env: Env) -> Result<ArrayBuffer<'_>> {
    ArrayBuffer::from_data(&env, self.data.clone())
  }

  #[napi]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.clone())
  }

  #[napi]
  pub fn text(&self) -> String {
    String::from_utf8(self.data.clone()).unwrap_or_default()
  }

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

  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.data.len() as u32
  }

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

  #[napi(js_name = "arrayBuffer")]
  pub fn array_buffer(&self, env: Env) -> Result<ArrayBuffer<'_>> {
    ArrayBuffer::from_data(&env, self.data.clone())
  }

  #[napi]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.clone())
  }

  #[napi]
  pub fn text(&self) -> String {
    String::from_utf8(self.data.clone()).unwrap_or_default()
  }

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

  #[napi(getter)]
  pub fn name(&self) -> String {
    self.name.clone()
  }

  #[napi(getter, js_name = "lastModified")]
  pub fn last_modified(&self) -> f64 {
    self.last_modified
  }

  #[napi(getter, js_name = "type")]
  pub fn mime_type(&self) -> String {
    self.r#type.clone()
  }

  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.data.len() as u32
  }
}

// ===========================================================================
// Module-level helpers (`node:buffer` functions).
// ===========================================================================

#[napi]
pub fn atob(data: String) -> Result<String> {
  let bytes = decode_base64(&data, false)?;
  Ok(bytes.iter().map(|b| *b as char).collect())
}

#[napi]
pub fn btoa(data: String) -> String {
  encode_base64(data.as_bytes(), false)
}

#[napi(js_name = "isAscii")]
pub fn is_ascii(env: Env, input: Unknown) -> Result<bool> {
  Ok(get_bytes(&env, raw_of(&input))?.iter().all(|b| *b < 128))
}

#[napi(js_name = "isUtf8")]
pub fn is_utf8(env: Env, input: Unknown) -> Result<bool> {
  Ok(std::str::from_utf8(&get_bytes(&env, raw_of(&input))?).is_ok())
}

#[napi(js_name = "isLatin1")]
pub fn is_latin1(input: String) -> bool {
  input.chars().all(|c| (c as u32) <= 0xff)
}

#[napi]
pub fn transcode(env: Env, source: Unknown, from_encoding: String, to_encoding: String) -> Result<JsBuffer> {
  let bytes = get_bytes(&env, raw_of(&source))?;
  let intermediate = bytes_to_str(&bytes, &from_encoding)?;
  let out = str_to_bytes(&intermediate, &to_encoding)?;
  Ok(JsBuffer::from(out))
}

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
  #[napi(js_name = "MAX_LENGTH")]
  pub max_length: u32,
  #[napi(js_name = "MAX_STRING_LENGTH")]
  pub max_string_length: u32,
}

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
  exports.set("INSPECT_MAX_BYTES", INSPECT_MAX_BYTES_VALUE)?;
  exports.set("poolSize", POOL_SIZE)?;
  Ok(())
}

#[napi]
pub const constants: BufferConstants = BufferConstants {
  max_length: MAX_LENGTH,
  max_string_length: MAX_STRING_LENGTH,
};

#[napi]
pub const kMaxLength: u32 = MAX_LENGTH;

#[napi]
pub const kStringMaxLength: u32 = MAX_STRING_LENGTH;

#[napi]
pub const INSPECT_MAX_BYTES: u32 = INSPECT_MAX_BYTES_VALUE;

#[napi]
pub const poolSize: u32 = POOL_SIZE;
