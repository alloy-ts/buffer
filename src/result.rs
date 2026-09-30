//! NAPI bindings mirroring [`std::result::Result`].
//!
//! Like [`crate::poll::Poll`], napi-derive 3.6.9 cannot express a generic
//! `#[napi]` enum, and both `Ok(T)` and `Err(E)` carry a payload, so the value
//! type is the catch-all `Unknown`. A `Result` is therefore a JS class that
//! wraps either an `Ok` value or an `Err` value, each an arbitrary JS value.
//!
//! Construct one from JS with the `ok` / `err` factories (mirroring the `Ok` /
//! `Err` constructors) and drive it with the same method vocabulary as
//! `std::result::Result`.
//!
//! The Rust struct is named `JsResult` only because a local `Result` type would
//! shadow the `core::result::Result` that napi-derive emits into its generated
//! code; it is exposed to JS as `Result` via `#[napi(js_name = "Result")]`.

use napi::bindgen_prelude::{Function, Result as NapiResult, Unknown};
use napi::Error;
use napi_derive::napi;

/// Either an `Ok` value or an `Err` value, each an arbitrary JS value.
enum Inner<'a> {
  Ok(Unknown<'a>),
  Err(Unknown<'a>),
}

impl<'a> Clone for Inner<'a> {
  fn clone(&self) -> Self {
    match self {
      Inner::Ok(v) => Inner::Ok(v.clone()),
      Inner::Err(e) => Inner::Err(e.clone()),
    }
  }
}

/// Mirrors [`std::result::Result`]. Exposed to JS as `Result` (a class).
///
/// Construct with the `ok` / `err` factories, then consume with the `isOk` /
/// `isErr` / `map` / `unwrap` family of methods.
#[napi(js_name = "Result")]
pub struct JsResult<'a> {
  inner: Inner<'a>,
}

impl<'a> Clone for JsResult<'a> {
  fn clone(&self) -> Self {
    JsResult {
      inner: self.inner.clone(),
    }
  }
}

#[napi]
impl<'a> JsResult<'a> {
  /// Construct an `Ok` result wrapping `value`. Mirrors `Ok(value)`.
  #[napi(factory)]
  pub fn ok(value: Unknown<'a>) -> Self {
    JsResult { inner: Inner::Ok(value) }
  }

  /// Construct an `Err` result wrapping `value`. Mirrors `Err(value)`.
  #[napi(factory)]
  pub fn err(value: Unknown<'a>) -> Self {
    JsResult { inner: Inner::Err(value) }
  }

  /// Construct from JS: `new Result(value, ok)` where `ok = true` makes an `Ok`
  /// and `ok = false` makes an `Err`. Prefer the `ok` / `err` factories, which
  /// read more like the `Ok` / `Err` constructors.
  #[napi(constructor)]
  pub fn new(value: Unknown<'a>, ok: bool) -> Self {
    if ok {
      JsResult { inner: Inner::Ok(value) }
    } else {
      JsResult { inner: Inner::Err(value) }
    }
  }

  // ----- Querying the variant -----

  /// `true` if this is `Ok`.
  #[napi]
  pub fn is_ok(&self) -> bool {
    matches!(self.inner, Inner::Ok(_))
  }

  /// `true` if this is `Err`.
  #[napi]
  pub fn is_err(&self) -> bool {
    matches!(self.inner, Inner::Err(_))
  }

  /// `true` if `Ok` and the contained value satisfies the predicate.
  #[napi]
  pub fn is_ok_and(&self, f: Function<'a, Unknown<'a>, bool>) -> NapiResult<bool> {
    match &self.inner {
      Inner::Ok(v) => f.call(v.clone()),
      Inner::Err(_) => Ok(false),
    }
  }

  /// `true` if `Err` and the contained error satisfies the predicate.
  #[napi]
  pub fn is_err_and(&self, f: Function<'a, Unknown<'a>, bool>) -> NapiResult<bool> {
    match &self.inner {
      Inner::Err(e) => f.call(e.clone()),
      Inner::Ok(_) => Ok(false),
    }
  }

  // ----- Adapter for each variant -----

  /// Converts to the `Ok` value, or `null` if this is `Err`.
  #[napi]
  pub fn get_ok(&self) -> Option<Unknown<'a>> {
    match &self.inner {
      Inner::Ok(v) => Some(v.clone()),
      Inner::Err(_) => None,
    }
  }

  /// Converts to the `Err` value, or `null` if this is `Ok`.
  #[napi]
  pub fn get_err(&self) -> Option<Unknown<'a>> {
    match &self.inner {
      Inner::Err(e) => Some(e.clone()),
      Inner::Ok(_) => None,
    }
  }

  // ----- Transforming contained values -----

  /// Maps the `Ok` value through `f`, leaving `Err` untouched.
  #[napi]
  pub fn map(&self, f: Function<'a>) -> NapiResult<JsResult<'a>> {
    match &self.inner {
      Inner::Ok(v) => Ok(JsResult { inner: Inner::Ok(f.call(v.clone())?) }),
      Inner::Err(e) => Ok(JsResult { inner: Inner::Err(e.clone()) }),
    }
  }

  /// Maps the `Err` value through `f`, leaving `Ok` untouched.
  #[napi]
  pub fn map_err(&self, f: Function<'a>) -> NapiResult<JsResult<'a>> {
    match &self.inner {
      Inner::Err(e) => Ok(JsResult { inner: Inner::Err(f.call(e.clone())?) }),
      Inner::Ok(v) => Ok(JsResult { inner: Inner::Ok(v.clone()) }),
    }
  }

  /// Returns `f(value)` if `Ok`, otherwise the provided `default`.
  #[napi]
  pub fn map_or(&self, default: Unknown<'a>, f: Function<'a>) -> NapiResult<Unknown<'a>> {
    match &self.inner {
      Inner::Ok(v) => f.call(v.clone()),
      Inner::Err(_) => Ok(default),
    }
  }

  /// Returns `f(value)` if `Ok`, otherwise `default(errValue)`.
  #[napi]
  pub fn map_or_else(&self, default: Function<'a>, f: Function<'a>) -> NapiResult<Unknown<'a>> {
    match &self.inner {
      Inner::Ok(v) => f.call(v.clone()),
      Inner::Err(e) => default.call(e.clone()),
    }
  }

  /// Calls `f` with the `Ok` value for its side effect, then returns `this`.
  #[napi]
  pub fn inspect(&self, f: Function<'a>) -> NapiResult<JsResult<'a>> {
    if let Inner::Ok(v) = &self.inner {
      f.call(v.clone())?;
    }
    Ok(self.clone())
  }

  /// Calls `f` with the `Err` value for its side effect, then returns `this`.
  #[napi]
  pub fn inspect_err(&self, f: Function<'a>) -> NapiResult<JsResult<'a>> {
    if let Inner::Err(e) = &self.inner {
      f.call(e.clone())?;
    }
    Ok(self.clone())
  }

  // ----- Boolean operators -----
  //
  // Omitted by design: napi classes are received by reference (`FromNapiRef`)
  // and never by value (`FromNapiValue`), and their lifetime parameter cannot be
  // threaded through a by-value `Result` argument or a callback that *returns* a
  // `Result`. The `and` / `or` / `and_then` / `or_else` forms can be composed in
  // JS from `is_ok` / `is_err` / `map` / `map_err` / `unwrap` instead.

  // ----- Extracting contained values -----

  /// Returns the `Ok` value, or throws if this is `Err`.
  #[napi]
  pub fn unwrap(&self) -> NapiResult<Unknown<'a>> {
    match &self.inner {
      Inner::Ok(v) => Ok(v.clone()),
      Inner::Err(_) => Err(Error::from_reason("called `Result::unwrap()` on an `Err` value")),
    }
  }

  /// Returns the `Err` value, or throws if this is `Ok`.
  #[napi]
  pub fn unwrap_err(&self) -> NapiResult<Unknown<'a>> {
    match &self.inner {
      Inner::Err(e) => Ok(e.clone()),
      Inner::Ok(_) => Err(Error::from_reason("called `Result::unwrap_err()` on an `Ok` value")),
    }
  }

  /// Returns the `Ok` value, or throws with `msg` if this is `Err`.
  #[napi]
  pub fn expect(&self, msg: String) -> NapiResult<Unknown<'a>> {
    match &self.inner {
      Inner::Ok(v) => Ok(v.clone()),
      Inner::Err(_) => Err(Error::from_reason(msg)),
    }
  }

  /// Returns the `Err` value, or throws with `msg` if this is `Ok`.
  #[napi]
  pub fn expect_err(&self, msg: String) -> NapiResult<Unknown<'a>> {
    match &self.inner {
      Inner::Err(e) => Ok(e.clone()),
      Inner::Ok(_) => Err(Error::from_reason(msg)),
    }
  }

  /// Returns the `Ok` value, or `default` if this is `Err`.
  #[napi]
  pub fn unwrap_or(&self, default: Unknown<'a>) -> Unknown<'a> {
    match &self.inner {
      Inner::Ok(v) => v.clone(),
      Inner::Err(_) => default,
    }
  }

  /// Returns the `Ok` value, or `f(errValue)` if this is `Err`.
  #[napi]
  pub fn unwrap_or_else(&self, f: Function<'a>) -> NapiResult<Unknown<'a>> {
    match &self.inner {
      Inner::Ok(v) => Ok(v.clone()),
      Inner::Err(e) => Ok(f.call(e.clone())?),
    }
  }
}

impl<'a> From<core::result::Result<Unknown<'a>, Unknown<'a>>> for JsResult<'a> {
  fn from(r: core::result::Result<Unknown<'a>, Unknown<'a>>) -> Self {
    match r {
      core::result::Result::Ok(v) => JsResult { inner: Inner::Ok(v) },
      core::result::Result::Err(e) => JsResult { inner: Inner::Err(e) },
    }
  }
}
