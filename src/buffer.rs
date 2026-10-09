//! NAPI bindings for a `node:buffer`-compatible `Buffer` class.
//!
//! This module mirrors the parts of Node's `node:buffer` module that can be
//! expressed naturally from Rust:
//!
//! * The `Buffer` class — a fixed-length byte sequence backed by a `Vec<u8>`.
//!   It exposes the offset-based `read*` / `write*` integer and float methods
//!   (`readUInt8`, `writeInt16BE`, `readDoubleLE`, …), the `alloc` /
//!   `allocUnsafe` / `from` / `concat` / `isBuffer` / `isEncoding` statics,
//!   and the instance helpers `slice`, `subarray`, `fill`, `equals`,
//!   `compare`, `indexOf`, `includes`, `reverse`, `swap16/32/64`, `toString`,
//!   `toJSON`, `write` and a `length` getter.
//! * The `BufMut` write family — `put_u8`, `put_slice`, `put_u64` (…) — kept
//!   for the cursor/append model described in the `bytes::BufMut` trait. These
//!   grow the underlying `Vec<u8>` like `Vec<u8>`'s `BufMut` impl does.
//! * Module-level helpers: `atob`, `btoa`, `isAscii`, `isUtf8`, `transcode`
//!   and `constants`.
//!
//! NAPI mapping notes:
//! * napi-rs v3.13.0 does **not** implement `FromNapiValue` for `u64`/`i64`/
//!   `u128`/`i128`/`f32`, so the 64-/128-bit methods take/return `BigInt`
//!   (via `napi::bindgen_prelude::BigInt`) and the float methods take `f64`.
//! * `napi::bindgen_prelude::Buffer` is aliased to `JsBuffer`.
//! * Reads/writes always validate bounds and return a real JS error instead
//!   of panicking across the FFI boundary.
//! * Methods that hand data back (`slice`, `bytes`, …) return *copies* as
//!   `JsBuffer`; napi-rs cannot safely expose a zero-copy view.

use napi::bindgen_prelude::BigInt;
use napi::bindgen_prelude::Buffer as JsBuffer;
use napi::bindgen_prelude::Unknown;
use napi::{Error, Result, Status, Value};
use napi_derive::napi;

// ===========================================================================
// Encoding helpers (utf8 / ascii / latin1 / utf16le / hex / base64).
// ===========================================================================

fn str_to_bytes(s: &str, enc: &str) -> Result<Vec<u8>> {
  match enc.to_ascii_lowercase().as_str() {
    "utf8" | "utf-8" => Ok(s.as_bytes().to_vec()),
    "ascii" => {
      if s.bytes().any(|b| b > 127) {
        return Err(Error::new(Status::GenericFailure, "invalid ASCII string"));
      }
      Ok(s.as_bytes().to_vec())
    }
    "latin1" | "binary" => Ok(s.chars().map(|c| c as u8).collect()),
    "utf16le" | "ucs2" => {
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
      Status::GenericFailure,
      format!("unsupported encoding: {other}"),
    )),
  }
}

fn bytes_to_str(b: &[u8], enc: &str) -> Result<String> {
  match enc.to_ascii_lowercase().as_str() {
    "utf8" | "utf-8" => String::from_utf8(b.to_vec())
      .map_err(|_| Error::new(Status::GenericFailure, "invalid UTF-8 bytes")),
    "ascii" => {
      if b.iter().any(|x| *x > 127) {
        return Err(Error::new(Status::GenericFailure, "invalid ASCII bytes"));
      }
      Ok(b.iter().map(|x| *x as char).collect())
    }
    "latin1" | "binary" => Ok(b.iter().map(|x| *x as char).collect()),
    "utf16le" | "ucs2" => {
      if b.len() % 2 != 0 {
        return Err(Error::new(
          Status::GenericFailure,
          "invalid UTF-16LE bytes: odd length",
        ));
      }
      let units = b
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect::<Vec<_>>();
      String::from_utf16(&units)
        .map_err(|_| Error::new(Status::GenericFailure, "invalid UTF-16LE bytes"))
    }
    "hex" => Ok(encode_hex(b)),
    "base64" => Ok(encode_base64(b, false)),
    "base64url" => Ok(encode_base64(b, true)),
    other => Err(Error::new(
      Status::GenericFailure,
      format!("unsupported encoding: {other}"),
    )),
  }
}

fn decode_hex(s: &str) -> Result<Vec<u8>> {
  let s = s.trim();
  if s.len() % 2 != 0 {
    return Err(Error::new(Status::GenericFailure, "invalid hex string"));
  }
  let mut out = Vec::with_capacity(s.len() / 2);
  let bytes = s.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
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
    _ => Err(Error::new(Status::GenericFailure, "invalid hex digit")),
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
  let mut cleaned: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
  // Tolerate missing padding.
  while cleaned.len() % 4 != 0 {
    cleaned.push(b'=');
  }
  let mut out = Vec::new();
  let mut i = 0;
  while i + 4 <= cleaned.len() {
    let mut buf = [0u8; 4];
    for k in 0..4 {
      buf[k] = b64_val(cleaned[i + k], url)?;
    }
    let n = ((buf[0] << 18) | (buf[1] << 12) | (buf[2] << 6) | buf[3]) as u32;
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
    _ => Err(Error::new(Status::GenericFailure, "invalid base64 character")),
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

const SUPPORTED_ENCODINGS: &[&str] = &[
  "utf8",
  "utf-8",
  "ascii",
  "latin1",
  "binary",
  "utf16le",
  "ucs2",
  "hex",
  "base64",
  "base64url",
];

// ===========================================================================
// `Buffer` — node:buffer-compatible byte buffer.
// ===========================================================================

#[napi]
pub struct Buffer {
  data: Vec<u8>,
}

#[napi]
impl Buffer {
  /// Creates a zero-filled buffer of `length` bytes (like `Buffer.alloc`).
  #[napi(constructor)]
  pub fn new(length: u32) -> Self {
    Self {
      data: vec![0u8; length as usize],
    }
  }

  /// Allocates a zero-filled buffer of `length` bytes.
  #[napi]
  pub fn alloc(length: u32, fill: Option<u32>, encoding: Option<String>) -> Result<Buffer> {
    let mut b = Buffer::new(length);
    if let Some(v) = fill {
      b.data.iter_mut().for_each(|x| *x = v as u8);
    } else if let Some(enc) = encoding {
      // `fill` was provided as a string with an encoding; default to a space.
      let bytes = str_to_bytes(" ", &enc)?;
      if !bytes.is_empty() {
        for (i, v) in b.data.iter_mut().enumerate() {
          *v = bytes[i % bytes.len()];
        }
      }
    }
    Ok(b)
  }

  /// Allocates an *uninitialized* buffer of `length` bytes.
  ///
  /// For safety this still zero-fills (napi cannot hand back truly
  /// uninitialized memory across the FFI boundary), but it is documented as
  /// the `allocUnsafe` equivalent.
  #[napi]
  pub fn alloc_unsafe(length: u32) -> Buffer {
    Buffer::new(length)
  }

  /// Creates a buffer by copying an existing Node `Buffer`.
  #[napi(factory)]
  pub fn from_buffer(buffer: JsBuffer) -> Buffer {
    Buffer {
      data: buffer.as_ref().to_vec(),
    }
  }

  /// Creates a buffer from a string in the given encoding (default `utf8`).
  #[napi(factory)]
  pub fn from_string(string: String, encoding: Option<String>) -> Result<Buffer> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    Ok(Buffer {
      data: str_to_bytes(&string, &enc)?,
    })
  }

  /// Creates a buffer from an array of numbers, each truncated mod 256.
  #[napi(factory)]
  pub fn from_array(items: Vec<f64>) -> Buffer {
    Buffer {
      data: items
        .iter()
        .map(|v| ((*v as i32) & 0xff) as u8)
        .collect(),
    }
  }

  /// Concatenates a list of buffers into one. If `total_length` is omitted it
  /// is the sum of the input lengths.
  #[napi]
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

  /// Returns true if `value` is a Node `Buffer`.
  #[napi]
  pub fn is_buffer(value: Unknown) -> Result<bool> {
    value.is_buffer()
  }

  /// Returns true if `encoding` is a supported encoding name.
  #[napi]
  pub fn is_encoding(encoding: String) -> bool {
    SUPPORTED_ENCODINGS.contains(&encoding.to_ascii_lowercase().as_str())
  }

  /// The number of bytes in the buffer.
  #[napi(getter)]
  pub fn length(&self) -> u32 {
    self.data.len() as u32
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

  fn off(off: Option<u32>) -> usize {
    off.unwrap_or(0) as usize
  }

  // --- unsigned integer reads -------------------------------------------

  #[napi]
  pub fn read_u_int8(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 1)?;
    Ok(self.data[o] as u32)
  }

  #[napi]
  pub fn read_u_int8_le(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int8(offset)
  }

  #[napi]
  pub fn read_u_int8_be(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int8(offset)
  }

  #[napi]
  pub fn read_int8(&self, offset: Option<u32>) -> Result<i32> {
    let o = Self::off(offset);
    self.check(o, 1)?;
    Ok((self.data[o] as i8) as i32)
  }

  #[napi]
  pub fn read_u_int16_le(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    Ok(u16::from_le_bytes([self.data[o], self.data[o + 1]]) as u32)
  }

  #[napi]
  pub fn read_u_int16_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    Ok(u16::from_be_bytes([self.data[o], self.data[o + 1]]) as u32)
  }

  #[napi]
  pub fn read_int16_le(&self, offset: Option<u32>) -> Result<i32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    Ok(i16::from_le_bytes([self.data[o], self.data[o + 1]]) as i32)
  }

  #[napi]
  pub fn read_int16_be(&self, offset: Option<u32>) -> Result<i32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    Ok(i16::from_be_bytes([self.data[o], self.data[o + 1]]) as i32)
  }

  #[napi]
  pub fn read_u_int32_le(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(u32::from_le_bytes(b))
  }

  #[napi]
  pub fn read_u_int32_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(u32::from_be_bytes(b))
  }

  #[napi]
  pub fn read_int32_le(&self, offset: Option<u32>) -> Result<i32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(i32::from_le_bytes(b))
  }

  #[napi]
  pub fn read_int32_be(&self, offset: Option<u32>) -> Result<i32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(i32::from_be_bytes(b))
  }

  #[napi]
  pub fn read_float_le(&self, offset: Option<u32>) -> Result<f64> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(f32::from_le_bytes(b) as f64)
  }

  #[napi]
  pub fn read_float_be(&self, offset: Option<u32>) -> Result<f64> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(f32::from_be_bytes(b) as f64)
  }

  #[napi]
  pub fn read_double_le(&self, offset: Option<u32>) -> Result<f64> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(f64::from_le_bytes(b))
  }

  #[napi]
  pub fn read_double_be(&self, offset: Option<u32>) -> Result<f64> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(f64::from_be_bytes(b))
  }

  #[napi]
  pub fn read_big_u_int64_le(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(u64::from_le_bytes(b)))
  }

  #[napi]
  pub fn read_big_u_int64_be(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(u64::from_be_bytes(b)))
  }

  #[napi]
  pub fn read_big_int64_le(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(i64::from_le_bytes(b)))
  }

  #[napi]
  pub fn read_big_int64_be(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(i64::from_be_bytes(b)))
  }

  // --- unsigned/signed integer writes -----------------------------------

  #[napi]
  pub fn write_u_int8(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 1)?;
    self.data[o] = (value & 0xff) as u8;
    Ok((o + 1) as u32)
  }

  #[napi]
  pub fn write_int8(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 1)?;
    self.data[o] = (value & 0xff) as u8;
    Ok((o + 1) as u32)
  }

  #[napi]
  pub fn write_u_int16_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as u16).to_le_bytes());
    Ok((o + 2) as u32)
  }

  #[napi]
  pub fn write_u_int16_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as u16).to_be_bytes());
    Ok((o + 2) as u32)
  }

  #[napi]
  pub fn write_int16_le(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as i16).to_le_bytes());
    Ok((o + 2) as u32)
  }

  #[napi]
  pub fn write_int16_be(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as i16).to_be_bytes());
    Ok((o + 2) as u32)
  }

  #[napi]
  pub fn write_u_int32_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_le_bytes());
    Ok((o + 4) as u32)
  }

  #[napi]
  pub fn write_u_int32_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_be_bytes());
    Ok((o + 4) as u32)
  }

  #[napi]
  pub fn write_int32_le(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_le_bytes());
    Ok((o + 4) as u32)
  }

  #[napi]
  pub fn write_int32_be(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_be_bytes());
    Ok((o + 4) as u32)
  }

  #[napi]
  pub fn write_float_le(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&(value as f32).to_le_bytes());
    Ok((o + 4) as u32)
  }

  #[napi]
  pub fn write_float_be(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&(value as f32).to_be_bytes());
    Ok((o + 4) as u32)
  }

  #[napi]
  pub fn write_double_le(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    self.data[o..o + 8].copy_from_slice(&value.to_le_bytes());
    Ok((o + 8) as u32)
  }

  #[napi]
  pub fn write_double_be(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    self.data[o..o + 8].copy_from_slice(&value.to_be_bytes());
    Ok((o + 8) as u32)
  }

  #[napi]
  pub fn write_big_u_int64_le(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let (_, v, _) = value.get_u64();
    self.data[o..o + 8].copy_from_slice(&v.to_le_bytes());
    Ok((o + 8) as u32)
  }

  #[napi]
  pub fn write_big_u_int64_be(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let (_, v, _) = value.get_u64();
    self.data[o..o + 8].copy_from_slice(&v.to_be_bytes());
    Ok((o + 8) as u32)
  }

  #[napi]
  pub fn write_big_int64_le(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let (v, _) = value.get_i64();
    self.data[o..o + 8].copy_from_slice(&v.to_le_bytes());
    Ok((o + 8) as u32)
  }

  #[napi]
  pub fn write_big_int64_be(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let (v, _) = value.get_i64();
    self.data[o..o + 8].copy_from_slice(&v.to_be_bytes());
    Ok((o + 8) as u32)
  }

  // --- string read/write -------------------------------------------------

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
    let o = Self::off(offset);
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
  #[napi]
  pub fn to_string(
    &self,
    encoding: Option<String>,
    start: Option<u32>,
    end: Option<u32>,
  ) -> Result<String> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let s = Self::off(Some(start.unwrap_or(0)));
    let e = end.unwrap_or(self.data.len() as u32) as usize;
    if s > e || e > self.data.len() {
      return Err(Error::new(
        Status::GenericFailure,
        "invalid start/end range",
      ));
    }
    bytes_to_str(&self.data[s..e], &enc)
  }

  /// Returns a JSON-compatible representation `{ type: "Buffer", data: [...] }`.
  #[napi]
  pub fn to_json(&self) -> BufferJson {
    BufferJson {
      r#type: "Buffer".to_string(),
      data: self.data.iter().map(|b| *b as u32).collect(),
    }
  }

  // --- views / comparison ------------------------------------------------

  /// Returns a copy of the bytes in `start..end` as a Node `Buffer`.
  #[napi]
  pub fn slice(&self, start: Option<u32>, end: Option<u32>) -> Result<JsBuffer> {
    let (s, e) = self.range(start, end)?;
    Ok(JsBuffer::from(&self.data[s..e]))
  }

  /// Alias of `slice` (Node's `subarray` shares memory; here it copies, as
  /// napi cannot hand back a shared view).
  #[napi]
  pub fn subarray(&self, start: Option<u32>, end: Option<u32>) -> Result<JsBuffer> {
    self.slice(start, end)
  }

  /// Returns a copy of all written bytes as a Node `Buffer`.
  #[napi]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  /// Fills `value` (number, string, or buffer) over `start..end`. Mutates in
  /// place.
  #[napi]
  pub fn fill_number(&mut self, value: u32, start: Option<u32>, end: Option<u32>) -> Result<()> {
    let (s, e) = self.range(start, end)?;
    self.data[s..e].iter_mut().for_each(|x| *x = value as u8);
    Ok(())
  }

  #[napi]
  pub fn fill_string(
    &mut self,
    value: String,
    start: Option<u32>,
    end: Option<u32>,
    encoding: Option<String>,
  ) -> Result<()> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let bytes = str_to_bytes(&value, &enc)?;
    if bytes.is_empty() {
      return Ok(());
    }
    let (s, e) = self.range(start, end)?;
    for (i, slot) in self.data[s..e].iter_mut().enumerate() {
      *slot = bytes[i % bytes.len()];
    }
    Ok(())
  }

  #[napi]
  pub fn fill_buffer(&mut self, value: JsBuffer, start: Option<u32>, end: Option<u32>) -> Result<()> {
    let bytes = value.as_ref();
    if bytes.is_empty() {
      return Ok(());
    }
    let (s, e) = self.range(start, end)?;
    for (i, slot) in self.data[s..e].iter_mut().enumerate() {
      *slot = bytes[i % bytes.len()];
    }
    Ok(())
  }

  /// Returns true if the bytes equal `other`.
  #[napi]
  pub fn equals(&self, other: JsBuffer) -> bool {
    self.data == other.as_ref()
  }

  /// Lexicographic comparison with `other`: -1, 0, or 1.
  #[napi]
  pub fn compare(&self, other: JsBuffer) -> i32 {
    self.data.cmp(other.as_ref()) as i32
  }

  /// Returns the index of `value` (a byte, searched from `offset`), or -1.
  #[napi]
  pub fn index_of_number(&self, value: u32, offset: Option<u32>) -> i32 {
    let o = Self::off(offset);
    match self.data[o..].iter().position(|b| *b == (value & 0xff) as u8) {
      Some(p) => (o + p) as i32,
      None => -1,
    }
  }

  #[napi]
  pub fn index_of_string(&self, value: String, offset: Option<u32>, encoding: Option<String>) -> Result<i32> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let bytes = str_to_bytes(&value, &enc)?;
    let o = Self::off(offset);
    if bytes.is_empty() {
      return Ok(o as i32);
    }
    if let Some(p) = self.data[o..].windows(bytes.len()).position(|w| w == bytes.as_slice()) {
      return Ok((o + p) as i32);
    }
    Ok(-1)
  }

  #[napi]
  pub fn includes_number(&self, value: u32, offset: Option<u32>) -> bool {
    self.index_of_number(value, offset) >= 0
  }

  #[napi]
  pub fn includes_string(&self, value: String, offset: Option<u32>, encoding: Option<String>) -> Result<bool> {
    Ok(self.index_of_string(value, offset, encoding)? >= 0)
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

  // --- BufMut-style appends (grow the buffer) ----------------------------

  #[napi]
  pub fn put_slice(&mut self, src: JsBuffer) -> Result<()> {
    self.data.extend_from_slice(src.as_ref());
    Ok(())
  }

  #[napi]
  pub fn put_bytes(&mut self, val: u8, cnt: u32) {
    self.data.extend(std::iter::repeat(val).take(cnt as usize));
  }

  #[napi]
  pub fn put_u8(&mut self, n: u8) {
    self.data.push(n);
  }

  #[napi]
  pub fn put_i8(&mut self, n: i8) {
    self.data.push(n as u8);
  }

  #[napi]
  pub fn put_u16(&mut self, n: u16) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi]
  pub fn put_u16_le(&mut self, n: u16) {
    self.data.extend_from_slice(&n.to_le_bytes());
  }

  #[napi]
  pub fn put_u16_ne(&mut self, n: u16) {
    self.data.extend_from_slice(&n.to_ne_bytes());
  }

  #[napi]
  pub fn put_i16(&mut self, n: i16) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi]
  pub fn put_i16_le(&mut self, n: i16) {
    self.data.extend_from_slice(&n.to_le_bytes());
  }

  #[napi]
  pub fn put_i16_ne(&mut self, n: i16) {
    self.data.extend_from_slice(&n.to_ne_bytes());
  }

  #[napi]
  pub fn put_u32(&mut self, n: u32) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi]
  pub fn put_u32_le(&mut self, n: u32) {
    self.data.extend_from_slice(&n.to_le_bytes());
  }

  #[napi]
  pub fn put_u32_ne(&mut self, n: u32) {
    self.data.extend_from_slice(&n.to_ne_bytes());
  }

  #[napi]
  pub fn put_i32(&mut self, n: i32) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi]
  pub fn put_i32_le(&mut self, n: i32) {
    self.data.extend_from_slice(&n.to_le_bytes());
  }

  #[napi]
  pub fn put_i32_ne(&mut self, n: i32) {
    self.data.extend_from_slice(&n.to_ne_bytes());
  }

  #[napi]
  pub fn put_u64(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u64();
    self.data.extend_from_slice(&v.to_be_bytes());
  }

  #[napi]
  pub fn put_u64_le(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u64();
    self.data.extend_from_slice(&v.to_le_bytes());
  }

  #[napi]
  pub fn put_u64_ne(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u64();
    self.data.extend_from_slice(&v.to_ne_bytes());
  }

  #[napi]
  pub fn put_i64(&mut self, n: BigInt) {
    let (v, _) = n.get_i64();
    self.data.extend_from_slice(&v.to_be_bytes());
  }

  #[napi]
  pub fn put_i64_le(&mut self, n: BigInt) {
    let (v, _) = n.get_i64();
    self.data.extend_from_slice(&v.to_le_bytes());
  }

  #[napi]
  pub fn put_i64_ne(&mut self, n: BigInt) {
    let (v, _) = n.get_i64();
    self.data.extend_from_slice(&v.to_ne_bytes());
  }

  #[napi]
  pub fn put_u128(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u128();
    self.data.extend_from_slice(&v.to_be_bytes());
  }

  #[napi]
  pub fn put_u128_le(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u128();
    self.data.extend_from_slice(&v.to_le_bytes());
  }

  #[napi]
  pub fn put_u128_ne(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u128();
    self.data.extend_from_slice(&v.to_ne_bytes());
  }

  #[napi]
  pub fn put_i128(&mut self, n: BigInt) {
    let (v, _) = n.get_i128();
    self.data.extend_from_slice(&v.to_be_bytes());
  }

  #[napi]
  pub fn put_i128_le(&mut self, n: BigInt) {
    let (v, _) = n.get_i128();
    self.data.extend_from_slice(&v.to_le_bytes());
  }

  #[napi]
  pub fn put_i128_ne(&mut self, n: BigInt) {
    let (v, _) = n.get_i128();
    self.data.extend_from_slice(&v.to_ne_bytes());
  }

  #[napi]
  pub fn put_uint(&mut self, n: BigInt, nbytes: u32) -> Result<()> {
    self.put_uint_like(n, nbytes, false)
  }

  #[napi]
  pub fn put_uint_le(&mut self, n: BigInt, nbytes: u32) -> Result<()> {
    self.put_uint_like(n, nbytes, true)
  }

  #[napi]
  pub fn put_f32(&mut self, n: f64) {
    self.data.extend_from_slice(&(n as f32).to_be_bytes());
  }

  #[napi]
  pub fn put_f32_le(&mut self, n: f64) {
    self.data.extend_from_slice(&(n as f32).to_le_bytes());
  }

  #[napi]
  pub fn put_f64(&mut self, n: f64) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi]
  pub fn put_f64_le(&mut self, n: f64) {
    self.data.extend_from_slice(&n.to_le_bytes());
  }

  /// Ensures capacity for at least `additional` more bytes.
  #[napi]
  pub fn reserve(&mut self, additional: u32) {
    self.data.reserve(additional as usize);
  }
}

impl Buffer {
  fn range(&self, start: Option<u32>, end: Option<u32>) -> Result<(usize, usize)> {
    let s = Self::off(start).min(self.data.len());
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

  fn put_uint_like(&mut self, n: BigInt, nbytes: u32, le: bool) -> Result<()> {
    if (nbytes as usize) > 8 {
      return Err(Error::new(
        Status::GenericFailure,
        format!("nbytes must be <= 8, got {nbytes}"),
      ));
    }
    let (_, v, _) = n.get_u64();
    let bytes = if le {
      v.to_le_bytes()[..nbytes as usize].to_vec()
    } else {
      v.to_be_bytes()[8 - nbytes as usize..].to_vec()
    };
    self.data.extend_from_slice(&bytes);
    Ok(())
  }
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
#[napi]
pub fn is_ascii(input: JsBuffer) -> bool {
  input.as_ref().iter().all(|b| *b < 128)
}

/// Returns true if `input` is valid UTF-8.
#[napi]
pub fn is_utf8(input: JsBuffer) -> bool {
  std::str::from_utf8(input.as_ref()).is_ok()
}

/// Re-encodes `source` from `from_encoding` to `to_encoding`.
#[napi]
pub fn transcode(source: JsBuffer, from_encoding: String, to_encoding: String) -> Result<JsBuffer> {
  let intermediate = bytes_to_str(source.as_ref(), &from_encoding)?;
  let out = str_to_bytes(&intermediate, &to_encoding)?;
  Ok(JsBuffer::from(out.as_slice()))
}

/// Returns the `node:buffer` constants.
#[napi]
pub fn constants() -> BufferConstants {
  BufferConstants {
    max_length: 0x7fff_ffff,
    max_string_length: 0x1ffff_fe8,
    inspect_max_bytes: 50,
    k_max_length: 0x7fff_ffff,
    k_string_max_length: 0x1ffff_fe8,
  }
}

#[napi(object)]
pub struct BufferConstants {
  pub max_length: u32,
  pub max_string_length: u32,
  pub inspect_max_bytes: u32,
  pub k_max_length: u32,
  pub k_string_max_length: u32,
}
