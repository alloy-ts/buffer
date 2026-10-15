//! NAPI bindings mirroring [`std::future::Future`].
//!
//! The Rust `Future` trait (`poll(self: Pin<&mut Self>, cx: &mut Context) ->
//! Poll<Output>`) cannot be exposed directly to JavaScript. Instead this module
//! provides:
//!
//! * A `Future` napi *class* that wraps either:
//!   - a real Rust future (e.g. built by [`Future::ready`] / [`Future::pending`]),
//!     or
//!   - a JS object that *implements the `Future` interface* — i.e. has a
//!     `poll(cx: Context) -> Poll` method (hand it to `new Future(obj)`), or
//!   - a bare JS `(cx: Context) -> Poll` callback (via [`Future::poll_fn`],
//!     mirroring `std::future::poll_fn`).
//! * A `Waker` and a `Context` napi class so the `poll` signature
//!   (`poll(cx: Context) -> Poll`) is representable in the generated `.d.ts`.
//!
//! The output value is kept as the catch-all `Unknown` type (napi-derive 3.6.9
//! cannot express generic `#[napi]` types), consistent with `crate::poll::Poll`.
//!
//! The `Context` created by `Context::new()` carries a **no-op waker**: wakeups
//! are ignored. A real executor should construct the `Context` (and therefore
//! the `Waker`) from its own task waker so that `cx.waker().wake()` actually
//! schedules re-polling.

use std::fmt;
use std::future::Future as StdFuture;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::result::Result as StdResult;
use std::task::{Context as StdContext, Poll as StdPoll};

use napi::bindgen_prelude::{Function, JsObjectValue, Object, Result, Unknown};
use napi::JsValue;
use napi_derive::napi;

use crate::poll::{Context, Poll, Waker as NapiWaker};

/// Either a Rust future, a JS object implementing the `Future` interface, or a
/// bare JS `(cx) -> Poll` callback.
enum Inner<'a> {
  /// A JS object with a `poll(cx: Context) -> Poll` method.
  Js(Object<'a>),
  /// A bare JS callback `fn(cx: Context) -> Poll`.
  Fn(Function<'a, (Context,), Unknown<'a>>),
  /// A real Rust future whose output is a JS value (`Unknown`).
  Rust(LocalFutureObj<'a, Unknown<'a>>),
}

/// Convert a JS `Poll` value (as returned by a JS `poll`) into our `Poll`.
///
/// `Poll` does not implement `FromNapiValue` for the lifetime-parameterized
/// struct, so we read its `isReady` / `value` properties and rebuild via the
/// public `Poll::new` constructor.
fn poll_result_to_poll<'a>(result: Unknown<'a>) -> Result<Poll<'a>> {
  let obj = result.coerce_to_object()?;
  let is_ready: bool = obj.get_named_property("isReady")?;
  let value = if is_ready {
    Some(obj.get_named_property::<Unknown>("value")?)
  } else {
    None
  };
  Ok(Poll::new(value))
}

/// Mirrors [`std::future::Future`]. Exposed to JS as a class.
///
/// The TS-facing surface is: a `Future` is an object with a
/// `poll(cx: Context): Poll` method. To supply one from JS, pass an object that
/// implements that method to [`Future::new`].
#[napi]
pub struct Future<'a> {
  inner: Inner<'a>,
}

#[napi]
impl<'a> Future<'a> {
  /// Wrap a JS object that implements the `Future` interface (i.e. has a
  /// `poll(cx: Context) -> Poll` method) as a `Future`.
  #[napi(constructor)]
  pub fn new(js_future: Object<'a>) -> Self {
    Future {
      inner: Inner::Js(js_future),
    }
  }

  /// Create a `Future` that is immediately `Ready` with `value`. Mirrors
  /// [`std::future::ready`].
  #[napi(factory)]
  pub fn ready(value: Unknown<'a>) -> Self {
    Future {
      inner: Inner::Rust(LocalFutureObj::from(Box::pin(std::future::ready(value)))),
    }
  }

  /// Create a `Future` that is forever `Pending`.
  #[napi(factory)]
  pub fn pending() -> Future<'a> {
    Future {
      inner: Inner::Rust(LocalFutureObj::from(Box::pin(std::future::pending::<Unknown<'a>>()))),
    }
  }

  /// Wrap a JS `(cx: Context) -> Poll` callback as a `Future`. Mirrors
  /// [`std::future::poll_fn`].
  #[napi(factory)]
  pub fn poll_fn(f: Function<'a, (Context,), Unknown<'a>>) -> Self {
    Future {
      inner: Inner::Fn(f),
    }
  }

  /// Attempt to resolve the future, registering the waker from `cx` for later
  /// wakeup if the value is not yet available. Mirrors
  /// [`std::future::Future::poll`].
  ///
  /// * For a Rust future, this delegates to its `poll`.
  /// * For a JS future, this calls the object's `poll(cx)` method and returns
  ///   the resulting `Poll`.
  #[napi]
  pub fn poll(&mut self, cx: &mut Context) -> Result<Poll<'a>> {
    match &mut self.inner {
      Inner::Rust(f) => {
        let mut std_cx = cx.std_context();
        let p = Pin::new(f).poll(&mut std_cx);
        Ok(Poll::from(p))
      }
      Inner::Js(obj) => {
        // Hand the JS `poll` an owned `Context` whose waker is a clone of the
        // real one, so `cx.waker().wake()` still targets the same task.
        let js_cx = Context::from_waker(cx.waker.clone());
        // Receive the result as `Unknown` (a `Poll` instance) and rebuild our
        // `Poll` via its public constructor, since `Poll` does not implement
        // `FromNapiValue` for the lifetime-parameterized struct.
        let func: Function<'a, (Context,), Unknown<'a>> = obj.get_named_property("poll")?;
        let result: Unknown<'a> = func.apply(obj.clone(), (js_cx,))?;
        poll_result_to_poll(result)
      }
      Inner::Fn(f) => {
        let js_cx = Context::from_waker(cx.waker.clone());
        let result: Unknown<'a> = f.call((js_cx,))?;
        poll_result_to_poll(result)
      }
    }
  }
}

// ===========================================================================
// `LocalFutureObj` / `FutureObj` / `UnsafeFutureObj` — faithful port of
// `futures::future` (originally `futures_task` / `futures_core`).
//
// These are type-erased futures (roughly `Box<dyn Future<Output = T> + 'a>` and
// the `Send` variant) built on a raw-pointer + `UnsafeFutureObj` vtable rather
// than `Box<dyn Trait>`, matching the upstream API surface exactly. They are
// plain Rust types, not `#[napi]` classes: a `Pin<Box<dyn Future>>` has no
// meaningful JS representation, and they are what the `Future` class's
// `Inner::Rust` variant above is built on.
// ===========================================================================

/// A custom trait object for polling futures, roughly akin to
/// `Box<dyn Future<Output = T> + 'a>`.
///
/// This custom trait object was introduced as currently it is not possible to
/// take `dyn Trait` by value and `Box<dyn Trait>` is not available in no_std
/// contexts.
pub struct LocalFutureObj<'a, T> {
  future: *mut (dyn StdFuture<Output = T> + 'static),
  drop_fn: unsafe fn(*mut (dyn StdFuture<Output = T> + 'static)),
  _marker: PhantomData<&'a ()>,
}

// As LocalFutureObj only holds pointers, even if we move it, the pointed to
// values won't move, so this is safe as long as we don't provide any way for a
// user to directly access the pointers and move their values.
impl<T> Unpin for LocalFutureObj<'_, T> {}

#[allow(single_use_lifetimes)]
unsafe fn remove_future_lifetime<'a, T>(
  ptr: *mut (dyn StdFuture<Output = T> + 'a),
) -> *mut (dyn StdFuture<Output = T> + 'static) {
  unsafe { mem::transmute(ptr) }
}

#[allow(single_use_lifetimes)]
unsafe fn remove_drop_lifetime<'a, T>(
  ptr: unsafe fn(*mut (dyn StdFuture<Output = T> + 'a)),
) -> unsafe fn(*mut (dyn StdFuture<Output = T> + 'static)) {
  unsafe { mem::transmute(ptr) }
}

impl<'a, T> LocalFutureObj<'a, T> {
  /// Create a `LocalFutureObj` from a custom trait object representation.
  #[inline]
  pub fn new<F: UnsafeFutureObj<'a, T> + 'a>(f: F) -> Self {
    Self {
      future: unsafe { remove_future_lifetime(f.into_raw()) },
      drop_fn: unsafe { remove_drop_lifetime(F::drop) },
      _marker: PhantomData,
    }
  }

  /// Converts the `LocalFutureObj` into a `FutureObj`.
  ///
  /// # Safety
  ///
  /// To make this operation safe one has to ensure that the `UnsafeFutureObj`
  /// instance from which this `LocalFutureObj` was created actually
  /// implements `Send`.
  #[inline]
  pub unsafe fn into_future_obj(self) -> FutureObj<'a, T> {
    FutureObj(self)
  }
}

impl<T> fmt::Debug for LocalFutureObj<'_, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("LocalFutureObj").finish()
  }
}

impl<'a, T> From<FutureObj<'a, T>> for LocalFutureObj<'a, T> {
  #[inline]
  fn from(f: FutureObj<'a, T>) -> Self {
    f.0
  }
}

impl<T> StdFuture for LocalFutureObj<'_, T> {
  type Output = T;

  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut StdContext<'_>) -> StdPoll<T> {
    unsafe { Pin::new_unchecked(&mut *self.future).poll(cx) }
  }
}

impl<T> Drop for LocalFutureObj<'_, T> {
  fn drop(&mut self) {
    unsafe { (self.drop_fn)(self.future) }
  }
}

/// A custom trait object for polling futures, roughly akin to
/// `Box<dyn Future<Output = T> + Send + 'a>`.
///
/// This custom trait object was introduced as currently it is not possible to
/// take `dyn Trait` by value and `Box<dyn Trait>` is not available in no_std
/// contexts.
///
/// You should generally not need to use this type outside of `no_std` or when
/// implementing `Spawn`, consider using `BoxFuture` instead.
pub struct FutureObj<'a, T>(LocalFutureObj<'a, T>);

impl<T> Unpin for FutureObj<'_, T> {}
// SAFETY: `FutureObj::new` and `into_future_obj` only ever wrap `Send` futures,
// so claiming `Send` here is sound.
unsafe impl<T> Send for FutureObj<'_, T> {}

impl<'a, T> FutureObj<'a, T> {
  /// Create a `FutureObj` from a custom trait object representation.
  #[inline]
  pub fn new<F: UnsafeFutureObj<'a, T> + Send>(f: F) -> Self {
    Self(LocalFutureObj::new(f))
  }
}

impl<T> fmt::Debug for FutureObj<'_, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("FutureObj").finish()
  }
}

impl<T> StdFuture for FutureObj<'_, T> {
  type Output = T;

  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut StdContext<'_>) -> StdPoll<T> {
    Pin::new(&mut self.0).poll(cx)
  }
}

/// A custom implementation of a future trait object for `FutureObj`, providing
/// a vtable with drop support.
///
/// This custom representation is typically used only in `no_std` contexts,
/// where the default `Box`-based implementation is not available.
///
/// # Safety
///
/// See the safety notes on individual methods for what guarantees an
/// implementor must provide.
pub unsafe trait UnsafeFutureObj<'a, T>: 'a {
  /// Convert an owned instance into a (conceptually owned) fat pointer.
  ///
  /// # Safety
  ///
  /// ## Implementor
  ///
  /// The trait implementor must guarantee that it is safe to convert the
  /// provided `*mut (dyn Future<Output = T> + 'a)` into a `Pin<&mut (dyn
  /// Future<Output = T> + 'a)>` and call methods on it, non-reentrantly,
  /// until `UnsafeFutureObj::drop` is called with it.
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a);

  /// Drops the future represented by the given fat pointer.
  ///
  /// # Safety
  ///
  /// ## Implementor
  ///
  /// The trait implementor must guarantee that it is safe to call this
  /// function once per `into_raw` invocation.
  ///
  /// ## Caller
  ///
  /// The caller must ensure:
  ///
  ///  * the pointer passed was obtained from an `into_raw` invocation from
  ///    this same trait object
  ///  * the pointer is not currently in use as a `Pin<&mut (dyn Future<Output
  ///    = T> + 'a)>`
  ///  * the pointer must not be used again after this function is called
  unsafe fn drop(ptr: *mut (dyn StdFuture<Output = T> + 'a));
}

unsafe impl<'a, T, F> UnsafeFutureObj<'a, T> for &'a mut F
where
  F: StdFuture<Output = T> + Unpin + 'a,
{
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    self as *mut dyn StdFuture<Output = T>
  }

  unsafe fn drop(_ptr: *mut (dyn StdFuture<Output = T> + 'a)) {}
}

unsafe impl<'a, T> UnsafeFutureObj<'a, T> for &'a mut (dyn StdFuture<Output = T> + Unpin + 'a) {
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    self as *mut dyn StdFuture<Output = T>
  }

  unsafe fn drop(_ptr: *mut (dyn StdFuture<Output = T> + 'a)) {}
}

unsafe impl<'a, T, F> UnsafeFutureObj<'a, T> for Pin<&'a mut F>
where
  F: StdFuture<Output = T> + 'a,
{
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    unsafe { self.get_unchecked_mut() as *mut dyn StdFuture<Output = T> }
  }

  unsafe fn drop(_ptr: *mut (dyn StdFuture<Output = T> + 'a)) {}
}

unsafe impl<'a, T> UnsafeFutureObj<'a, T> for Pin<&'a mut (dyn StdFuture<Output = T> + 'a)> {
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    unsafe { self.get_unchecked_mut() as *mut dyn StdFuture<Output = T> }
  }

  unsafe fn drop(_ptr: *mut (dyn StdFuture<Output = T> + 'a)) {}
}

// `Box` / `Pin<Box>` `UnsafeFutureObj` impls (the `if_alloc` module upstream;
// always available here since this crate targets `std`).
unsafe impl<'a, T, F> UnsafeFutureObj<'a, T> for Box<F>
where
  F: StdFuture<Output = T> + 'a,
{
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    Box::into_raw(self)
  }

  unsafe fn drop(ptr: *mut (dyn StdFuture<Output = T> + 'a)) {
    drop(unsafe { Box::from_raw(ptr.cast::<F>()) })
  }
}

unsafe impl<'a, T: 'a> UnsafeFutureObj<'a, T> for Box<dyn StdFuture<Output = T> + 'a> {
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    Box::into_raw(self)
  }

  unsafe fn drop(ptr: *mut (dyn StdFuture<Output = T> + 'a)) {
    drop(unsafe { Box::from_raw(ptr) })
  }
}

unsafe impl<'a, T: 'a> UnsafeFutureObj<'a, T> for Box<dyn StdFuture<Output = T> + Send + 'a> {
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    Box::into_raw(self)
  }

  unsafe fn drop(ptr: *mut (dyn StdFuture<Output = T> + 'a)) {
    drop(unsafe { Box::from_raw(ptr) })
  }
}

unsafe impl<'a, T, F> UnsafeFutureObj<'a, T> for Pin<Box<F>>
where
  F: StdFuture<Output = T> + 'a,
{
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    let mut this = mem::ManuallyDrop::new(self);
    unsafe { this.as_mut().get_unchecked_mut() as *mut _ }
  }

  unsafe fn drop(ptr: *mut (dyn StdFuture<Output = T> + 'a)) {
    drop(Pin::from(unsafe { Box::from_raw(ptr) }))
  }
}

unsafe impl<'a, T: 'a> UnsafeFutureObj<'a, T> for Pin<Box<dyn StdFuture<Output = T> + 'a>> {
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    let mut this = mem::ManuallyDrop::new(self);
    unsafe { this.as_mut().get_unchecked_mut() as *mut _ }
  }

  unsafe fn drop(ptr: *mut (dyn StdFuture<Output = T> + 'a)) {
    drop(Pin::from(unsafe { Box::from_raw(ptr) }))
  }
}

unsafe impl<'a, T: 'a> UnsafeFutureObj<'a, T> for Pin<Box<dyn StdFuture<Output = T> + Send + 'a>> {
  fn into_raw(self) -> *mut (dyn StdFuture<Output = T> + 'a) {
    let mut this = mem::ManuallyDrop::new(self);
    unsafe { this.as_mut().get_unchecked_mut() as *mut _ }
  }

  unsafe fn drop(ptr: *mut (dyn StdFuture<Output = T> + 'a)) {
    drop(Pin::from(unsafe { Box::from_raw(ptr) }))
  }
}

// Generic `From<Box<F>>` / `From<Pin<Box<F>>>` (upstream only provides these
// for `Output = ()`; we need generic `T` for the napi `Future` integration).
impl<'a, T, F: StdFuture<Output = T> + 'a> From<Box<F>> for LocalFutureObj<'a, T> {
  fn from(boxed: Box<F>) -> Self {
    LocalFutureObj::new(boxed)
  }
}

impl<'a, T, F: StdFuture<Output = T> + 'a> From<Pin<Box<F>>> for LocalFutureObj<'a, T> {
  fn from(boxed: Pin<Box<F>>) -> Self {
    LocalFutureObj::new(boxed)
  }
}

impl<'a, T, F: StdFuture<Output = T> + Send + 'a> From<Box<F>> for FutureObj<'a, T> {
  fn from(boxed: Box<F>) -> Self {
    FutureObj::new(boxed)
  }
}

impl<'a, T, F: StdFuture<Output = T> + Send + 'a> From<Pin<Box<F>>> for FutureObj<'a, T> {
  fn from(boxed: Pin<Box<F>>) -> Self {
    FutureObj::new(boxed)
  }
}

// ===========================================================================
// `std::future::Future` for the napi `Future` class.
//
// Implementing the real `Future` trait makes the napi `Future` awaitable from
// Rust, and therefore also an `IntoFuture` via the std blanket
// `impl<F: Future> IntoFuture for F`. Polling delegates to the class's own
// `poll` method, bridging the host executor's waker into the JS-facing
// `Context` so `cx.waker().wake()` from JS targets the real task rather than
// the no-op waker.
// ===========================================================================

impl<'a> StdFuture for Future<'a> {
  type Output = Unknown<'a>;

  fn poll(self: Pin<&mut Self>, cx: &mut StdContext<'_>) -> StdPoll<Self::Output> {
    // SAFETY: `Future` holds only `Unpin` fields (`Object`, `Function`,
    // `LocalFutureObj`), so moving out of the pinned projection is sound.
    let this = unsafe { self.get_unchecked_mut() };
    let mut js_cx = Context::from_waker(NapiWaker::from_std(cx.waker().clone()));
    match this.poll(&mut js_cx) {
      Ok(p) => match p.value() {
        Some(v) => StdPoll::Ready(v),
        None => StdPoll::Pending,
      },
      // A JS `poll` that throws can't be surfaced through `Future::poll`; treat
      // it as still pending rather than panicking across the napi boundary.
      Err(_) => StdPoll::Pending,
    }
  }
}

// ===========================================================================
// `BoxFuture` / `LocalBoxFuture` — type aliases mirroring `futures::future`.
// ===========================================================================

/// A `Pin<Box<dyn Future + Send + 'a>>` — a type-erased, `Send` future.
///
/// Mirrors `futures::future::BoxFuture`.
pub type BoxFuture<'a, T> = Pin<Box<dyn StdFuture<Output = T> + Send + 'a>>;

/// A `Pin<Box<dyn Future + 'a>>` — a type-erased, non-`Send` future.
///
/// Mirrors `futures::future::LocalBoxFuture`.
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn StdFuture<Output = T> + 'a>>;

// ===========================================================================
// `FusedFuture` / `TryFuture` — mirror `futures_core::future`.
// ===========================================================================

/// A future which tracks whether it should no longer be polled.
///
/// `is_terminated` returns `true` once the future should be ignored or dropped
/// rather than polled again. Mirrors `futures_core::future::FusedFuture`.
pub trait FusedFuture: StdFuture {
  /// Returns `true` if the underlying future should no longer be polled.
  fn is_terminated(&self) -> bool;
}

impl<F: FusedFuture + ?Sized + Unpin> FusedFuture for &mut F {
  fn is_terminated(&self) -> bool {
    <F as FusedFuture>::is_terminated(&**self)
  }
}

impl<P> FusedFuture for Pin<P>
where
  P: std::ops::DerefMut + Unpin,
  P::Target: FusedFuture,
{
  fn is_terminated(&self) -> bool {
    <P::Target as FusedFuture>::is_terminated(&**self)
  }
}

impl<F: FusedFuture + ?Sized + Unpin> FusedFuture for Box<F> {
  fn is_terminated(&self) -> bool {
    <F as FusedFuture>::is_terminated(&**self)
  }
}

mod private_try_future {
  use super::{StdFuture, StdResult};

  pub trait Sealed {}

  impl<F, T, E> Sealed for F where F: ?Sized + StdFuture<Output = StdResult<T, E>> {}
}

/// A convenience trait for futures whose output is a `Result`, with adapters
/// tailored to such futures.
///
/// Mirrors `futures_core::future::TryFuture`. The `Sealed` supertrait keeps it
/// crate-local so downstream crates can't implement it for arbitrary types.
pub trait TryFuture: StdFuture + private_try_future::Sealed {
  /// The type of successful values yielded by this future.
  type Ok;
  /// The type of failures yielded by this future.
  type Error;

  /// Poll this `TryFuture` as if it were a `Future`.
  fn try_poll(
    self: Pin<&mut Self>,
    cx: &mut StdContext<'_>,
  ) -> StdPoll<StdResult<Self::Ok, Self::Error>>;
}

impl<F, T, E> TryFuture for F
where
  F: ?Sized + StdFuture<Output = StdResult<T, E>>,
{
  type Ok = T;
  type Error = E;

  fn try_poll(
    self: Pin<&mut Self>,
    cx: &mut StdContext<'_>,
  ) -> StdPoll<StdResult<Self::Ok, Self::Error>> {
    // Fully-qualify to pick the `Future::poll` trait method and avoid ambiguity
    // with the inherent `poll` on the napi `Future` class (which returns a
    // `napi::Result`).
    StdFuture::poll(self, cx)
  }
}

// ===========================================================================
// Executors — re-exported from `futures_executor` (added via `cargo add
// futures-executor --features thread-pool`).
//
// Re-implementing `ThreadPool` / `LocalPool` / `block_on` would duplicate
// thousands of lines and internal dependencies (`FuturesUnordered`,
// `UnparkMutex`, the `enter` guard, …), so we surface the upstream types
// directly. They are pure-Rust executors (not `#[napi]` classes) and are meant
// for Rust-side use / driving Rust futures; to use them, bring the
// `futures_task::{Spawn, LocalSpawn, SpawnError, ArcWake}` traits into scope.
// ===========================================================================

pub use futures_executor::{
  block_on, block_on_stream, BlockingStream, LocalPool, LocalSpawner, ThreadPool, ThreadPoolBuilder,
};
