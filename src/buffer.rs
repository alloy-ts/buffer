//! NAPI bindings for a `node:buffer`-compatible `Buffer` class.

use napi::bindgen_prelude::BigInt;
use napi::bindgen_prelude::Buffer as JsBuffer;
use napi::bindgen_prelude::Unknown;
use napi::{Error, JsValue, Result, Status};
use napi_derive::napi;

use std::collections::HashMap;
use std::sync::Mutex;

static OBJECT_URLS: Mutex<Option<HashMap<String, (Vec<u8>, String)>>> = Mutex::new(None);

// ===========================================================================
// `Blob` and `File` classes.
// ===========================================================================

#[napi(object)]
pub struct BlobOptions {
  #[napi(js_name = "type")]
  pub mime_type: Option<String>,
  pub endings: Option<String>,
}

#[napi(object)]
pub struct FileOptions {
  #[napi(js_name = "type")]
  pub mime_type: Option<String>,
  pub endings: Option<String>,
  pub last_modified: Option<BigInt>,
}

#[napi]
pub struct Blob {
  pub(crate) data: Vec<u8>,
  pub(crate) mime_type: String,
}

#[napi]
impl Blob {
  #[napi(constructor)]
  pub fn new(sources: Option<Vec<Unknown>>, options: Option<BlobOptions>) -> Result<Self> {
    let mime = options.as_ref().and_then(|o| o.mime_type.clone()).unwrap_or_default();
    let endings = options.as_ref().and_then(|o| o.endings.clone()).unwrap_or_else(|| "transparent".to_string());

    let mut data = Vec::new();

    if let Some(srcs) = sources {
      for src in srcs {
        if src.is_buffer()? || src.is_arraybuffer()? {
          let buf: JsBuffer = unsafe { src.cast()? };
          data.extend_from_slice(buf.as_ref());
        } else if let Ok(s) = src.coerce_to_string() {
          if let Ok(utf8) = s.into_utf8() {
            if let Ok(mut str_val) = utf8.as_str() {
              let str_buf;
              if endings == "native" {
                str_buf = str_val.replace("\r\n", "\n").replace('\r', "\n");
                str_val = &str_buf;
              }
              data.extend_from_slice(str_val.as_bytes());
            }
          }
        }
      }
    }

    Ok(Self {
      data,
      mime_type: mime,
    })
  }

  pub fn from_data_and_type(data: Vec<u8>, mime_type: String) -> Self {
    Self { data, mime_type }
  }

  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.data.len() as u32
  }

  #[napi(getter, js_name = "type")]
  pub fn get_type(&self) -> String {
    self.mime_type.clone()
  }

  #[napi]
  pub fn array_buffer(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi]
  pub fn slice(&self, start: Option<i64>, end: Option<i64>, mime_type: Option<String>) -> Result<Blob> {
    let len = self.data.len() as i64;
    let s = match start {
      Some(v) if v < 0 => (len + v).max(0) as usize,
      Some(v) => (v.min(len)) as usize,
      None => 0,
    };
    let e = match end {
      Some(v) if v < 0 => (len + v).max(0) as usize,
      Some(v) => (v.min(len)) as usize,
      None => len as usize,
    };
    let s = s.min(e);
    let mime = mime_type.unwrap_or_else(|| self.mime_type.clone());
    Ok(Blob {
      data: self.data[s..e].to_vec(),
      mime_type: mime,
    })
  }

  #[napi]
  pub fn text(&self) -> Result<String> {
    Ok(String::from_utf8_lossy(&self.data).to_string())
  }

  #[napi]
  pub fn stream(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi]
  pub fn text_stream(&self) -> Result<String> {
    self.text()
  }
}

#[napi]
pub struct File {
  data: Vec<u8>,
  mime_type: String,
  name: String,
  last_modified: BigInt,
}

#[napi]
impl File {
  #[napi(constructor)]
  pub fn new(sources: Option<Vec<Unknown>>, file_name: String, options: Option<FileOptions>) -> Result<Self> {
    let blob_opts = options.as_ref().map(|o| BlobOptions {
      mime_type: o.mime_type.clone(),
      endings: o.endings.clone(),
    });
    let blob = Blob::new(sources, blob_opts)?;
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_millis() as u64)
      .unwrap_or(0);
    let last_modified = options.as_ref().and_then(|o| o.last_modified.clone()).unwrap_or_else(|| BigInt::from(now));

    Ok(Self {
      data: blob.data,
      mime_type: blob.mime_type,
      name: file_name,
      last_modified,
    })
  }

  #[napi(getter)]
  pub fn name(&self) -> String {
    self.name.clone()
  }

  #[napi(getter)]
  pub fn last_modified(&self) -> BigInt {
    self.last_modified.clone()
  }

  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.data.len() as u32
  }

  #[napi(getter, js_name = "type")]
  pub fn get_type(&self) -> String {
    self.mime_type.clone()
  }

  #[napi]
  pub fn array_buffer(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi]
  pub fn slice(&self, start: Option<i64>, end: Option<i64>, mime_type: Option<String>) -> Result<Blob> {
    Blob {
      data: self.data.clone(),
      mime_type: self.mime_type.clone(),
    }.slice(start, end, mime_type)
  }

  #[napi]
  pub fn text(&self) -> Result<String> {
    Ok(String::from_utf8_lossy(&self.data).to_string())
  }
}

/// Resolves a `blob:nodedata:...` URL string to its registered `Blob`.
#[napi]
pub fn resolve_object_url(id: String) -> Result<Blob> {
  let guard = OBJECT_URLS.lock().unwrap();
  if let Some(map) = guard.as_ref() {
    if let Some((data, mime_type)) = map.get(&id) {
      return Ok(Blob::from_data_and_type(data.clone(), mime_type.clone()));
    }
  }
  Err(Error::new(Status::GenericFailure, format!("Blob not found for URL: {id}")))
}

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

fn read_uint_n(data: &[u8], off: usize, byte_len: usize, le: bool) -> Result<u64> {
  if byte_len == 0 || byte_len > 8 {
    return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 8"));
  }
  if off + byte_len > data.len() {
    return Err(Error::new(Status::GenericFailure, "out of range"));
  }
  let mut val: u64 = 0;
  if le {
    for (i, &b) in data[off..off + byte_len].iter().enumerate() {
      val |= (b as u64) << (i * 8);
    }
  } else {
    for &b in &data[off..off + byte_len] {
      val = (val << 8) | (b as u64);
    }
  }
  Ok(val)
}

fn read_int_n(data: &[u8], off: usize, byte_len: usize, le: bool) -> Result<i64> {
  let uval = read_uint_n(data, off, byte_len, le)?;
  if byte_len == 8 {
    return Ok(uval as i64);
  }
  let sign_bit = 1u64 << (byte_len * 8 - 1);
  if (uval & sign_bit) != 0 {
    let mask = !((1u64 << (byte_len * 8)) - 1);
    Ok((uval | mask) as i64)
  } else {
    Ok(uval as i64)
  }
}

fn write_uint_n(data: &mut [u8], off: usize, byte_len: usize, val: u64, le: bool) -> Result<u32> {
  if byte_len == 0 || byte_len > 8 {
    return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 8"));
  }
  if off + byte_len > data.len() {
    return Err(Error::new(Status::GenericFailure, "out of range"));
  }
  if le {
    for i in 0..byte_len {
      data[off + i] = ((val >> (i * 8)) & 0xff) as u8;
    }
  } else {
    for i in 0..byte_len {
      data[off + byte_len - 1 - i] = ((val >> (i * 8)) & 0xff) as u8;
    }
  }
  Ok((off + byte_len) as u32)
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
  #[napi]
  pub fn alloc_unsafe(length: u32, _alignment: Option<u32>) -> Buffer {
    Buffer::new(length)
  }

  /// Allocates an unpooled uninitialized buffer of `length` bytes.
  #[napi]
  pub fn alloc_unsafe_slow(length: u32, _alignment: Option<u32>) -> Buffer {
    Buffer::new(length)
  }

  /// Calculates byte length of `string` or buffer view.
  #[napi]
  pub fn byte_length(string: Unknown, encoding: Option<String>) -> Result<u32> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    if let Ok(s) = string.coerce_to_string() {
      if let Ok(utf8) = s.into_utf8() {
        if let Ok(str_val) = utf8.as_str() {
          let bytes = str_to_bytes(str_val, &enc)?;
          return Ok(bytes.len() as u32);
        }
      }
    } else if string.is_buffer()? || string.is_arraybuffer()? {
      let buf: JsBuffer = unsafe { string.cast()? };
      return Ok(buf.as_ref().len() as u32);
    }
    Ok(0)
  }

  /// Returns string length in UTF-16 code units.
  #[napi]
  pub fn string_length(input: Unknown, encoding: Option<String>) -> Result<u32> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let bytes = if input.is_buffer()? || input.is_arraybuffer()? {
      let buf: JsBuffer = unsafe { input.cast()? };
      buf.as_ref().to_vec()
    } else if let Ok(s) = input.coerce_to_string() {
      if let Ok(utf8) = s.into_utf8() {
        if let Ok(str_val) = utf8.as_str() {
          str_val.as_bytes().to_vec()
        } else { Vec::new() }
      } else { Vec::new() }
    } else {
      Vec::new()
    };

    let str_val = bytes_to_str(&bytes, &enc)?;
    Ok(str_val.encode_utf16().count() as u32)
  }

  /// Copies memory from `view` into a new Buffer.
  #[napi]
  pub fn copy_bytes_from(view: JsBuffer, offset: Option<u32>, length: Option<u32>) -> Result<Buffer> {
    let bytes = view.as_ref();
    let off = offset.unwrap_or(0) as usize;
    let len = length.unwrap_or((bytes.len().saturating_sub(off)) as u32) as usize;
    if off > bytes.len() || off + len > bytes.len() {
      return Err(Error::new(Status::GenericFailure, "out of bounds"));
    }
    Ok(Buffer {
      data: bytes[off..off + len].to_vec(),
    })
  }

  /// Static compare helper.
  #[napi]
  pub fn compare_static(buf1: JsBuffer, buf2: JsBuffer) -> i32 {
    buf1.as_ref().cmp(buf2.as_ref()) as i32
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

  /// The byteOffset of the underlying ArrayBuffer.
  #[napi(getter)]
  pub fn byte_offset(&self) -> u32 {
    0
  }

  /// The underlying ArrayBuffer object.
  #[napi(getter)]
  pub fn buffer(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  /// Deprecated alias for buffer.
  #[napi(getter)]
  pub fn parent(&self) -> JsBuffer {
    self.buffer()
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

  #[napi(js_name = "readUInt16LE")]
  pub fn read_u_int16_le(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    Ok(u16::from_le_bytes([self.data[o], self.data[o + 1]]) as u32)
  }

  #[napi(js_name = "readUInt16BE")]
  pub fn read_u_int16_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    Ok(u16::from_be_bytes([self.data[o], self.data[o + 1]]) as u32)
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

  #[napi(js_name = "readUInt32LE")]
  pub fn read_u_int32_le(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(u32::from_le_bytes(b))
  }

  #[napi(js_name = "readUInt32BE")]
  pub fn read_u_int32_be(&self, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    let b: [u8; 4] = self.data[o..o + 4].try_into().unwrap();
    Ok(u32::from_be_bytes(b))
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

  #[napi(js_name = "readBigUInt64LE")]
  pub fn read_big_u_int64_le(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(u64::from_le_bytes(b)))
  }

  #[napi(js_name = "readBigUInt64BE")]
  pub fn read_big_u_int64_be(&self, offset: Option<u32>) -> Result<BigInt> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let b: [u8; 8] = self.data[o..o + 8].try_into().unwrap();
    Ok(BigInt::from(u64::from_be_bytes(b)))
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

  // Aliases for uint reads
  #[napi]
  pub fn read_uint8(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int8(offset)
  }

  #[napi]
  pub fn read_uint16_le(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_le(offset)
  }

  #[napi]
  pub fn read_uint16_be(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_be(offset)
  }

  #[napi]
  pub fn read_uint32_le(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_le(offset)
  }

  #[napi]
  pub fn read_uint32_be(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_be(offset)
  }

  #[napi]
  pub fn read_big_uint64_le(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_u_int64_le(offset)
  }

  #[napi]
  pub fn read_big_uint64_be(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_u_int64_be(offset)
  }

  // Variable byte length reads
  #[napi(js_name = "readIntBE")]
  pub fn read_int_be(&self, offset: u32, byte_length: u32) -> Result<BigInt> {
    let val = read_int_n(&self.data, offset as usize, byte_length as usize, false)?;
    Ok(BigInt::from(val))
  }

  #[napi(js_name = "readIntLE")]
  pub fn read_int_le(&self, offset: u32, byte_length: u32) -> Result<BigInt> {
    let val = read_int_n(&self.data, offset as usize, byte_length as usize, true)?;
    Ok(BigInt::from(val))
  }

  #[napi(js_name = "readUIntBE")]
  pub fn read_u_int_be(&self, offset: u32, byte_length: u32) -> Result<BigInt> {
    let val = read_uint_n(&self.data, offset as usize, byte_length as usize, false)?;
    Ok(BigInt::from(val))
  }

  #[napi(js_name = "readUIntLE")]
  pub fn read_u_int_le(&self, offset: u32, byte_length: u32) -> Result<BigInt> {
    let val = read_uint_n(&self.data, offset as usize, byte_length as usize, true)?;
    Ok(BigInt::from(val))
  }

  #[napi(js_name = "readUintBE")]
  pub fn read_uint_be(&self, offset: u32, byte_length: u32) -> Result<BigInt> {
    self.read_u_int_be(offset, byte_length)
  }

  #[napi(js_name = "readUintLE")]
  pub fn read_uint_le(&self, offset: u32, byte_length: u32) -> Result<BigInt> {
    self.read_u_int_le(offset, byte_length)
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

  #[napi(js_name = "writeUInt16LE")]
  pub fn write_u_int16_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as u16).to_le_bytes());
    Ok((o + 2) as u32)
  }

  #[napi(js_name = "writeUInt16BE")]
  pub fn write_u_int16_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 2)?;
    self.data[o..o + 2].copy_from_slice(&(value as u16).to_be_bytes());
    Ok((o + 2) as u32)
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

  #[napi(js_name = "writeUInt32LE")]
  pub fn write_u_int32_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_le_bytes());
    Ok((o + 4) as u32)
  }

  #[napi(js_name = "writeUInt32BE")]
  pub fn write_u_int32_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 4)?;
    self.data[o..o + 4].copy_from_slice(&value.to_be_bytes());
    Ok((o + 4) as u32)
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

  #[napi(js_name = "writeBigUInt64LE")]
  pub fn write_big_u_int64_le(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let (_, v, _) = value.get_u64();
    self.data[o..o + 8].copy_from_slice(&v.to_le_bytes());
    Ok((o + 8) as u32)
  }

  #[napi(js_name = "writeBigUInt64BE")]
  pub fn write_big_u_int64_be(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    let o = Self::off(offset);
    self.check(o, 8)?;
    let (_, v, _) = value.get_u64();
    self.data[o..o + 8].copy_from_slice(&v.to_be_bytes());
    Ok((o + 8) as u32)
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

  // Aliases for uint writes
  #[napi]
  pub fn write_uint8(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int8(value, offset)
  }

  #[napi]
  pub fn write_uint16_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_le(value, offset)
  }

  #[napi]
  pub fn write_uint16_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_be(value, offset)
  }

  #[napi]
  pub fn write_uint32_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_le(value, offset)
  }

  #[napi]
  pub fn write_uint32_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_be(value, offset)
  }

  #[napi]
  pub fn write_big_uint64_le(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_le(value, offset)
  }

  #[napi]
  pub fn write_big_uint64_be(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_be(value, offset)
  }

  // Variable byte length writes
  #[napi(js_name = "writeIntBE")]
  pub fn write_int_be(&mut self, value: BigInt, offset: u32, byte_length: u32) -> Result<u32> {
    let (v, _) = value.get_i64();
    write_uint_n(&mut self.data, offset as usize, byte_length as usize, v as u64, false)
  }

  #[napi(js_name = "writeIntLE")]
  pub fn write_int_le(&mut self, value: BigInt, offset: u32, byte_length: u32) -> Result<u32> {
    let (v, _) = value.get_i64();
    write_uint_n(&mut self.data, offset as usize, byte_length as usize, v as u64, true)
  }

  #[napi(js_name = "writeUIntBE")]
  pub fn write_u_int_be(&mut self, value: BigInt, offset: u32, byte_length: u32) -> Result<u32> {
    let (_, v, _) = value.get_u64();
    write_uint_n(&mut self.data, offset as usize, byte_length as usize, v, false)
  }

  #[napi(js_name = "writeUIntLE")]
  pub fn write_u_int_le(&mut self, value: BigInt, offset: u32, byte_length: u32) -> Result<u32> {
    let (_, v, _) = value.get_u64();
    write_uint_n(&mut self.data, offset as usize, byte_length as usize, v, true)
  }

  #[napi(js_name = "writeUintBE")]
  pub fn write_uint_be(&mut self, value: BigInt, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_be(value, offset, byte_length)
  }

  #[napi(js_name = "writeUintLE")]
  pub fn write_uint_le(&mut self, value: BigInt, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_le(value, offset, byte_length)
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
      json_type: "Buffer".to_string(),
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
    self.data.as_slice().cmp(other.as_ref()) as i32
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

  #[napi]
  pub fn last_index_of_number(&self, value: u32, offset: Option<u32>) -> i32 {
    let b = (value & 0xff) as u8;
    let max_off = offset.unwrap_or(self.data.len().saturating_sub(1) as u32) as usize;
    let limit = (max_off + 1).min(self.data.len());
    match self.data[..limit].iter().rposition(|x| *x == b) {
      Some(p) => p as i32,
      None => -1,
    }
  }

  #[napi]
  pub fn last_index_of_string(&self, value: String, offset: Option<u32>, encoding: Option<String>) -> Result<i32> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    let bytes = str_to_bytes(&value, &enc)?;
    if bytes.is_empty() {
      return Ok(offset.unwrap_or(self.data.len() as u32) as i32);
    }
    let max_off = offset.unwrap_or(self.data.len().saturating_sub(bytes.len()) as u32) as usize;
    let limit = max_off.min(self.data.len().saturating_sub(bytes.len()));
    for i in (0..=limit).rev() {
      if self.data[i..i + bytes.len()] == bytes[..] {
        return Ok(i as i32);
      }
    }
    Ok(-1)
  }

  #[napi]
  pub fn copy(
    &self,
    target: &mut Buffer,
    target_start: Option<u32>,
    source_start: Option<u32>,
    source_end: Option<u32>,
  ) -> Result<u32> {
    let t_start = target_start.unwrap_or(0) as usize;
    let s_start = source_start.unwrap_or(0) as usize;
    let s_end = source_end.unwrap_or(self.data.len() as u32) as usize;

    if s_start >= self.data.len() || s_start >= s_end {
      return Ok(0);
    }
    let copy_len = (s_end - s_start).min(target.data.len().saturating_sub(t_start));
    if copy_len == 0 {
      return Ok(0);
    }

    target.data[t_start..t_start + copy_len].copy_from_slice(&self.data[s_start..s_start + copy_len]);
    Ok(copy_len as u32)
  }

  #[napi]
  pub fn keys(&self) -> Vec<u32> {
    (0..self.data.len() as u32).collect()
  }

  #[napi]
  pub fn values(&self) -> Vec<u32> {
    self.data.iter().map(|b| *b as u32).collect()
  }

  #[napi]
  pub fn entries(&self) -> Vec<Vec<u32>> {
    self.data.iter().enumerate().map(|(i, b)| vec![i as u32, *b as u32]).collect()
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
  #[napi(js_name = "type")]
  pub json_type: String,
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

/// Returns true if every UTF-16 code unit of `input` is in the range U+0000 to U+00FF.
#[napi]
pub fn is_latin1(input: String) -> bool {
  input.encode_utf16().all(|c| c <= 0xff)
}

/// Re-encodes `source` from `from_encoding` to `to_encoding`.
#[napi]
pub fn transcode(source: JsBuffer, from_encoding: String, to_encoding: String) -> Result<JsBuffer> {
  let text = bytes_to_str(source.as_ref(), &from_encoding)?;
  let target_enc = to_encoding.to_ascii_lowercase();
  let out = match target_enc.as_str() {
    "ascii" => {
      text.chars().map(|c| if (c as u32) <= 127 { c as u8 } else { b'?' }).collect()
    }
    "latin1" | "binary" => {
      text.chars().map(|c| if (c as u32) <= 255 { c as u8 } else { b'?' }).collect()
    }
    _ => str_to_bytes(&text, &to_encoding)?,
  };
  Ok(JsBuffer::from(out.as_slice()))
}

/// Constant: maximum number of bytes returned when `buf.inspect()` is called.
#[napi]
pub fn inspect_max_bytes() -> u32 {
  50
}

/// Constant: largest size allowed for a single `Buffer` instance.
#[napi]
pub fn k_max_length() -> BigInt {
  BigInt::from(9_007_199_254_740_991u64)
}

/// Constant: largest length allowed for a single `string` instance.
#[napi]
pub fn k_string_max_length() -> u32 {
  536_870_888
}

/// Returns the `node:buffer` constants.
#[napi]
pub fn constants() -> BufferConstants {
  BufferConstants {
    max_length: BigInt::from(9_007_199_254_740_991u64),
    max_string_length: 536_870_888,
    inspect_max_bytes: 50,
    k_max_length: BigInt::from(9_007_199_254_740_991u64),
    k_string_max_length: 536_870_888,
  }
}

#[napi(object)]
pub struct BufferConstants {
  pub max_length: BigInt,
  pub max_string_length: u32,
  pub inspect_max_bytes: u32,
  pub k_max_length: BigInt,
  pub k_string_max_length: u32,
}
