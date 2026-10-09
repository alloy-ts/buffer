pub mod bytes;
pub mod buffer;
pub mod reader;

pub use buffer::*;
pub use bytes::*;
pub use reader::*;

use napi_derive::napi;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
