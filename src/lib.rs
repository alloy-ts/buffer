
use napi_derive::napi;

pub mod future;
pub mod task;
mod poll;
mod result;

pub use future::{FutureObj, LocalFutureObj};
pub use task::{LocalSpawn, Spawn, SpawnError, Task};

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}
