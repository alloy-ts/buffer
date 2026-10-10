//! NAPI bindings for a `node:buffer`-compatible `Buffer` class.

use napi::bindgen_prelude::BigInt;
use napi::bindgen_prelude::Buffer as JsBuffer;
use napi::bindgen_prelude::Either;
use napi::bindgen_prelude::Either3;
use napi::bindgen_prelude::Unknown;
use napi::{Error, JsValue, Result, Status};
use napi_derive::napi;

fn buffer_ref<'a>(input: &'a Either<JsBuffer, &'a Buffer>) -> &'a [u8] {
  match input {
    Either::A(b) => b.as_ref(),
    Either::B(b) => b.data.as_slice(),
  }
}


// ===========================================================================
// Encoding helpers (utf8 / ascii / latin1 / utf16le / hex / base64).
// ===========================================================================

fn str_to_bytes(s: &str, enc: &str) -> Result<Vec<u8>> {
  match enc.to_ascii_lowercase().as_str() {
    "utf8" | "utf-8" => Ok(s.as_bytes().to_vec()),
    "ascii" | "latin1" | "binary" => Ok(s.chars().map(|c| (c as u32 & 0xff) as u8).collect()),
    "utf16le" | "utf-16le" | "ucs2" | "ucs-2" => {
      let mut out = Vec::with_capacity(s.len() * 2);
      for unit in s.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
      }
      Ok(out)
    }
    "hex" => Ok(decode_hex(s)),
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
    "utf8" | "utf-8" => Ok(String::from_utf8_lossy(b).into_owned()),
    "ascii" => Ok(b.iter().map(|x| (*x & 0x7f) as char).collect()),
    "latin1" | "binary" => Ok(b.iter().map(|x| *x as char).collect()),
    "utf16le" | "utf-16le" | "ucs2" | "ucs-2" => {
      let units: Vec<u16> = b
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
      Ok(String::from_utf16_lossy(&units))
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

fn decode_hex(s: &str) -> Vec<u8> {
  let bytes = s.as_bytes();
  let mut out = Vec::with_capacity(bytes.len() / 2);
  let mut i = 0;
  while i + 1 < bytes.len() {
    let hi = match hex_val(bytes[i]) {
      Some(v) => v,
      None => break,
    };
    let lo = match hex_val(bytes[i + 1]) {
      Some(v) => v,
      None => break,
    };
    out.push((hi << 4) | lo);
    i += 2;
  }
  out
}

fn hex_val(b: u8) -> Option<u8> {
  match b {
    b'0'..=b'9' => Some(b - b'0'),
    b'a'..=b'f' => Some(b - b'a' + 10),
    b'A'..=b'F' => Some(b - b'A' + 10),
    _ => None,
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
    let n = (((buf[0] as u32) << 18)
      | ((buf[1] as u32) << 12)
      | ((buf[2] as u32) << 6)
      | (buf[3] as u32)) as u32;
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
  "utf-16le",
  "ucs2",
  "ucs-2",
  "hex",
  "base64",
  "base64url",
];

// ===========================================================================
// `Buffer` — node:buffer-compatible byte buffer.
// ===========================================================================

#[napi]
pub struct Buffer {
  pub data: Vec<u8>,
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
  pub fn alloc(
    length: u32,
    #[napi(ts_arg_type = "number | string")] fill: Option<Either<u32, String>>,
    encoding: Option<String>,
  ) -> Result<Buffer> {
    let mut b = Buffer::new(length);
    if let Some(fill_val) = fill {
      match fill_val {
        Either::A(num) => {
          b.data.iter_mut().for_each(|x| *x = (num & 0xff) as u8);
        }
        Either::B(s) => {
          let enc = encoding.unwrap_or_else(|| "utf8".to_string());
          let bytes = str_to_bytes(&s, &enc)?;
          if !bytes.is_empty() {
            for (i, v) in b.data.iter_mut().enumerate() {
              *v = bytes[i % bytes.len()];
            }
          }
        }
      }
    }
    Ok(b)
  }

  /// Allocates an uninitialized buffer of `length` bytes.
  #[napi(js_name = "allocUnsafe")]
  pub fn alloc_unsafe(length: u32) -> Buffer {
    Buffer::new(length)
  }

  /// Allocates an uninitialized buffer of `length` bytes.
  #[napi(js_name = "allocUnsafeSlow")]
  pub fn alloc_unsafe_slow(length: u32) -> Buffer {
    Buffer::new(length)
  }

  /// Returns the byte length of input when encoded.
  #[napi(js_name = "byteLength")]
  pub fn byte_length(
    #[napi(ts_arg_type = "string | Buffer | Uint8Array")] input: Either3<
      String,
      JsBuffer,
      &Buffer,
    >,
    encoding: Option<String>,
  ) -> Result<u32> {
    match input {
      Either3::A(s) => {
        let enc = encoding.unwrap_or_else(|| "utf8".to_string());
        Ok(str_to_bytes(&s, &enc)?.len() as u32)
      }
      Either3::B(buf) => Ok(buf.as_ref().len() as u32),
      Either3::C(buf) => Ok(buf.data.len() as u32),
    }
  }

  /// Returns string length in UTF-16 code units when decoded.
  #[napi(js_name = "stringLength")]
  pub fn string_length(
    #[napi(ts_arg_type = "Buffer | Uint8Array")] input: Either<JsBuffer, &Buffer>,
    encoding: Option<String>,
  ) -> Result<u32> {
    let bytes = buffer_ref(&input);
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let s = bytes_to_str(bytes, &enc)?;
    Ok(s.chars().map(|c| c.len_utf16()).sum::<usize>() as u32)
  }

  /// Compares two buffers lexicographically.
  #[napi]
  pub fn compare_buffers(
    #[napi(ts_arg_type = "Buffer | Uint8Array")] buf1: Either<JsBuffer, &Buffer>,
    #[napi(ts_arg_type = "Buffer | Uint8Array")] buf2: Either<JsBuffer, &Buffer>,
  ) -> i32 {
    buffer_ref(&buf1).cmp(buffer_ref(&buf2)) as i32
  }

  /// Copies bytes from view into a new Buffer.
  #[napi(js_name = "copyBytesFrom")]
  pub fn copy_bytes_from(
    view: JsBuffer,
    offset: Option<u32>,
    length: Option<u32>,
  ) -> Result<Buffer> {
    let bytes = view.as_ref();
    let off = offset.unwrap_or(0) as usize;
    if off > bytes.len() {
      return Err(Error::new(Status::GenericFailure, "offset out of range"));
    }
    let len = length.unwrap_or((bytes.len() - off) as u32) as usize;
    let end = (off + len).min(bytes.len());
    Ok(Buffer {
      data: bytes[off..end].to_vec(),
    })
  }

  /// Creates a buffer from an ArrayBuffer view.
  #[napi(factory)]
  pub fn from_array_buffer(
    array_buffer: JsBuffer,
    byte_offset: Option<u32>,
    length: Option<u32>,
  ) -> Result<Buffer> {
    Self::copy_bytes_from(array_buffer, byte_offset, length)
  }

  #[napi(getter, js_name = "poolSize")]
  pub fn pool_size() -> u32 {
    65536
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

  #[napi(js_name = "readUint8")]
  pub fn read_uint8(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int8(offset)
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

  #[napi(js_name = "readUint16LE")]
  pub fn read_uint16_le(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_le(offset)
  }

  #[napi]
  pub fn read_u_int16_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    Ok(u16::from_be_bytes([self.data[o], self.data[o + 1]]) as u32)
  }

  #[napi(js_name = "readUint16BE")]
  pub fn read_uint16_be(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_be(offset)
  }

  #[napi(js_name = "readInt16LE")]
  pub fn read_int16_le(&self, offset: Option<u32>) -> Result<i32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    Ok(i16::from_le_bytes([self.data[o], self.data[o + 1]]) as i32)
  }

  #[napi(js_name = "readInt16BE")]
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

  #[napi(js_name = "readUint32LE")]
  pub fn read_uint32_le(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_le(offset)
  }

  #[napi]
  pub fn read_u_int32_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(u32::from_be_bytes(b))
  }

  #[napi(js_name = "readUint32BE")]
  pub fn read_uint32_be(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_be(offset)
  }

  #[napi(js_name = "readInt32LE")]
  pub fn read_int32_le(&self, offset: Option<u32>) -> Result<i32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(i32::from_le_bytes(b))
  }

  #[napi(js_name = "readInt32BE")]
  pub fn read_int32_be(&self, offset: Option<u32>) -> Result<i32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(i32::from_be_bytes(b))
  }

  #[napi(js_name = "readFloatLE")]
  pub fn read_float_le(&self, offset: Option<u32>) -> Result<f64> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(f32::from_le_bytes(b) as f64)
  }

  #[napi(js_name = "readFloatBE")]
  pub fn read_float_be(&self, offset: Option<u32>) -> Result<f64> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(f32::from_be_bytes(b) as f64)
  }

  #[napi(js_name = "readDoubleLE")]
  pub fn read_double_le(&self, offset: Option<u32>) -> Result<f64> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(f64::from_le_bytes(b))
  }

  #[napi(js_name = "readDoubleBE")]
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

  #[napi(js_name = "readBigUint64LE")]
  pub fn read_big_uint64_le(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_u_int64_le(offset)
  }

  #[napi]
  pub fn read_big_u_int64_be(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(u64::from_be_bytes(b)))
  }

  #[napi(js_name = "readBigUint64BE")]
  pub fn read_big_uint64_be(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_u_int64_be(offset)
  }

  #[napi]
  pub fn read_u_int_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let bl = byte_length as usize;
    if bl == 0 || bl > 6 {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, bl)?;
    let mut val: u64 = 0;
    for i in 0..bl {
      val = (val << 8) | (self.data[o + i] as u64);
    }
    Ok(val as f64)
  }

  #[napi(js_name = "readUintBE")]
  pub fn read_uint_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_u_int_be(offset, byte_length)
  }

  #[napi]
  pub fn read_u_int_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let bl = byte_length as usize;
    if bl == 0 || bl > 6 {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, bl)?;
    let mut val: u64 = 0;
    for i in (0..bl).rev() {
      val = (val << 8) | (self.data[o + i] as u64);
    }
    Ok(val as f64)
  }

  #[napi(js_name = "readUintLE")]
  pub fn read_uint_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_u_int_le(offset, byte_length)
  }

  #[napi]
  pub fn read_int_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let bl = byte_length as usize;
    if bl == 0 || bl > 6 {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, bl)?;
    let mut val: u64 = 0;
    for i in 0..bl {
      val = (val << 8) | (self.data[o + i] as u64);
    }
    if (self.data[o] & 0x80) != 0 {
      val |= !0u64 << (bl * 8);
    }
    Ok((val as i64) as f64)
  }

  #[napi]
  pub fn read_int_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let bl = byte_length as usize;
    if bl == 0 || bl > 6 {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, bl)?;
    let mut val: u64 = 0;
    for i in (0..bl).rev() {
      val = (val << 8) | (self.data[o + i] as u64);
    }
    if (self.data[o + bl - 1] & 0x80) != 0 {
      val |= !0u64 << (bl * 8);
    }
    Ok((val as i64) as f64)
  }

  #[napi(js_name = "readBigInt64LE")]
  pub fn read_big_int64_le(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(i64::from_le_bytes(b)))
  }

  #[napi(js_name = "readBigInt64BE")]
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

  #[napi(js_name = "writeUint8")]
  pub fn write_uint8(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int8(value, offset)
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

  #[napi(js_name = "writeUint16LE")]
  pub fn write_uint16_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_le(value, offset)
  }

  #[napi]
  pub fn write_u_int16_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as u16).to_be_bytes());
    Ok((o + 2) as u32)
  }

  #[napi(js_name = "writeUint16BE")]
  pub fn write_uint16_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_be(value, offset)
  }

  #[napi(js_name = "writeInt16LE")]
  pub fn write_int16_le(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as i16).to_le_bytes());
    Ok((o + 2) as u32)
  }

  #[napi(js_name = "writeInt16BE")]
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

  #[napi(js_name = "writeUint32LE")]
  pub fn write_uint32_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_le(value, offset)
  }

  #[napi]
  pub fn write_u_int32_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_be_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeUint32BE")]
  pub fn write_uint32_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_be(value, offset)
  }

  #[napi(js_name = "writeInt32LE")]
  pub fn write_int32_le(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_le_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeInt32BE")]
  pub fn write_int32_be(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_be_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeFloatLE")]
  pub fn write_float_le(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&(value as f32).to_le_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeFloatBE")]
  pub fn write_float_be(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&(value as f32).to_be_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeDoubleLE")]
  pub fn write_double_le(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    self.data[o..o + 8].copy_from_slice(&value.to_le_bytes());
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeDoubleBE")]
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

  #[napi(js_name = "writeBigUint64LE")]
  pub fn write_big_uint64_le(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_le(value, offset)
  }

  #[napi]
  pub fn write_big_u_int64_be(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let (_, v, _) = value.get_u64();
    self.data[o..o + 8].copy_from_slice(&v.to_be_bytes());
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeBigUint64BE")]
  pub fn write_big_uint64_be(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_be(value, offset)
  }

  #[napi]
  pub fn write_u_int_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    let o = offset as usize;
    let bl = byte_length as usize;
    if bl == 0 || bl > 6 {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, bl)?;
    let v = value as u64;
    for i in (0..bl).rev() {
      self.data[o + i] = ((v >> ((bl - 1 - i) * 8)) & 0xff) as u8;
    }
    Ok((o + bl) as u32)
  }

  #[napi(js_name = "writeUintBE")]
  pub fn write_uint_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_be(value, offset, byte_length)
  }

  #[napi]
  pub fn write_u_int_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    let o = offset as usize;
    let bl = byte_length as usize;
    if bl == 0 || bl > 6 {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, bl)?;
    let v = value as u64;
    for i in 0..bl {
      self.data[o + i] = ((v >> (i * 8)) & 0xff) as u8;
    }
    Ok((o + bl) as u32)
  }

  #[napi(js_name = "writeUintLE")]
  pub fn write_uint_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_le(value, offset, byte_length)
  }

  #[napi]
  pub fn write_int_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_be(value, offset, byte_length)
  }

  #[napi]
  pub fn write_int_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_le(value, offset, byte_length)
  }

  #[napi(js_name = "writeBigInt64LE")]
  pub fn write_big_int64_le(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let (v, _) = value.get_i64();
    self.data[o..o + 8].copy_from_slice(&v.to_le_bytes());
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeBigInt64BE")]
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

  /// Alias of `slice`.
  #[napi]
  pub fn subarray(&self, start: Option<u32>, end: Option<u32>) -> Result<JsBuffer> {
    self.slice(start, end)
  }

  /// Returns a copy of all written bytes as a Node `Buffer`.
  #[napi]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  /// Fills `value` over `start..end`.
  #[napi(js_name = "fillNumber")]
  pub fn fill_number(&mut self, value: u32, start: Option<u32>, end: Option<u32>) -> Result<()> {
    let (s, e) = self.range(start, end)?;
    self.data[s..e].iter_mut().for_each(|x| *x = value as u8);
    Ok(())
  }

  #[napi(js_name = "fillString")]
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

  #[napi(js_name = "fillBuffer")]
  pub fn fill_buffer(
    &mut self,
    #[napi(ts_arg_type = "Buffer | Uint8Array")] value: Either<JsBuffer, &Buffer>,
    start: Option<u32>,
    end: Option<u32>,
  ) -> Result<()> {
    let bytes = buffer_ref(&value);
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
  pub fn equals(
    &self,
    #[napi(ts_arg_type = "Buffer | Uint8Array")] other: Either<JsBuffer, &Buffer>,
  ) -> bool {
    self.data.as_slice() == buffer_ref(&other)
  }

  /// Lexicographic comparison with `other`: -1, 0, or 1.
  #[napi]
  pub fn compare(
    &self,
    #[napi(ts_arg_type = "Buffer | Uint8Array")] other: Either<JsBuffer, &Buffer>,
  ) -> i32 {
    self.data.as_slice().cmp(buffer_ref(&other)) as i32
  }

  /// Returns the index of `value` (a byte, searched from `offset`), or -1.
  #[napi(js_name = "indexOfNumber")]
  pub fn index_of_number(&self, value: u32, offset: Option<u32>) -> i32 {
    let o = Self::off(offset);
    match self.data[o..].iter().position(|b| *b == (value & 0xff) as u8) {
      Some(p) => (o + p) as i32,
      None => -1,
    }
  }

  #[napi(js_name = "indexOfString")]
  pub fn index_of_string(
    &self,
    value: String,
    offset: Option<u32>,
    encoding: Option<String>,
  ) -> Result<i32> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let bytes = str_to_bytes(&value, &enc)?;
    let o = Self::off(offset);
    if bytes.is_empty() {
      return Ok(o as i32);
    }
    if o < self.data.len() {
      if let Some(p) = self.data[o..].windows(bytes.len()).position(|w| w == bytes.as_slice()) {
        return Ok((o + p) as i32);
      }
    }
    Ok(-1)
  }

  #[napi(js_name = "lastIndexOfNumber")]
  pub fn last_index_of_number(&self, value: u32, offset: Option<u32>) -> i32 {
    let target = (value & 0xff) as u8;
    if self.data.is_empty() {
      return -1;
    }
    let start = offset.map(|o| (o as usize).min(self.data.len() - 1)).unwrap_or(self.data.len() - 1);
    for i in (0..=start).rev() {
      if self.data[i] == target {
        return i as i32;
      }
    }
    -1
  }

  #[napi(js_name = "lastIndexOfString")]
  pub fn last_index_of_string(
    &self,
    value: String,
    offset: Option<u32>,
    encoding: Option<String>,
  ) -> Result<i32> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let bytes = str_to_bytes(&value, &enc)?;
    if bytes.is_empty() {
      return Ok(offset.unwrap_or(self.data.len() as u32) as i32);
    }
    if bytes.len() > self.data.len() {
      return Ok(-1);
    }
    let max_start = offset.map(|o| (o as usize).min(self.data.len() - bytes.len())).unwrap_or(self.data.len() - bytes.len());
    for i in (0..=max_start).rev() {
      if self.data[i..i + bytes.len()] == bytes[..] {
        return Ok(i as i32);
      }
    }
    Ok(-1)
  }

  /// Copies data from a region of this buffer to a target buffer.
  #[napi]
  pub fn copy(
    &self,
    #[napi(ts_arg_type = "Buffer | Uint8Array")] mut target: Either<JsBuffer, &mut Buffer>,
    target_start: Option<u32>,
    source_start: Option<u32>,
    source_end: Option<u32>,
  ) -> Result<u32> {
    let ts = target_start.unwrap_or(0) as usize;
    let ss = source_start.unwrap_or(0) as usize;
    let se = source_end.unwrap_or(self.data.len() as u32) as usize;
    let ss = ss.min(self.data.len());
    let se = se.min(self.data.len());
    if ss >= se {
      return Ok(0);
    }
    let len = se - ss;
    let target_bytes: &mut [u8] = match &mut target {
      Either::A(b) => b.as_mut(),
      Either::B(b) => b.data.as_mut_slice(),
    };
    if ts >= target_bytes.len() {
      return Ok(0);
    }
    let to_copy = len.min(target_bytes.len() - ts);
    target_bytes[ts..ts + to_copy].copy_from_slice(&self.data[ss..ss + to_copy]);
    Ok(to_copy as u32)
  }

  #[napi(js_name = "includesNumber")]
  pub fn includes_number(&self, value: u32, offset: Option<u32>) -> bool {
    self.index_of_number(value, offset) >= 0
  }

  #[napi(js_name = "includesString")]
  pub fn includes_string(
    &self,
    value: String,
    offset: Option<u32>,
    encoding: Option<String>,
  ) -> Result<bool> {
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

  #[napi(js_name = "putSlice")]
  pub fn put_slice(
    &mut self,
    #[napi(ts_arg_type = "Buffer | Uint8Array")] src: Either<JsBuffer, &Buffer>,
  ) -> Result<()> {
    self.data.extend_from_slice(buffer_ref(&src));
    Ok(())
  }

  #[napi(js_name = "putBytes")]
  pub fn put_bytes(&mut self, val: u8, cnt: u32) {
    self.data.extend(std::iter::repeat(val).take(cnt as usize));
  }

  #[napi(js_name = "putU8")]
  pub fn put_u8(&mut self, n: u8) {
    self.data.push(n);
  }

  #[napi(js_name = "putI8")]
  pub fn put_i8(&mut self, n: i8) {
    self.data.push(n as u8);
  }

  #[napi(js_name = "putU16")]
  pub fn put_u16(&mut self, n: u16) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi(js_name = "putU16Le")]
  pub fn put_u16_le(&mut self, n: u16) {
    self.data.extend_from_slice(&n.to_le_bytes());
  }

  #[napi(js_name = "putU16Ne")]
  pub fn put_u16_ne(&mut self, n: u16) {
    self.data.extend_from_slice(&n.to_ne_bytes());
  }

  #[napi(js_name = "putI16")]
  pub fn put_i16(&mut self, n: i16) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi(js_name = "putI16Le")]
  pub fn put_i16_le(&mut self, n: i16) {
    self.data.extend_from_slice(&n.to_le_bytes());
  }

  #[napi(js_name = "putI16Ne")]
  pub fn put_i16_ne(&mut self, n: i16) {
    self.data.extend_from_slice(&n.to_ne_bytes());
  }

  #[napi(js_name = "putU32")]
  pub fn put_u32(&mut self, n: u32) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi(js_name = "putU32Le")]
  pub fn put_u32_le(&mut self, n: u32) {
    self.data.extend_from_slice(&n.to_le_bytes());
  }

  #[napi(js_name = "putU32Ne")]
  pub fn put_u32_ne(&mut self, n: u32) {
    self.data.extend_from_slice(&n.to_ne_bytes());
  }

  #[napi(js_name = "putI32")]
  pub fn put_i32(&mut self, n: i32) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi(js_name = "putI32Le")]
  pub fn put_i32_le(&mut self, n: i32) {
    self.data.extend_from_slice(&n.to_le_bytes());
  }

  #[napi(js_name = "putI32Ne")]
  pub fn put_i32_ne(&mut self, n: i32) {
    self.data.extend_from_slice(&n.to_ne_bytes());
  }

  #[napi(js_name = "putU64")]
  pub fn put_u64(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u64();
    self.data.extend_from_slice(&v.to_be_bytes());
  }

  #[napi(js_name = "putU64Le")]
  pub fn put_u64_le(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u64();
    self.data.extend_from_slice(&v.to_le_bytes());
  }

  #[napi(js_name = "putU64Ne")]
  pub fn put_u64_ne(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u64();
    self.data.extend_from_slice(&v.to_ne_bytes());
  }

  #[napi(js_name = "putI64")]
  pub fn put_i64(&mut self, n: BigInt) {
    let (v, _) = n.get_i64();
    self.data.extend_from_slice(&v.to_be_bytes());
  }

  #[napi(js_name = "putI64Le")]
  pub fn put_i64_le(&mut self, n: BigInt) {
    let (v, _) = n.get_i64();
    self.data.extend_from_slice(&v.to_le_bytes());
  }

  #[napi(js_name = "putI64Ne")]
  pub fn put_i64_ne(&mut self, n: BigInt) {
    let (v, _) = n.get_i64();
    self.data.extend_from_slice(&v.to_ne_bytes());
  }

  #[napi(js_name = "putU128")]
  pub fn put_u128(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u128();
    self.data.extend_from_slice(&v.to_be_bytes());
  }

  #[napi(js_name = "putU128Le")]
  pub fn put_u128_le(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u128();
    self.data.extend_from_slice(&v.to_le_bytes());
  }

  #[napi(js_name = "putU128Ne")]
  pub fn put_u128_ne(&mut self, n: BigInt) {
    let (_, v, _) = n.get_u128();
    self.data.extend_from_slice(&v.to_ne_bytes());
  }

  #[napi(js_name = "putI128")]
  pub fn put_i128(&mut self, n: BigInt) {
    let (v, _) = n.get_i128();
    self.data.extend_from_slice(&v.to_be_bytes());
  }

  #[napi(js_name = "putI128Le")]
  pub fn put_i128_le(&mut self, n: BigInt) {
    let (v, _) = n.get_i128();
    self.data.extend_from_slice(&v.to_le_bytes());
  }

  #[napi(js_name = "putI128Ne")]
  pub fn put_i128_ne(&mut self, n: BigInt) {
    let (v, _) = n.get_i128();
    self.data.extend_from_slice(&v.to_ne_bytes());
  }

  #[napi(js_name = "putUint")]
  pub fn put_uint(&mut self, n: BigInt, nbytes: u32) -> Result<()> {
    self.put_uint_like(n, nbytes, false)
  }

  #[napi(js_name = "putUintLe")]
  pub fn put_uint_le(&mut self, n: BigInt, nbytes: u32) -> Result<()> {
    self.put_uint_like(n, nbytes, true)
  }

  #[napi(js_name = "putF32")]
  pub fn put_f32(&mut self, n: f64) {
    self.data.extend_from_slice(&(n as f32).to_be_bytes());
  }

  #[napi(js_name = "putF32Le")]
  pub fn put_f32_le(&mut self, n: f64) {
    self.data.extend_from_slice(&(n as f32).to_le_bytes());
  }

  #[napi(js_name = "putF64")]
  pub fn put_f64(&mut self, n: f64) {
    self.data.extend_from_slice(&n.to_be_bytes());
  }

  #[napi(js_name = "putF64Le")]
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
// `Blob` and `File` classes.
// ===========================================================================

#[napi(object)]
pub struct BlobOptions {
  pub r#type: Option<String>,
}

#[napi]
pub struct Blob {
  data: Vec<u8>,
  mime_type: String,
}

#[napi]
impl Blob {
  #[napi(constructor)]
  pub fn new(
    #[napi(ts_arg_type = "Array<string | Uint8Array>")] sources: Option<
      Vec<Either<String, JsBuffer>>,
    >,
    options: Option<BlobOptions>,
  ) -> Result<Self> {
    let mut data = Vec::new();
    if let Some(srcs) = sources {
      for src in srcs {
        match src {
          Either::A(s) => data.extend_from_slice(s.as_bytes()),
          Either::B(buf) => data.extend_from_slice(buf.as_ref()),
        }
      }
    }
    let mime_type = options
      .and_then(|o| o.r#type)
      .unwrap_or_else(String::new);
    Ok(Blob { data, mime_type })
  }

  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.data.len() as u32
  }

  #[napi(getter, js_name = "type")]
  pub fn get_type(&self) -> String {
    self.mime_type.clone()
  }

  #[napi(js_name = "arrayBuffer")]
  pub fn array_buffer(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi]
  pub fn slice(
    &self,
    start: Option<i32>,
    end: Option<i32>,
    content_type: Option<String>,
  ) -> Result<Blob> {
    let len = self.data.len() as i32;
    let s = match start {
      Some(v) if v < 0 => (len + v).max(0) as usize,
      Some(v) => (v as usize).min(self.data.len()),
      None => 0,
    };
    let e = match end {
      Some(v) if v < 0 => (len + v).max(0) as usize,
      Some(v) => (v as usize).min(self.data.len()),
      None => self.data.len(),
    };
    let sliced_data = if s < e {
      self.data[s..e].to_vec()
    } else {
      Vec::new()
    };
    let mime_type = content_type.unwrap_or_else(|| self.mime_type.clone());
    Ok(Blob {
      data: sliced_data,
      mime_type,
    })
  }

  #[napi]
  pub fn text(&self) -> String {
    String::from_utf8_lossy(&self.data).into_owned()
  }
}

#[napi(object)]
pub struct FileOptions {
  pub r#type: Option<String>,
  pub last_modified: Option<f64>,
}

#[napi]
pub struct File {
  blob: Blob,
  name: String,
  last_modified: f64,
}

#[napi]
impl File {
  #[napi(constructor)]
  pub fn new(
    #[napi(ts_arg_type = "Array<string | Uint8Array>")] sources: Option<
      Vec<Either<String, JsBuffer>>,
    >,
    file_name: String,
    options: Option<FileOptions>,
  ) -> Result<Self> {
    let blob = Blob::new(
      sources,
      options.as_ref().map(|o| BlobOptions {
        r#type: o.r#type.clone(),
      }),
    )?;
    let last_modified = options
      .and_then(|o| o.last_modified)
      .unwrap_or(0.0);
    Ok(File {
      blob,
      name: file_name,
      last_modified,
    })
  }

  #[napi(getter)]
  pub fn name(&self) -> String {
    self.name.clone()
  }

  #[napi(getter, js_name = "lastModified")]
  pub fn last_modified(&self) -> f64 {
    self.last_modified
  }

  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.blob.size()
  }

  #[napi(getter, js_name = "type")]
  pub fn get_type(&self) -> String {
    self.blob.get_type()
  }

  #[napi(js_name = "arrayBuffer")]
  pub fn array_buffer(&self) -> JsBuffer {
    self.blob.array_buffer()
  }

  #[napi]
  pub fn bytes(&self) -> JsBuffer {
    self.blob.bytes()
  }

  #[napi]
  pub fn slice(
    &self,
    start: Option<i32>,
    end: Option<i32>,
    content_type: Option<String>,
  ) -> Result<Blob> {
    self.blob.slice(start, end, content_type)
  }

  #[napi]
  pub fn text(&self) -> String {
    self.blob.text()
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
pub fn is_ascii(
  #[napi(ts_arg_type = "Buffer | Uint8Array")] input: Either<JsBuffer, &Buffer>,
) -> bool {
  buffer_ref(&input).iter().all(|b| *b < 128)
}

/// Returns true if every UTF-16 code unit in `input` is <= U+00FF.
#[napi(js_name = "isLatin1")]
pub fn is_latin1(input: String) -> bool {
  input.chars().all(|c| (c as u32) <= 0xff)
}

/// Returns true if `input` is valid UTF-8.
#[napi(js_name = "isUtf8")]
pub fn is_utf8(
  #[napi(ts_arg_type = "Buffer | Uint8Array")] input: Either<JsBuffer, &Buffer>,
) -> bool {
  std::str::from_utf8(buffer_ref(&input)).is_ok()
}

/// Resolves a blob URL string to a Blob object.
#[napi(js_name = "resolveObjectURL")]
pub fn resolve_object_url(_id: String) -> Option<Blob> {
  None
}

/// Re-encodes `source` from `from_encoding` to `to_encoding`.
#[napi]
pub fn transcode(
  #[napi(ts_arg_type = "Buffer | Uint8Array")] source: Either<JsBuffer, &Buffer>,
  from_encoding: String,
  to_encoding: String,
) -> Result<JsBuffer> {
  let intermediate = bytes_to_str(buffer_ref(&source), &from_encoding)?;
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
