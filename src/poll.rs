//! NAPI bindings mirroring [`core::task`] — the task-system primitives
//! (`Poll`, `Waker`, `Context`) that back `Future::poll`.
//!
//! This module also port-mirrors two futures-rs helpers:
//!
//! * [`PollFn`] / [`poll_fn`] — a `Future` wrapping a JS `(cx: Context) ->
//!   Poll` callback, exposed as a napi *class* (a napi class cannot be generic
//!   over the closure type `F`, so it is specialized to a JS callback). It also
//!   implements the real `std::future::Future` (bridging the host executor's
//!   waker into JS via [`Waker::from_std`]).
//! * [`PollOnce`] / [`poll`] / the `poll!` macro — a faithful *generic* Rust
//!   port of `futures::future::{poll, PollOnce}`. These operate on real
//!   `std::future::Future`s (the macro must expand to an `.await`), so they are
//!   plain Rust, not `#[napi]` classes, and are not surfaced to JS.
//!
//! Notes on the napi-derive (3.6.9) constraints that shape this file:
//!
//! * A `string_enum` can only hold unit (data-free) variants, so the `Poll`
//!   status is exposed as a separate `PollStatus` (kebab-case `"ready"` /
//!   `"pending"`), while the value-carrying `Ready(T)` payload is modelled with
//!   a *struct* holding `Option<Unknown>` (`Some` = ready, `None` = pending).
//! * `#[napi]` enums can't be generic or carry lifetimes, so the value type is
//!   the catch-all `Unknown` (consistent with [`crate::future`] and
//!   [`crate::result`]).
//! * `Waker` / `Context` are exposed as classes so the `poll(cx: Context) ->
//!   Poll` signature is representable in the generated `.d.ts`.
//!
//! The lower-level `RawWaker` / `RawWakerVTable` / `LocalWaker` from the std
//! module are intentionally *not* surfaced: they are executor-internal and
//! carry raw function pointers that have no meaningful JS representation. They
//! are reimplemented internally (see [`noop_raw_waker`]) to build the no-op
//! waker handed out by `Context::new()`.
//!
//! To bridge a concrete `std::task::Poll<SomeT>`, match on it and wrap the
//! ready value with a napi-compatible conversion before calling `Poll::ready` /
//! the constructor.

use std::future::Future as StdFuture;
use std::pin::Pin;
use std::task::{
  Context as StdContext, Poll as StdPoll, RawWaker as StdRawWaker,
  RawWakerVTable as StdRawWakerVTable, Waker as StdWaker,
};

use napi::JsValue;
use napi::bindgen_prelude::{Env, Function, JsObjectValue, Result as NapiResult, Unknown};
use napi_derive::napi;

/// Status discriminator mirroring the two `std::task::Poll` variants.
///
/// Exposed as a kebab-case string enum (`"ready"` / `"pending"`).
#[napi(string_enum = "kebab-case")]
pub enum PollStatus {
  Ready,
  Pending,
}

/// Mirrors [`std::task::Poll`]: either a value is ready (`Some`) or the task is
/// pending (`None`).
#[napi]
pub struct Poll<'a> {
  value: Option<Unknown<'a>>,
}

#[napi]
impl<'a> Poll<'a> {
  /// Create a poll from an optional value (`Some` = ready, `None` = pending).
  #[napi(constructor)]
  pub fn new(value: Option<Unknown<'a>>) -> Self {
    Poll { value }
  }

  /// Build a `Ready` poll wrapping `value`.
  #[napi(factory)]
  pub fn ready(value: Unknown<'a>) -> Self {
    Poll { value: Some(value) }
  }

  /// Build a `Pending` poll.
  #[napi(factory)]
  pub fn pending() -> Poll<'a> {
    Poll { value: None }
  }

  /// The status discriminator (`"ready"` / `"pending"`).
  #[napi(getter)]
  pub fn status(&self) -> PollStatus {
    if self.value.is_some() {
      PollStatus::Ready
    } else {
      PollStatus::Pending
    }
  }

  /// `true` if this is `Ready`.
  #[napi]
  pub fn is_ready(&self) -> bool {
    self.value.is_some()
  }

  /// `true` if this is `Pending`.
  #[napi]
  pub fn is_pending(&self) -> bool {
    self.value.is_none()
  }

  /// Returns the contained value if `Ready`, otherwise `null`.
  #[napi]
  pub fn value(&self) -> Option<Unknown<'a>> {
    self.value.clone()
  }

  /// Maps the contained `Ready` value through a JS callback, leaving `Pending`
  /// untouched. The callback receives the ready value and must return the new
  /// value.
  #[napi]
  pub fn map(&self, _env: Env, f: Function<'a>) -> NapiResult<Poll<'a>> {
    match &self.value {
      Some(v) => {
        let out: Unknown = f.call(v.clone())?;
        Ok(Poll { value: Some(out) })
      }
      None => Ok(Poll { value: None }),
    }
  }

  /// Unwraps a `Ready` value, panicking on `Pending`. Mirrors the (currently
  /// unstable on `std`) `Poll::unwrap`.
  #[napi]
  pub fn unwrap(&self) -> Unknown<'a> {
    self
      .value
      .clone()
      .expect("called `Poll::unwrap()` on a `Pending` value")
  }
}

impl<'a> From<std::task::Poll<Unknown<'a>>> for Poll<'a> {
  fn from(p: std::task::Poll<Unknown<'a>>) -> Self {
    match p {
      std::task::Poll::Ready(v) => Poll { value: Some(v) },
      std::task::Poll::Pending => Poll { value: None },
    }
  }
}

/// Build a no-op [`StdRawWaker`] whose `wake` / `wake_by_ref` / `drop` do nothing.
fn noop_raw_waker() -> StdRawWaker {
  unsafe fn clone(_: *const ()) -> StdRawWaker {
    noop_raw_waker()
  }
  unsafe fn no_op(_: *const ()) {}
  static VTABLE: StdRawWakerVTable = StdRawWakerVTable::new(clone, no_op, no_op, no_op);
  StdRawWaker::new(std::ptr::null(), &VTABLE)
}

/// Build a no-op [`StdWaker`] whose `wake` / `wake_by_ref` / `drop` do nothing.
fn noop_waker() -> StdWaker {
  unsafe { StdWaker::from_raw(noop_raw_waker()) }
}

/// Mirrors [`std::task::Waker`].
#[napi]
pub struct Waker {
  pub(crate) inner: StdWaker,
}

#[napi]
impl Waker {
  /// Create a waker backed by a no-op (wakeups are ignored).
  #[napi(constructor)]
  pub fn new() -> Self {
    Waker {
      inner: noop_waker(),
    }
  }

  /// Wake the associated task. With the no-op waker this is a no-op.
  #[napi]
  pub fn wake(&self) {
    self.inner.wake_by_ref();
  }

  /// Clone the waker (cheap; shares the same underlying task).
  #[napi]
  pub fn clone(&self) -> Waker {
    Waker {
      inner: self.inner.clone(),
    }
  }

  /// Build a `Waker` from a std `task::Waker`. Used to bridge a real executor's
  /// waker into the JS-facing `Context` (see the `PollFn` / `PollOnce`
  /// `std::future::Future` impls), so that `cx.waker().wake()` from JS targets
  /// the host task rather than the no-op waker.
  pub(crate) fn from_std(waker: StdWaker) -> Self {
    Waker { inner: waker }
  }
}

/// Mirrors [`std::task::Context`]. Holds a [`Waker`] that JS futures may use to
/// request re-polling.
#[napi]
pub struct Context {
  pub(crate) waker: Waker,
}

#[napi]
impl Context {
  /// Create a `Context` with a no-op waker. A real executor should instead build
  /// the `Context` (and therefore the `Waker`) from its own task waker.
  #[napi(constructor)]
  pub fn new() -> Self {
    Context {
      waker: Waker::new(),
    }
  }

  /// Returns a clone of the waker associated with this context.
  #[napi]
  pub fn waker(&self) -> Waker {
    self.waker.clone()
  }

  /// Build a `Context` around an existing waker (used internally by the `Future`
  /// bindings when handing a JS future its own context).
  pub(crate) fn from_waker(waker: Waker) -> Self {
    Context { waker }
  }

  /// Build a std `Context` borrowing this context's waker for Rust-side polling.
  pub(crate) fn std_context(&self) -> StdContext<'_> {
    StdContext::from_waker(&self.waker.inner)
  }
}

/// Reconstruct a [`Poll`] from a JS value returned by a `poll(cx)` call.
///
/// `Poll` does not implement `FromNapiValue` for the lifetime-parameterized
/// struct, so we read its `isReady` / `value` properties and rebuild via the
/// public `Poll::new` constructor. Mirrors the same reconstruction done in
/// `crate::future::poll_result_to_poll`.
fn poll_from_unknown<'a>(result: Unknown<'a>) -> NapiResult<Poll<'a>> {
  let obj = result.coerce_to_object()?;
  let is_ready: bool = obj.get_named_property("isReady")?;
  let value = if is_ready {
    Some(obj.get_named_property::<Unknown>("value")?)
  } else {
    None
  };
  Ok(Poll::new(value))
}

// ===========================================================================
// `poll_fn` / `PollFn` — mirror `futures::future::{poll_fn, PollFn}`.
// ===========================================================================

/// Wrap a JS `(cx: Context) -> Poll` callback.
///
/// napi classes may not hold a `Function` field directly, so the callback is
/// stored in this opaque (non-`#[napi]`) newtype — the same trick
/// [`crate::future::Future`] uses for its JS callback variant.
struct JsCallback<'a>(Function<'a, (Context,), Unknown<'a>>);

/// Future for the [`poll_fn`] function.
///
/// Wraps a JS `(cx: Context) -> Poll` callback and exposes it as a `Future`
/// (i.e. an object with a `poll(cx: Context) -> Poll` method). Mirrors
/// `futures::future::PollFn`, specialized to a JS callback (a napi class cannot
/// be generic over the closure type `F`).
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[napi]
pub struct PollFn<'a> {
  f: JsCallback<'a>,
}

#[napi]
impl<'a> PollFn<'a> {
  /// Wrap a JS `(cx: Context) -> Poll` callback as a `PollFn`.
  #[napi(constructor)]
  pub fn new(f: Function<'a, (Context,), Unknown<'a>>) -> Self {
    PollFn { f: JsCallback(f) }
  }

  /// Attempt to resolve the wrapped callback, returning its `Poll` result.
  /// Mirrors `Future::poll` for `PollFn`.
  #[napi]
  pub fn poll(&mut self, cx: &mut Context) -> NapiResult<Poll<'a>> {
    let js_cx = Context::from_waker(cx.waker.clone());
    let result: Unknown<'a> = self.f.0.call((js_cx,))?;
    poll_from_unknown(result)
  }
}

/// Creates a new future wrapping around a JS callback returning [`Poll`].
///
/// Polling the returned future delegates to the wrapped callback. Mirrors
/// `futures::future::poll_fn`.
#[napi]
pub fn poll_fn<'a>(f: Function<'a, (Context,), Unknown<'a>>) -> PollFn<'a> {
  PollFn { f: JsCallback(f) }
}

impl<'a> StdFuture for PollFn<'a> {
  type Output = Unknown<'a>;

  fn poll(self: Pin<&mut Self>, cx: &mut StdContext<'_>) -> StdPoll<Self::Output> {
    // SAFETY: `PollFn` holds only `Unpin` fields (`JsCallback` wraps a
    // `Function`), so moving out of the pinned projection is sound.
    let this = unsafe { self.get_unchecked_mut() };
    let js_cx = Context::from_waker(Waker::from_std(cx.waker().clone()));
    let result = this
      .f
      .0
      .call((js_cx,))
      .expect("poll_fn: wrapped callback threw");
    let p = poll_from_unknown(result).expect("poll_fn: callback did not return a Poll");
    match p.value {
      Some(v) => StdPoll::Ready(v),
      None => StdPoll::Pending,
    }
  }
}

// ===========================================================================
// `poll!` / `poll` / `PollOnce` — faithful Rust port of
// `futures::future::{poll, PollOnce}` and the `poll!` macro.
//
// These operate on *real* `std::future::Future`s: the `poll!` macro must expand
// to an `.await`, which requires a real `Future`, so they are plain Rust
// generics rather than `#[napi]` classes and are therefore not surfaced to JS.
// (The JS-facing `Future` / `Poll` machinery lives in [`crate::future`] and
// [`crate::poll`.])
// ===========================================================================

/// A future which returns the result of polling a future once.
///
/// Mirrors `futures::future::PollOnce`: always `Ready` once polled, with the
/// contained value being the inner future's `Poll` (`Output = Poll<F::Output>`).
#[allow(missing_debug_implementations, dead_code)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct PollOnce<F: StdFuture + Unpin> {
  future: F,
}

impl<F: StdFuture + Unpin> StdFuture for PollOnce<F> {
  type Output = StdPoll<F::Output>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut StdContext<'_>) -> StdPoll<Self::Output> {
    StdPoll::Ready(Pin::new(&mut self.future).poll(cx))
  }
}

/// Returns a [`PollOnce`] future that resolves to the result of polling
/// `future` once. Mirrors `futures::future::poll`.
#[allow(dead_code)]
pub fn poll<F: StdFuture + Unpin>(future: F) -> PollOnce<F> {
  PollOnce { future }
}

/// A macro which returns the result of polling a future once within the
/// current `async` context. Mirrors `futures::future::poll!`.
///
/// Only usable inside `async` functions, closures, and blocks.
#[macro_export]
macro_rules! poll {
  ($x:expr $(,)?) => {
    $crate::poll::poll($x).await
  };
}
