//! NAPI bindings mirroring [`futures_task`].
//!
//! The `Spawn` / `LocalSpawn` traits (the interface an executor implements so
//! that futures can be pushed onto it) and the [`SpawnError`] type returned when
//! spawning fails live here. Upstream these live in `futures_task::spawn`; they
//! are grouped under this `task` module as the task-scheduling surface.
//!
//! Provides the [`Spawn`] / [`LocalSpawn`] traits (the interface an executor
//! implements so that futures can be pushed onto it) and the [`SpawnError`]
//! type returned when spawning fails.
//!
//! These are plain Rust traits — a `Spawn` impl is not itself a `#[napi]`
//! class — but [`SpawnError`] is exposed to JavaScript as a class so it can be
//! thrown / inspected from JS and returned out of `#[napi]` functions.

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use napi_derive::napi;

use crate::{FutureObj, LocalFutureObj};

/// The `Spawn` trait allows for pushing futures onto an executor that will
/// run them to completion.
pub trait Spawn {
  /// Spawns a future that will be run to completion.
  ///
  /// # Errors
  ///
  /// The executor may be unable to spawn tasks. Spawn errors should
  /// represent relatively rare scenarios, such as the executor
  /// having been shut down so that it is no longer able to accept
  /// tasks.
  fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError>;

  /// Determines whether the executor is able to spawn new tasks.
  ///
  /// This method will return `Ok` when the executor is *likely*
  /// (but not guaranteed) to accept a subsequent spawn attempt.
  /// Likewise, an `Err` return means that `spawn` is likely, but
  /// not guaranteed, to yield an error.
  #[inline]
  fn status(&self) -> Result<(), SpawnError> {
    Ok(())
  }
}

/// The `LocalSpawn` is similar to [`Spawn`], but allows spawning futures
/// that don't implement `Send`.
pub trait LocalSpawn {
  /// Spawns a future that will be run to completion.
  ///
  /// # Errors
  ///
  /// The executor may be unable to spawn tasks. Spawn errors should
  /// represent relatively rare scenarios, such as the executor
  /// having been shut down so that it is no longer able to accept
  /// tasks.
  fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError>;

  /// Determines whether the executor is able to spawn new tasks.
  ///
  /// This method will return `Ok` when the executor is *likely*
  /// (but not guaranteed) to accept a subsequent spawn attempt.
  /// Likewise, an `Err` return means that `spawn` is likely, but
  /// not guaranteed, to yield an error.
  #[inline]
  fn status_local(&self) -> Result<(), SpawnError> {
    Ok(())
  }
}

/// An error that occurred during spawning.
#[napi]
pub struct SpawnError {
  _priv: (),
}

#[napi]
impl SpawnError {
  /// Spawning failed because the executor has been shut down.
  #[napi(factory)]
  pub fn shutdown() -> Self {
    Self { _priv: () }
  }

  /// Check whether spawning failed because the executor was shut down.
  #[napi]
  pub fn is_shutdown(&self) -> bool {
    true
  }
}

impl fmt::Debug for SpawnError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_tuple("SpawnError").field(&"shutdown").finish()
  }
}

impl fmt::Display for SpawnError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Executor is shutdown")
  }
}

impl std::error::Error for SpawnError {}

// Allows `SpawnError` to be returned from `#[napi]` functions as
// `Result<T, SpawnError>` (the generated wrapper converts via `.into()`).
impl From<SpawnError> for napi::Error {
  fn from(_: SpawnError) -> napi::Error {
    napi::Error::from_reason("Executor is shutdown".to_string())
  }
}

impl<Sp: ?Sized + Spawn> Spawn for &Sp {
  fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
    Sp::spawn_obj(self, future)
  }

  fn status(&self) -> Result<(), SpawnError> {
    Sp::status(self)
  }
}

impl<Sp: ?Sized + Spawn> Spawn for &mut Sp {
  fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
    Sp::spawn_obj(self, future)
  }

  fn status(&self) -> Result<(), SpawnError> {
    Sp::status(self)
  }
}

impl<Sp: ?Sized + LocalSpawn> LocalSpawn for &Sp {
  fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
    Sp::spawn_local_obj(self, future)
  }

  fn status_local(&self) -> Result<(), SpawnError> {
    Sp::status_local(self)
  }
}

impl<Sp: ?Sized + LocalSpawn> LocalSpawn for &mut Sp {
  fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
    Sp::spawn_local_obj(self, future)
  }

  fn status_local(&self) -> Result<(), SpawnError> {
    Sp::status_local(self)
  }
}

// Blanket `Spawn` / `LocalSpawn` impls for the standard smart pointers, mirroring
// the upstream `if_alloc` module (always available here since this crate targets
// `std`).
mod if_alloc {
  use super::*;

  impl<Sp: ?Sized + Spawn> Spawn for Box<Sp> {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
      (**self).spawn_obj(future)
    }

    fn status(&self) -> Result<(), SpawnError> {
      (**self).status()
    }
  }

  impl<Sp: ?Sized + LocalSpawn> LocalSpawn for Box<Sp> {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
      (**self).spawn_local_obj(future)
    }

    fn status_local(&self) -> Result<(), SpawnError> {
      (**self).status_local()
    }
  }

  impl<Sp: ?Sized + Spawn> Spawn for Rc<Sp> {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
      (**self).spawn_obj(future)
    }

    fn status(&self) -> Result<(), SpawnError> {
      (**self).status()
    }
  }

  impl<Sp: ?Sized + LocalSpawn> LocalSpawn for Rc<Sp> {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
      (**self).spawn_local_obj(future)
    }

    fn status_local(&self) -> Result<(), SpawnError> {
      (**self).status_local()
    }
  }

  impl<Sp: ?Sized + Spawn> Spawn for Arc<Sp> {
    fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
      (**self).spawn_obj(future)
    }

    fn status(&self) -> Result<(), SpawnError> {
      (**self).status()
    }
  }

  impl<Sp: ?Sized + LocalSpawn> LocalSpawn for Arc<Sp> {
    fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
      (**self).spawn_local_obj(future)
    }

    fn status_local(&self) -> Result<(), SpawnError> {
      (**self).status_local()
    }
  }
}

// ===========================================================================
// `Task` — NAPI-exposed, futures-task based executor.
//
// `Task::spawn` (and its aliases `exec` / `start`) drive a
// [`crate::future::Future`] to completion on the calling (JS) thread. napi
// futures are bound to the JS thread and are not `Send`, so they are always run
// on a single-threaded `futures_executor::LocalPool`.
//
// A `Task` may additionally own a multi-threaded `futures_executor::ThreadPool`
// (via [`Task::new_threaded`]). That pool can run `Send + 'static` *Rust*
// futures in parallel; `Task` implements the [`Spawn`] / [`LocalSpawn`] traits
// so Rust code can push such futures onto it. JS-backed futures still go through
// the local pool.
// ===========================================================================

use futures_executor::{LocalPool, LocalSpawner, ThreadPool};
use futures_task::{FutureObj as RealFutureObj, LocalFutureObj as RealLocalFutureObj, LocalSpawn as _, Spawn as _};

use crate::future::Future;

/// A futures-task based task executor.
///
/// Always owns a single-threaded `LocalPool` for JS-backed [`Future`]s, and may
/// optionally own a multi-threaded `ThreadPool` for `Send` Rust futures.
#[napi]
pub struct Task {
  local: LocalPool,
  local_spawner: LocalSpawner,
  threaded: Option<ThreadPool>,
}

#[napi]
impl Task {
  /// Create a task executor backed only by a single-threaded `LocalPool`.
  ///
  /// Use this for JS-backed futures (`Future` instances created from JS), which
  /// are bound to the JS thread and are not `Send`.
  #[napi(constructor)]
  pub fn new() -> Self {
    let local = LocalPool::new();
    let local_spawner = local.spawner();
    Task { local, local_spawner, threaded: None }
  }

  /// Create a task executor that also owns a multi-threaded `ThreadPool`.
  ///
  /// The `ThreadPool` can run `Send + 'static` Rust futures in parallel (via the
  /// [`Spawn`] trait). JS-backed futures are still driven on the local pool.
  #[napi]
  pub fn new_threaded() -> napi::Result<Self> {
    let local = LocalPool::new();
    let local_spawner = local.spawner();
    let threaded = ThreadPool::new()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Task { local, local_spawner, threaded: Some(threaded) })
  }

  /// Spawn (and run to completion) a [`Future`] on this executor.
  ///
  /// Because napi futures are bound to the JS thread and are not `Send`, this
  /// drives the future to completion on the *calling* thread (via
  /// `LocalPool::run_until`) rather than moving it to a background worker. The
  /// future's resolved JS value is discarded; if you need the result, await or
  /// poll the future directly.
  #[napi]
  pub fn spawn(&mut self, future: &mut Future) -> napi::Result<()> {
    self.local.run_until(&mut *future);
    Ok(())
  }

  /// Alias for [`Task::spawn`].
  #[napi]
  pub fn exec(&mut self, future: &mut Future) -> napi::Result<()> {
    self.spawn(future)
  }

  /// Alias for [`Task::spawn`].
  #[napi]
  pub fn start(&mut self, future: &mut Future) -> napi::Result<()> {
    self.spawn(future)
  }
}

// ---------------------------------------------------------------------------
// `Task` as a `Spawn` / `LocalSpawn` executor (Rust side).
//
// Our ported `FutureObj` / `LocalFutureObj` are a faithful, layout-identical
// port of `futures_task`'s types, so we can transmute them into the real types
// and hand them to `futures_executor`'s `ThreadPool` / `LocalPool`. The
// compile-time size/align assertions below guard that invariant.
// ---------------------------------------------------------------------------

const _: () = {
  assert!(
    core::mem::size_of::<FutureObj<'static, ()>>() == core::mem::size_of::<RealFutureObj<'static, ()>>()
  );
  assert!(
    core::mem::align_of::<FutureObj<'static, ()>>() == core::mem::align_of::<RealFutureObj<'static, ()>>()
  );
  assert!(
    core::mem::size_of::<LocalFutureObj<'static, ()>>()
      == core::mem::size_of::<RealLocalFutureObj<'static, ()>>()
  );
  assert!(
    core::mem::align_of::<LocalFutureObj<'static, ()>>()
      == core::mem::align_of::<RealLocalFutureObj<'static, ()>>()
  );
};

impl Spawn for Task {
  fn spawn_obj(&self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
    match &self.threaded {
      Some(pool) => {
        // SAFETY: `FutureObj` is a layout-identical port of
        // `futures_task::FutureObj` (verified by the assertions above); both
        // carry the same `dyn Future` vtable pointer + drop function.
        let real: RealFutureObj<'static, ()> = unsafe { core::mem::transmute(future) };
        pool.spawn_obj(real).map_err(|_| SpawnError::shutdown())
      }
      None => Err(SpawnError::shutdown()),
    }
  }
}

impl LocalSpawn for Task {
  fn spawn_local_obj(&self, future: LocalFutureObj<'static, ()>) -> Result<(), SpawnError> {
    // SAFETY: see the `Spawn` impl above; `LocalFutureObj` is layout-identical
    // to `futures_task::LocalFutureObj`.
    let real: RealLocalFutureObj<'static, ()> = unsafe { core::mem::transmute(future) };
    self.local_spawner.spawn_local_obj(real).map_err(|_| SpawnError::shutdown())
  }
}
