//! NAPI bindings for a `node:buffer`-compatible `Buffer`, `Blob`, and `File` API.

use std::sync::atomic::{AtomicU32, Ordering};
use napi::bindgen_prelude::{BigInt, Buffer as JsBuffer, FromNapiValue, ToNapiValue, Unknown};
use napi::{Env, Error, JsValue, Result, Status, ValueType};
use napi_derive::napi;

static POOL_SIZE: AtomicU32 = AtomicU32::new(65536);

fn parse_offset(val: Option<f64>, len: i64, default_val: i64) -> i64 {
  match val {
    None => default_val,
    Some(v) => {
      let i = v as i64;
      if i < 0 {
        (len + i).max(0)
      } else {
        i
      }
    }
  }
}

// ===========================================================================
// `Blob` and `File` classes.
// ===========================================================================

#[napi(object)]
#[derive(Default)]
pub struct BlobOptions {
  pub endings: Option<String>,
  pub r#type: Option<String>,
}

#[napi(object)]
#[derive(Default)]
pub struct FileOptions {
  pub endings: Option<String>,
  pub r#type: Option<String>,
  pub last_modified: Option<f64>,
}

#[napi]
pub struct Blob {
  pub(crate) data: Vec<u8>,
  pub(crate) content_type: String,
}

#[napi]
impl Blob {
  #[napi(constructor)]
  pub fn new(
    env: Env,
    #[napi(ts_arg_type = "Array<any>")] sources: Option<Vec<Unknown>>,
    options: Option<BlobOptions>,
  ) -> Result<Self> {
    let opts = options.unwrap_or_default();
    let content_type = opts.r#type.unwrap_or_default();
    let endings = opts.endings.unwrap_or_else(|| "transparent".to_string());
    let is_native = endings.to_ascii_lowercase() == "native";

    let mut data = Vec::new();
    if let Some(src_list) = sources {
      for src in src_list {
        let env_raw = env.raw();
        let src_raw = src.raw();
        if src.is_buffer()? {
          let b = unsafe { JsBuffer::from_napi_value(env_raw, src_raw)? };
          data.extend_from_slice(b.as_ref());
        } else if src.get_type()? == ValueType::String {
          let s: String = unsafe { String::from_napi_value(env_raw, src_raw)? };
          if is_native {
            let eol = if cfg!(windows) { "\r\n" } else { "\n" };
            let s_norm = s.replace("\r\n", "\n");
            let s_converted = if eol == "\r\n" {
              s_norm.replace('\n', "\r\n")
            } else {
              s_norm
            };
            data.extend_from_slice(s_converted.as_bytes());
          } else {
            data.extend_from_slice(s.as_bytes());
          }
        } else if let Ok(b) = unsafe { JsBuffer::from_napi_value(env_raw, src_raw) } {
          data.extend_from_slice(b.as_ref());
        }
      }
    }

    Ok(Self {
      data,
      content_type,
    })
  }

  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.data.len() as u32
  }

  #[napi(getter, js_name = "type")]
  pub fn type_name(&self) -> String {
    self.content_type.clone()
  }

  #[napi]
  pub async fn array_buffer(&self) -> Result<JsBuffer> {
    Ok(JsBuffer::from(self.data.as_slice()))
  }

  #[napi]
  pub async fn bytes(&self) -> Result<JsBuffer> {
    Ok(JsBuffer::from(self.data.as_slice()))
  }

  #[napi]
  pub async fn text(&self) -> Result<String> {
    Ok(String::from_utf8_lossy(&self.data).into_owned())
  }

  #[napi]
  pub fn slice(&self, start: Option<f64>, end: Option<f64>, content_type: Option<String>) -> Blob {
    let len = self.data.len() as i64;
    let s = parse_offset(start, len, 0);
    let e = parse_offset(end, len, len);
    let s = s.clamp(0, len) as usize;
    let e = e.clamp(s as i64, len) as usize;

    Blob {
      data: self.data[s..e].to_vec(),
      content_type: content_type.unwrap_or_default(),
    }
  }

  #[napi]
  pub fn stream(&self, env: Env) -> Result<Unknown<'_>> {
    let buf = JsBuffer::from(self.data.as_slice());
    let val = unsafe { ToNapiValue::to_napi_value(env.raw(), buf)? };
    Ok(unsafe { Unknown::from_napi_value(env.raw(), val)? })
  }

  #[napi]
  pub fn text_stream(&self, env: Env) -> Result<Unknown<'_>> {
    let text = String::from_utf8_lossy(&self.data).into_owned();
    let val = unsafe { ToNapiValue::to_napi_value(env.raw(), text)? };
    Ok(unsafe { Unknown::from_napi_value(env.raw(), val)? })
  }
}

#[napi]
pub struct File {
  pub(crate) blob: Blob,
  pub(crate) file_name: String,
  pub(crate) last_modified_ms: f64,
}

#[napi]
impl File {
  #[napi(constructor)]
  pub fn new(
    env: Env,
    #[napi(ts_arg_type = "Array<any>")] sources: Option<Vec<Unknown>>,
    file_name: String,
    options: Option<FileOptions>,
  ) -> Result<Self> {
    let opts = options.unwrap_or_default();
    let blob_opts = BlobOptions {
      endings: opts.endings,
      r#type: opts.r#type,
    };
    let blob = Blob::new(env, sources, Some(blob_opts))?;
    let last_modified_ms = opts.last_modified.unwrap_or_else(|| {
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0)
    });

    Ok(Self {
      blob,
      file_name,
      last_modified_ms,
    })
  }

  #[napi(getter)]
  pub fn name(&self) -> String {
    self.file_name.clone()
  }

  #[napi(getter)]
  pub fn last_modified(&self) -> f64 {
    self.last_modified_ms
  }

  #[napi(getter)]
  pub fn size(&self) -> u32 {
    self.blob.size()
  }

  #[napi(getter, js_name = "type")]
  pub fn type_name(&self) -> String {
    self.blob.type_name()
  }

  #[napi]
  pub async fn array_buffer(&self) -> Result<JsBuffer> {
    self.blob.array_buffer().await
  }

  #[napi]
  pub async fn bytes(&self) -> Result<JsBuffer> {
    self.blob.bytes().await
  }

  #[napi]
  pub async fn text(&self) -> Result<String> {
    self.blob.text().await
  }

  #[napi]
  pub fn slice(&self, start: Option<f64>, end: Option<f64>, content_type: Option<String>) -> Blob {
    self.blob.slice(start, end, content_type)
  }

  #[napi]
  pub fn stream(&self, env: Env) -> Result<Unknown<'_>> {
    self.blob.stream(env)
  }

  #[napi]
  pub fn text_stream(&self, env: Env) -> Result<Unknown<'_>> {
    self.blob.text_stream(env)
  }
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
    "utf16le" | "ucs2" | "utf-16le" | "ucs-2" => {
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
    "utf16le" | "ucs2" | "utf-16le" | "ucs-2" => {
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
  "utf-16le",
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
  pub(crate) data: Vec<u8>,
}

#[napi]
impl Buffer {
  #[napi(constructor)]
  pub fn new(length: u32) -> Self {
    Self {
      data: vec![0u8; length as usize],
    }
  }

  #[napi(getter)]
  pub fn pool_size() -> u32 {
    POOL_SIZE.load(Ordering::Relaxed)
  }

  #[napi(setter)]
  pub fn set_pool_size(value: u32) {
    POOL_SIZE.store(value, Ordering::Relaxed);
  }

  #[napi]
  pub fn alloc(env: Env, length: u32, #[napi(ts_arg_type = "any")] fill: Option<Unknown>, encoding: Option<String>) -> Result<Buffer> {
    let mut b = Buffer::new(length);
    if let Some(f) = fill {
      let env_raw = env.raw();
      let f_raw = f.raw();
      if f.get_type()? == ValueType::Number {
        let num: f64 = unsafe { f64::from_napi_value(env_raw, f_raw)? };
        let byte_val = (num as i32 & 0xff) as u8;
        b.data.iter_mut().for_each(|x| *x = byte_val);
      } else if f.get_type()? == ValueType::String {
        let s: String = unsafe { String::from_napi_value(env_raw, f_raw)? };
        let enc = encoding.unwrap_or_else(|| "utf8".to_string());
        let fill_bytes = str_to_bytes(&s, &enc)?;
        if !fill_bytes.is_empty() {
          for (i, v) in b.data.iter_mut().enumerate() {
            *v = fill_bytes[i % fill_bytes.len()];
          }
        }
      } else if f.is_buffer()? {
        let buf = unsafe { JsBuffer::from_napi_value(env_raw, f_raw)? };
        let fill_bytes = buf.as_ref();
        if !fill_bytes.is_empty() {
          for (i, v) in b.data.iter_mut().enumerate() {
            *v = fill_bytes[i % fill_bytes.len()];
          }
        }
      }
    }
    Ok(b)
  }

  #[napi]
  pub fn alloc_unsafe(length: u32, _alignment: Option<u32>) -> Buffer {
    Buffer::new(length)
  }

  #[napi]
  pub fn alloc_unsafe_slow(length: u32, _alignment: Option<u32>) -> Buffer {
    Buffer::new(length)
  }

  #[napi]
  pub fn byte_length(env: Env, #[napi(ts_arg_type = "any")] input: Unknown, encoding: Option<String>) -> Result<u32> {
    let env_raw = env.raw();
    let input_raw = input.raw();
    if input.get_type()? == ValueType::String {
      let s: String = unsafe { String::from_napi_value(env_raw, input_raw)? };
      let enc = encoding.unwrap_or_else(|| "utf8".to_string());
      let bytes = str_to_bytes(&s, &enc)?;
      Ok(bytes.len() as u32)
    } else if input.is_buffer()? {
      let b = unsafe { JsBuffer::from_napi_value(env_raw, input_raw)? };
      Ok(b.as_ref().len() as u32)
    } else if let Ok(b) = unsafe { JsBuffer::from_napi_value(env_raw, input_raw) } {
      Ok(b.as_ref().len() as u32)
    } else {
      Err(Error::new(Status::GenericFailure, "invalid input for byteLength"))
    }
  }

  #[napi]
  pub fn string_length(input: JsBuffer, encoding: Option<String>) -> u32 {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string()).to_ascii_lowercase();
    let bytes = input.as_ref();
    match enc.as_str() {
      "utf8" | "utf-8" => {
        let s = String::from_utf8_lossy(bytes);
        s.encode_utf16().count() as u32
      }
      "utf16le" | "ucs2" | "utf-16le" | "ucs-2" => (bytes.len() / 2) as u32,
      "hex" => (bytes.len() * 2) as u32,
      _ => bytes.len() as u32,
    }
  }

  #[napi]
  pub fn compare_static(buf1: JsBuffer, buf2: JsBuffer) -> i32 {
    buf1.as_ref().cmp(buf2.as_ref()) as i32
  }

  #[napi(factory)]
  pub fn from_buffer(buffer: JsBuffer) -> Buffer {
    Buffer {
      data: buffer.as_ref().to_vec(),
    }
  }

  #[napi(factory)]
  pub fn from_string(string: String, encoding: Option<String>) -> Result<Buffer> {
    let enc = encoding.unwrap_or_else(|| "utf8".to_string());
    Ok(Buffer {
      data: str_to_bytes(&string, &enc)?,
    })
  }

  #[napi(factory)]
  pub fn from_array(items: Vec<f64>) -> Buffer {
    Buffer {
      data: items
        .iter()
        .map(|v| ((*v as i32) & 0xff) as u8)
        .collect(),
    }
  }

  #[napi(factory)]
  pub fn from_object(env: Env, #[napi(ts_arg_type = "any")] object: Unknown, #[napi(ts_arg_type = "any")] offset_or_encoding: Option<Unknown>, _length: Option<u32>) -> Result<Buffer> {
    let env_raw = env.raw();
    let obj_raw = object.raw();
    if object.get_type()? == ValueType::String {
      let s: String = unsafe { String::from_napi_value(env_raw, obj_raw)? };
      let enc = if let Some(oe) = offset_or_encoding {
        if oe.get_type()? == ValueType::String {
          unsafe { String::from_napi_value(env_raw, oe.raw())? }
        } else {
          "utf8".to_string()
        }
      } else {
        "utf8".to_string()
      };
      return Ok(Buffer {
        data: str_to_bytes(&s, &enc)?,
      });
    } else if object.is_buffer()? {
      let b = unsafe { JsBuffer::from_napi_value(env_raw, obj_raw)? };
      return Ok(Buffer {
        data: b.as_ref().to_vec(),
      });
    } else if let Ok(b) = unsafe { JsBuffer::from_napi_value(env_raw, obj_raw) } {
      return Ok(Buffer {
        data: b.as_ref().to_vec(),
      });
    }
    Err(Error::new(Status::GenericFailure, "invalid object for Buffer.from"))
  }

  #[napi(factory)]
  pub fn from_array_buffer(array_buffer: JsBuffer, byte_offset: Option<u32>, length: Option<u32>) -> Result<Buffer> {
    let bytes = array_buffer.as_ref();
    let off = byte_offset.unwrap_or(0) as usize;
    if off > bytes.len() {
      return Err(Error::new(Status::GenericFailure, "byteOffset out of range"));
    }
    let max_len = bytes.len() - off;
    let len = length.unwrap_or(max_len as u32) as usize;
    let len = len.min(max_len);
    Ok(Buffer {
      data: bytes[off..off + len].to_vec(),
    })
  }

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

  #[napi]
  pub fn copy_bytes_from(view: JsBuffer, offset: Option<u32>, length: Option<u32>) -> Result<Buffer> {
    let bytes = view.as_ref();
    let off = offset.unwrap_or(0) as usize;
    if off > bytes.len() {
      return Err(Error::new(Status::GenericFailure, "offset out of bounds"));
    }
    let available = bytes.len() - off;
    let len = length.unwrap_or(available as u32) as usize;
    let len = len.min(available);
    Ok(Buffer {
      data: bytes[off..off + len].to_vec(),
    })
  }

  #[napi]
  pub fn is_buffer(value: Unknown) -> Result<bool> {
    value.is_buffer()
  }

  #[napi]
  pub fn is_encoding(encoding: String) -> bool {
    SUPPORTED_ENCODINGS.contains(&encoding.to_ascii_lowercase().as_str())
  }

  #[napi(getter)]
  pub fn length(&self) -> u32 {
    self.data.len() as u32
  }

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

  #[napi]
  pub fn read_u_int_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let len = byte_length as usize;
    if !(1..=6).contains(&len) {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, len)?;
    let mut val: u64 = 0;
    for i in 0..len {
      val = (val << 8) | (self.data[o + i] as u64);
    }
    Ok(val as f64)
  }

  #[napi]
  pub fn read_u_int_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let o = offset as usize;
    let len = byte_length as usize;
    if !(1..=6).contains(&len) {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, len)?;
    let mut val: u64 = 0;
    for i in (0..len).rev() {
      val = (val << 8) | (self.data[o + i] as u64);
    }
    Ok(val as f64)
  }

  #[napi]
  pub fn read_int_be(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let u_val = self.read_u_int_be(offset, byte_length)? as u64;
    let len = byte_length as usize;
    let max = 1u64 << (len * 8);
    if u_val >= (max >> 1) {
      Ok((u_val as i64 - max as i64) as f64)
    } else {
      Ok(u_val as f64)
    }
  }

  #[napi]
  pub fn read_int_le(&self, offset: u32, byte_length: u32) -> Result<f64> {
    let u_val = self.read_u_int_le(offset, byte_length)? as u64;
    let len = byte_length as usize;
    let max = 1u64 << (len * 8);
    if u_val >= (max >> 1) {
      Ok((u_val as i64 - max as i64) as f64)
    } else {
      Ok(u_val as f64)
    }
  }

  #[napi(js_name = "readUint8")]
  pub fn read_uint8(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int8(offset)
  }

  #[napi(js_name = "readUInt16BE")]
  pub fn read_u_int16_be_alias(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_be(offset)
  }

  #[napi(js_name = "readUint16BE")]
  pub fn read_uint16_be(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_be(offset)
  }

  #[napi(js_name = "readUInt16LE")]
  pub fn read_u_int16_le_alias(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_le(offset)
  }

  #[napi(js_name = "readUint16LE")]
  pub fn read_uint16_le(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int16_le(offset)
  }

  #[napi(js_name = "readInt16BE")]
  pub fn read_int16_be_alias(&self, offset: Option<u32>) -> Result<i32> {
    self.read_int16_be(offset)
  }

  #[napi(js_name = "readInt16LE")]
  pub fn read_int16_le_alias(&self, offset: Option<u32>) -> Result<i32> {
    self.read_int16_le(offset)
  }

  #[napi(js_name = "readUInt32BE")]
  pub fn read_u_int32_be_alias(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_be(offset)
  }

  #[napi(js_name = "readUint32BE")]
  pub fn read_uint32_be(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_be(offset)
  }

  #[napi(js_name = "readUInt32LE")]
  pub fn read_u_int32_le_alias(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_le(offset)
  }

  #[napi(js_name = "readUint32LE")]
  pub fn read_uint32_le(&self, offset: Option<u32>) -> Result<u32> {
    self.read_u_int32_le(offset)
  }

  #[napi(js_name = "readInt32BE")]
  pub fn read_int32_be_alias(&self, offset: Option<u32>) -> Result<i32> {
    self.read_int32_be(offset)
  }

  #[napi(js_name = "readInt32LE")]
  pub fn read_int32_le_alias(&self, offset: Option<u32>) -> Result<i32> {
    self.read_int32_le(offset)
  }

  #[napi(js_name = "readFloatBE")]
  pub fn read_float_be_alias(&self, offset: Option<u32>) -> Result<f64> {
    self.read_float_be(offset)
  }

  #[napi(js_name = "readFloatLE")]
  pub fn read_float_le_alias(&self, offset: Option<u32>) -> Result<f64> {
    self.read_float_le(offset)
  }

  #[napi(js_name = "readDoubleBE")]
  pub fn read_double_be_alias(&self, offset: Option<u32>) -> Result<f64> {
    self.read_double_be(offset)
  }

  #[napi(js_name = "readDoubleLE")]
  pub fn read_double_le_alias(&self, offset: Option<u32>) -> Result<f64> {
    self.read_double_le(offset)
  }

  #[napi(js_name = "readBigUInt64BE")]
  pub fn read_big_u_int64_be_alias(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_u_int64_be(offset)
  }

  #[napi(js_name = "readBigUint64BE")]
  pub fn read_big_uint64_be(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_u_int64_be(offset)
  }

  #[napi(js_name = "readBigUInt64LE")]
  pub fn read_big_u_int64_le_alias(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_u_int64_le(offset)
  }

  #[napi(js_name = "readBigUint64LE")]
  pub fn read_big_uint64_le(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_u_int64_le(offset)
  }

  #[napi(js_name = "readBigInt64BE")]
  pub fn read_big_int64_be_alias(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_int64_be(offset)
  }

  #[napi(js_name = "readBigInt64LE")]
  pub fn read_big_int64_le_alias(&self, offset: Option<u32>) -> Result<BigInt> {
    self.read_big_int64_le(offset)
  }

  #[napi(js_name = "readUIntBE")]
  pub fn read_u_int_be_alias(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_u_int_be(offset, byte_length)
  }

  #[napi(js_name = "readUintBE")]
  pub fn read_uint_be_alias(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_u_int_be(offset, byte_length)
  }

  #[napi(js_name = "readUIntLE")]
  pub fn read_u_int_le_alias(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_u_int_le(offset, byte_length)
  }

  #[napi(js_name = "readUintLE")]
  pub fn read_uint_le_alias(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_u_int_le(offset, byte_length)
  }

  #[napi(js_name = "readIntBE")]
  pub fn read_int_be_alias(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_int_be(offset, byte_length)
  }

  #[napi(js_name = "readIntLE")]
  pub fn read_int_le_alias(&self, offset: u32, byte_length: u32) -> Result<f64> {
    self.read_int_le(offset, byte_length)
  }

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

  #[napi]
  pub fn write_u_int_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    let o = offset as usize;
    let len = byte_length as usize;
    if !(1..=6).contains(&len) {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, len)?;
    let mut val = value as u64;
    for i in (0..len).rev() {
      self.data[o + i] = (val & 0xff) as u8;
      val >>= 8;
    }
    Ok((o + len) as u32)
  }

  #[napi]
  pub fn write_u_int_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    let o = offset as usize;
    let len = byte_length as usize;
    if !(1..=6).contains(&len) {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, len)?;
    let mut val = value as u64;
    for i in 0..len {
      self.data[o + i] = (val & 0xff) as u8;
      val >>= 8;
    }
    Ok((o + len) as u32)
  }

  #[napi]
  pub fn write_int_be(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    let o = offset as usize;
    let len = byte_length as usize;
    if !(1..=6).contains(&len) {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, len)?;
    let mut val = (value as i64) as u64;
    for i in (0..len).rev() {
      self.data[o + i] = (val & 0xff) as u8;
      val >>= 8;
    }
    Ok((o + len) as u32)
  }

  #[napi]
  pub fn write_int_le(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    let o = offset as usize;
    let len = byte_length as usize;
    if !(1..=6).contains(&len) {
      return Err(Error::new(Status::GenericFailure, "byteLength must be between 1 and 6"));
    }
    self.check(o, len)?;
    let mut val = (value as i64) as u64;
    for i in 0..len {
      self.data[o + i] = (val & 0xff) as u8;
      val >>= 8;
    }
    Ok((o + len) as u32)
  }

  #[napi(js_name = "writeUint8")]
  pub fn write_uint8(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int8(value, offset)
  }

  #[napi(js_name = "writeUInt16BE")]
  pub fn write_u_int16_be_alias(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_be(value, offset)
  }

  #[napi(js_name = "writeUint16BE")]
  pub fn write_uint16_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_be(value, offset)
  }

  #[napi(js_name = "writeUInt16LE")]
  pub fn write_u_int16_le_alias(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_le(value, offset)
  }

  #[napi(js_name = "writeUint16LE")]
  pub fn write_uint16_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int16_le(value, offset)
  }

  #[napi(js_name = "writeInt16BE")]
  pub fn write_int16_be_alias(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    self.write_int16_be(value, offset)
  }

  #[napi(js_name = "writeInt16LE")]
  pub fn write_int16_le_alias(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    self.write_int16_le(value, offset)
  }

  #[napi(js_name = "writeUInt32BE")]
  pub fn write_u_int32_be_alias(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_be(value, offset)
  }

  #[napi(js_name = "writeUint32BE")]
  pub fn write_uint32_be(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_be(value, offset)
  }

  #[napi(js_name = "writeUInt32LE")]
  pub fn write_u_int32_le_alias(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_le(value, offset)
  }

  #[napi(js_name = "writeUint32LE")]
  pub fn write_uint32_le(&mut self, value: u32, offset: Option<u32>) -> Result<u32> {
    self.write_u_int32_le(value, offset)
  }

  #[napi(js_name = "writeInt32BE")]
  pub fn write_int32_be_alias(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    self.write_int32_be(value, offset)
  }

  #[napi(js_name = "writeInt32LE")]
  pub fn write_int32_le_alias(&mut self, value: i32, offset: Option<u32>) -> Result<u32> {
    self.write_int32_le(value, offset)
  }

  #[napi(js_name = "writeFloatBE")]
  pub fn write_float_be_alias(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    self.write_float_be(value, offset)
  }

  #[napi(js_name = "writeFloatLE")]
  pub fn write_float_le_alias(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    self.write_float_le(value, offset)
  }

  #[napi(js_name = "writeDoubleBE")]
  pub fn write_double_be_alias(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    self.write_double_be(value, offset)
  }

  #[napi(js_name = "writeDoubleLE")]
  pub fn write_double_le_alias(&mut self, value: f64, offset: Option<u32>) -> Result<u32> {
    self.write_double_le(value, offset)
  }

  #[napi(js_name = "writeBigUInt64BE")]
  pub fn write_big_u_int64_be_alias(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_be(value, offset)
  }

  #[napi(js_name = "writeBigUint64BE")]
  pub fn write_big_uint64_be(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_be(value, offset)
  }

  #[napi(js_name = "writeBigUInt64LE")]
  pub fn write_big_u_int64_le_alias(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_le(value, offset)
  }

  #[napi(js_name = "writeBigUint64LE")]
  pub fn write_big_uint64_le(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_u_int64_le(value, offset)
  }

  #[napi(js_name = "writeBigInt64BE")]
  pub fn write_big_int64_be_alias(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_int64_be(value, offset)
  }

  #[napi(js_name = "writeBigInt64LE")]
  pub fn write_big_int64_le_alias(&mut self, value: BigInt, offset: Option<u32>) -> Result<u32> {
    self.write_big_int64_le(value, offset)
  }

  #[napi(js_name = "writeUIntBE")]
  pub fn write_u_int_be_alias(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_be(value, offset, byte_length)
  }

  #[napi(js_name = "writeUintBE")]
  pub fn write_uint_be_alias(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_be(value, offset, byte_length)
  }

  #[napi(js_name = "writeUIntLE")]
  pub fn write_u_int_le_alias(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_le(value, offset, byte_length)
  }

  #[napi(js_name = "writeUintLE")]
  pub fn write_uint_le_alias(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_u_int_le(value, offset, byte_length)
  }

  #[napi(js_name = "writeIntBE")]
  pub fn write_int_be_alias(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_int_be(value, offset, byte_length)
  }

  #[napi(js_name = "writeIntLE")]
  pub fn write_int_le_alias(&mut self, value: f64, offset: u32, byte_length: u32) -> Result<u32> {
    self.write_int_le(value, offset, byte_length)
  }

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

  #[napi(js_name = "toJSON")]
  pub fn to_json(&self) -> BufferJson {
    BufferJson {
      r#type: "Buffer".to_string(),
      data: self.data.iter().map(|b| *b as u32).collect(),
    }
  }

  #[napi(getter)]
  pub fn buffer(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi(getter)]
  pub fn byte_offset(&self) -> u32 {
    0
  }

  #[napi(getter)]
  pub fn parent(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi]
  pub fn get_byte(&self, index: u32) -> Option<u32> {
    self.data.get(index as usize).map(|&b| b as u32)
  }

  #[napi]
  pub fn set_byte(&mut self, index: u32, value: u32) {
    if let Some(slot) = self.data.get_mut(index as usize) {
      *slot = (value & 0xff) as u8;
    }
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
    if s_start > s_end || s_end > self.data.len() {
      return Err(Error::new(Status::GenericFailure, "source range out of bounds"));
    }
    let to_copy = s_end - s_start;
    if t_start > target.data.len() {
      return Err(Error::new(Status::GenericFailure, "targetStart out of bounds"));
    }
    let max_copy = (target.data.len() - t_start).min(to_copy);
    target.data[t_start..t_start + max_copy].copy_from_slice(&self.data[s_start..s_start + max_copy]);
    Ok(max_copy as u32)
  }

  #[napi]
  pub fn slice(&self, start: Option<u32>, end: Option<u32>) -> Result<JsBuffer> {
    let (s, e) = self.range(start, end)?;
    Ok(JsBuffer::from(&self.data[s..e]))
  }

  #[napi]
  pub fn subarray(&self, start: Option<u32>, end: Option<u32>) -> Result<JsBuffer> {
    self.slice(start, end)
  }

  #[napi]
  pub fn bytes(&self) -> JsBuffer {
    JsBuffer::from(self.data.as_slice())
  }

  #[napi]
  pub fn fill(&mut self, env: Env, #[napi(ts_arg_type = "any")] value: Unknown, offset: Option<u32>, end: Option<u32>, encoding: Option<String>) -> Result<()> {
    let (s, e) = self.range(offset, end)?;
    if s >= e {
      return Ok(());
    }
    let env_raw = env.raw();
    let val_raw = value.raw();
    if value.get_type()? == ValueType::Number {
      let num: f64 = unsafe { f64::from_napi_value(env_raw, val_raw)? };
      let byte_val = (num as i32 & 0xff) as u8;
      self.data[s..e].iter_mut().for_each(|x| *x = byte_val);
    } else if value.get_type()? == ValueType::String {
      let str_val: String = unsafe { String::from_napi_value(env_raw, val_raw)? };
      let enc = encoding.unwrap_or_else(|| "utf8".to_string());
      let bytes = str_to_bytes(&str_val, &enc)?;
      if !bytes.is_empty() {
        for (i, slot) in self.data[s..e].iter_mut().enumerate() {
          *slot = bytes[i % bytes.len()];
        }
      }
    } else if value.is_buffer()? {
      let buf_val = unsafe { JsBuffer::from_napi_value(env_raw, val_raw)? };
      let bytes = buf_val.as_ref();
      if !bytes.is_empty() {
        for (i, slot) in self.data[s..e].iter_mut().enumerate() {
          *slot = bytes[i % bytes.len()];
        }
      }
    }
    Ok(())
  }

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

  #[napi]
  pub fn equals(&self, other: JsBuffer) -> bool {
    self.data == other.as_ref()
  }

  #[napi]
  pub fn compare(&self, other: JsBuffer) -> i32 {
    self.data.as_slice().cmp(other.as_ref()) as i32
  }

  #[napi]
  pub fn index_of(&self, env: Env, #[napi(ts_arg_type = "any")] value: Unknown, byte_offset: Option<u32>, encoding: Option<String>) -> Result<i32> {
    let o = byte_offset.unwrap_or(0) as usize;
    if o >= self.data.len() {
      return Ok(-1);
    }
    let env_raw = env.raw();
    let val_raw = value.raw();
    if value.get_type()? == ValueType::Number {
      let num: f64 = unsafe { f64::from_napi_value(env_raw, val_raw)? };
      let byte_val = (num as i32 & 0xff) as u8;
      match self.data[o..].iter().position(|b| *b == byte_val) {
        Some(p) => Ok((o + p) as i32),
        None => Ok(-1),
      }
    } else if value.get_type()? == ValueType::String {
      let str_val: String = unsafe { String::from_napi_value(env_raw, val_raw)? };
      let enc = encoding.unwrap_or_else(|| "utf8".to_string());
      let search_bytes = str_to_bytes(&str_val, &enc)?;
      if search_bytes.is_empty() {
        return Ok(o as i32);
      }
      if let Some(p) = self.data[o..].windows(search_bytes.len()).position(|w| w == search_bytes.as_slice()) {
        Ok((o + p) as i32)
      } else {
        Ok(-1)
      }
    } else if value.is_buffer()? {
      let buf_val = unsafe { JsBuffer::from_napi_value(env_raw, val_raw)? };
      let search_bytes = buf_val.as_ref();
      if search_bytes.is_empty() {
        return Ok(o as i32);
      }
      if let Some(p) = self.data[o..].windows(search_bytes.len()).position(|w| w == search_bytes) {
        Ok((o + p) as i32)
      } else {
        Ok(-1)
      }
    } else {
      Ok(-1)
    }
  }

  #[napi]
  pub fn last_index_of(&self, env: Env, #[napi(ts_arg_type = "any")] value: Unknown, byte_offset: Option<u32>, encoding: Option<String>) -> Result<i32> {
    let o = byte_offset.unwrap_or((self.data.len().saturating_sub(1)) as u32) as usize;
    let end_idx = (o + 1).min(self.data.len());
    let env_raw = env.raw();
    let val_raw = value.raw();
    if value.get_type()? == ValueType::Number {
      let num: f64 = unsafe { f64::from_napi_value(env_raw, val_raw)? };
      let byte_val = (num as i32 & 0xff) as u8;
      match self.data[..end_idx].iter().rposition(|b| *b == byte_val) {
        Some(p) => Ok(p as i32),
        None => Ok(-1),
      }
    } else if value.get_type()? == ValueType::String {
      let str_val: String = unsafe { String::from_napi_value(env_raw, val_raw)? };
      let enc = encoding.unwrap_or_else(|| "utf8".to_string());
      let search_bytes = str_to_bytes(&str_val, &enc)?;
      if search_bytes.is_empty() {
        return Ok(end_idx as i32);
      }
      if search_bytes.len() > end_idx {
        return Ok(-1);
      }
      for p in (0..=end_idx - search_bytes.len()).rev() {
        if &self.data[p..p + search_bytes.len()] == search_bytes.as_slice() {
          return Ok(p as i32);
        }
      }
      Ok(-1)
    } else if value.is_buffer()? {
      let buf_val = unsafe { JsBuffer::from_napi_value(env_raw, val_raw)? };
      let search_bytes = buf_val.as_ref();
      if search_bytes.is_empty() {
        return Ok(end_idx as i32);
      }
      if search_bytes.len() > end_idx {
        return Ok(-1);
      }
      for p in (0..=end_idx - search_bytes.len()).rev() {
        if &self.data[p..p + search_bytes.len()] == search_bytes {
          return Ok(p as i32);
        }
      }
      Ok(-1)
    } else {
      Ok(-1)
    }
  }

  #[napi]
  pub fn includes(&self, env: Env, #[napi(ts_arg_type = "any")] value: Unknown, byte_offset: Option<u32>, encoding: Option<String>) -> Result<bool> {
    Ok(self.index_of(env, value, byte_offset, encoding)? >= 0)
  }

  #[napi]
  pub fn entries(&self) -> Vec<Vec<u32>> {
    self.data.iter().enumerate().map(|(i, &b)| vec![i as u32, b as u32]).collect()
  }

  #[napi]
  pub fn keys(&self) -> Vec<u32> {
    (0..self.data.len() as u32).collect()
  }

  #[napi]
  pub fn values(&self) -> Vec<u32> {
    self.data.iter().map(|&b| b as u32).collect()
  }

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
  pub fn reverse(&mut self) {
    self.data.reverse();
  }

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

#[napi]
pub fn atob(data: String) -> Result<String> {
  let bytes = decode_base64(&data, false)?;
  Ok(bytes.iter().map(|b| *b as char).collect())
}

#[napi]
pub fn btoa(data: String) -> String {
  encode_base64(data.as_bytes(), false)
}

#[napi]
pub fn is_ascii(input: JsBuffer) -> bool {
  input.as_ref().iter().all(|b| *b < 128)
}

#[napi]
pub fn is_latin1(input: String) -> bool {
  input.chars().all(|c| (c as u32) <= 0xff)
}

#[napi]
pub fn is_utf8(input: JsBuffer) -> bool {
  std::str::from_utf8(input.as_ref()).is_ok()
}

#[napi(js_name = "resolveObjectURL")]
pub fn resolve_object_url(_id: String) -> Option<Blob> {
  None
}

#[napi]
pub fn transcode(source: JsBuffer, from_encoding: String, to_encoding: String) -> Result<JsBuffer> {
  let intermediate = bytes_to_str(source.as_ref(), &from_encoding)?;
  let out = str_to_bytes(&intermediate, &to_encoding)?;
  Ok(JsBuffer::from(out.as_slice()))
}

#[napi]
pub const INSPECT_MAX_BYTES: u32 = 50;

#[napi]
pub const kMaxLength: u32 = 0x7fff_ffff;

#[napi]
pub const kStringMaxLength: u32 = 0x1ffff_fe8;

#[napi]
pub fn constants() -> BufferConstants {
  BufferConstants {
    r#MAX_LENGTH: 0x7fff_ffff,
    r#MAX_STRING_LENGTH: 0x1ffff_fe8,
    inspect_max_bytes: 50,
    k_max_length: 0x7fff_ffff,
    k_string_max_length: 0x1ffff_fe8,
  }
}

#[napi(object)]
pub struct BufferConstants {
  pub r#MAX_LENGTH: u32,
  pub r#MAX_STRING_LENGTH: u32,
  pub inspect_max_bytes: u32,
  pub k_max_length: u32,
  pub k_string_max_length: u32,
}
