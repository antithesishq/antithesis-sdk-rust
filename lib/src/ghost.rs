//! A compile-time-checked layer for writing *read-only* test properties and
//! *ghost state* on top of the rest of the SDK.
//!
//! This module addresses two problems that arise when you sprinkle property
//! checks (assertions, guidance, randomness) throughout a system under test:
//!
//! 1. **Property code must never mutate the system it observes.** If the
//!    expression inside an assertion has a side effect that the program relies
//!    on, then the program behaves differently depending on whether the SDK is
//!    compiled in — a silent, dangerous divergence. The [`observe!`] macro
//!    turns "my property accidentally mutated the system" into a *compile
//!    error*.
//!
//! 2. **You sometimes want state that exists only to express properties.** In
//!    formal verification this is called *ghost state*: auxiliary state that
//!    exists only to express properties and is erased from production builds,
//!    so it can never change the behavior of the system it describes. A common
//!    use is holding a *reference model* of the system — a simplified shadow to
//!    diff the real system against — but it can just as well be an event
//!    counter or the set of keys you have seen. [`GhostState<T>`] is opaque
//!    ghost state whose *only* mutator is [`GhostState::mutate`] and whose only
//!    reader is [`observe!`]. Because nothing else can touch it, and because
//!    the closures that do touch it are provably incapable of mutating the
//!    surrounding system, the entire ghost state — its construction, mutation,
//!    and observation — can be safely compiled out with **zero** effect on
//!    program behavior.
//!
//! # How the read-only guarantee works
//!
//! Everything here is enforced with a single, ordinary Rust trait bound:
//! [`Fn`]. A closure that satisfies `Fn` captures its environment by shared
//! reference (or copy) only — the borrow checker rejects any attempt to take a
//! `&mut` borrow of, reassign, or move out of a captured variable. No `unsafe`,
//! no procedural macros, no AST inspection. `observe!`, [`GhostState::new`]
//! and [`GhostState::mutate`] all require `Fn` closures, so any attempt to
//! mutate the surrounding system from inside them fails to compile.
//!
//! ```compile_fail
//! use antithesis_sdk::observe;
//! let mut counter = 0u64;
//! // ERROR: cannot assign to `counter`, it is captured in a `Fn` closure
//! observe!(|| { counter += 1; });
//! ```
//!
//! ```compile_fail
//! use antithesis_sdk::observe;
//! let mut items = vec![1, 2, 3];
//! // ERROR: `Vec::pop` needs `&mut`, which a `Fn` closure cannot obtain
//! observe!(|| { items.pop(); });
//! ```
//!
//! Mutable *locals* created inside the closure are unaffected — the bound only
//! constrains captures:
//!
//! ```
//! use antithesis_sdk::observe;
//! let readings = [3u64, 7, 1];
//! observe!(|| {
//!     let mut total = 0;      // local: fine
//!     for r in &readings {    // reading the environment: fine
//!         total += *r;
//!     }
//!     let _ = total;
//! });
//! ```
//!
//! # Compiling out
//!
//! This layer is gated on the crate's `full` feature, just like the rest of the
//! SDK. When `full` is disabled, [`GhostState<T>`] becomes a zero-sized type,
//! `new`'s initializer is never run, and the `mutate`/`observe!` closures are
//! type-checked but **never executed**. The read-only and type checks still
//! happen in *every* build configuration, so the same source compiles
//! identically whether or not the SDK is active.
//!
//! # Limitation
//!
//! `Fn` enforces read-only access *through the reference system*. It stops
//! `&mut` borrows, reassignment, moves, and `&mut self` method calls, but it
//! does **not** stop interior mutability (`Cell`, `RefCell`, `Mutex`, atomics)
//! or `unsafe`. That is the borrow checker's definition of "read-only," and it
//! is the one seam in the guarantee.
//!
//! # Thread-safety
//!
//! [`GhostState<T>`] is deliberately single-threaded and lock-free:
//! [`mutate`](GhostState::mutate) takes `&mut self`. Introducing
//! synchronization here could perturb the ordering of the system under test,
//! and any concurrency should be a property of the system itself, not of the
//! Antithesis instrumentation. Wrap the ghost state in your own synchronization
//! primitive if you need to share it — that primitive then belongs to (and is
//! visible to Antithesis as part of) your system.

/// Opaque *ghost state* whose inner `T` can be read only through [`observe!`]
/// and mutated only through [`GhostState::mutate`].
///
/// When the crate's `full` feature is enabled this holds a `T`; otherwise it is
/// a zero-sized type and every access is compiled out. See the [module
/// docs](self) for the full rationale.
///
/// The common base traits are derived and available whenever `T` implements
/// them, with identical bounds in both build configurations.
///
/// # Example
///
/// ```
/// use antithesis_sdk::{observe, ghost::GhostState};
/// use serde_json::json;
///
/// // A tiny reference model: the number of items we believe are in flight.
/// let mut in_flight = GhostState::new(|| 0i64);
///
/// // Drive it from your system's events.
/// in_flight.mutate(|n| *n += 1);
/// in_flight.mutate(|n| *n -= 1);
///
/// // Check a property over it — read-only.
/// observe!(in_flight, |n: &i64| {
///     antithesis_sdk::assert_always!(*n >= 0, "never negative in flight", &json!({ "n": *n }));
/// });
/// ```
#[cfg(feature = "full")]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GhostState<T>(T);

/// See the [`full`-featured definition](GhostState) for documentation.
#[cfg(not(feature = "full"))]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GhostState<T>(::core::marker::PhantomData<T>);

#[cfg(feature = "full")]
impl<T> GhostState<T> {
    /// Creates ghost state, initializing the inner `T` with `init`.
    ///
    /// `init` is a read-only closure (it may observe the surrounding system but
    /// not mutate it). When the SDK is compiled out, `init` is **never called**
    /// and no `T` is constructed.
    pub fn new<F: Fn() -> T>(init: F) -> Self {
        GhostState(init())
    }

    /// The sole way to mutate the ghost state.
    ///
    /// `f` receives an exclusive `&mut T` to the ghost state's interior (which
    /// it may freely mutate); everything it captures from the surrounding
    /// environment is read-only, enforced by the `Fn` bound. When the SDK is
    /// compiled out, `f` is type-checked but never called.
    pub fn mutate<F: Fn(&mut T)>(&mut self, f: F) {
        f(&mut self.0)
    }

    /// Private read accessor. The only callers are the crate-internal
    /// `__observe*` helpers in this module — there is deliberately **no** public
    /// way to obtain a `&T`, so ghost state can only be read from inside an
    /// `observe!` closure.
    fn inner(&self) -> &T {
        &self.0
    }
}

#[cfg(not(feature = "full"))]
impl<T> GhostState<T> {
    /// See the [`full`-featured definition](GhostState::new).
    #[allow(unused_variables)]
    pub fn new<F: Fn() -> T>(init: F) -> Self {
        GhostState(::core::marker::PhantomData)
    }

    /// See the [`full`-featured definition](GhostState::mutate).
    #[allow(unused_variables)]
    pub fn mutate<F: Fn(&mut T)>(&mut self, f: F) {}
}

// Generates the per-arity `__observeN` helper functions that back `observe!`.
//
// These are defined and instantiated *within this crate*, so the `full` cfg is
// resolved against this crate's features (not the downstream crate's). Each
// function carries the `Fn(&T0, ..)` bound that enforces read-only access and
// pins the closure's parameter types to the ghost states' inner types. When
// `full` is off the body is empty, so the closure is type-checked but never
// executed.
macro_rules! define_observe_helpers {
    ($( $name:ident ( $($ty:ident : $arg:ident),* ) ),* $(,)?) => {$(
        #[cfg(feature = "full")]
        #[doc(hidden)]
        #[allow(clippy::too_many_arguments)]
        pub fn $name<$($ty,)* F: Fn($(&$ty),*)>(
            $($arg: &GhostState<$ty>,)* f: F,
        ) {
            f($($arg.inner()),*)
        }

        #[cfg(not(feature = "full"))]
        #[doc(hidden)]
        #[allow(unused_variables, clippy::too_many_arguments)]
        pub fn $name<$($ty,)* F: Fn($(&$ty),*)>(
            $($arg: &GhostState<$ty>,)* f: F,
        ) {}
    )*};
}

define_observe_helpers! {
    __observe0(),
    __observe1(T0: m0),
    __observe2(T0: m0, T1: m1),
    __observe3(T0: m0, T1: m1, T2: m2),
    __observe4(T0: m0, T1: m1, T2: m2, T3: m3),
    __observe5(T0: m0, T1: m1, T2: m2, T3: m3, T4: m4),
    __observe6(T0: m0, T1: m1, T2: m2, T3: m3, T4: m4, T5: m5),
    __observe7(T0: m0, T1: m1, T2: m2, T3: m3, T4: m4, T5: m5, T6: m6),
    __observe8(T0: m0, T1: m1, T2: m2, T3: m3, T4: m4, T5: m5, T6: m6, T7: m7),
}

/// Runs a read-only observation block, optionally borrowing one or more
/// [`GhostState`]s.
///
/// The block is a closure that may call any of this crate's APIs — assertions,
/// guidance, randomness, lifecycle — but is forbidden by the compiler from
/// mutating anything it captures from the surrounding system (see the [module
/// docs](self)). With no ghost state it is a pure property block; with ghost
/// state it receives a shared `&T` to each one's interior. It evaluates to `()`.
///
/// When the SDK is compiled out the closure is type-checked but never executed.
///
/// Up to 8 ghost states may be observed at once.
///
/// # Examples
///
/// ```
/// use antithesis_sdk::{observe, ghost::GhostState};
/// use serde_json::json;
///
/// // No ghost state: a plain property block over the surrounding system.
/// let temperature = 42;
/// observe!(|| {
///     antithesis_sdk::assert_always!(temperature < 100, "not overheating", &json!({ "t": temperature }));
/// });
///
/// // One ghost state.
/// let seen = GhostState::new(|| 0u64);
/// observe!(seen, |count: &u64| {
///     let _ = *count;
/// });
///
/// // Several ghost states with a trailing comma.
/// let a = GhostState::new(|| 1u64);
/// let b = GhostState::new(|| String::from("ok"));
/// observe!(a, b, |x: &u64, y: &String| {
///     let _ = (*x, y.len());
/// },);
/// ```
#[macro_export]
macro_rules! observe {
    ($closure:expr $(,)?) => {
        $crate::ghost::__observe0($closure)
    };
    ($m0:expr, $closure:expr $(,)?) => {
        $crate::ghost::__observe1(&$m0, $closure)
    };
    ($m0:expr, $m1:expr, $closure:expr $(,)?) => {
        $crate::ghost::__observe2(&$m0, &$m1, $closure)
    };
    ($m0:expr, $m1:expr, $m2:expr, $closure:expr $(,)?) => {
        $crate::ghost::__observe3(&$m0, &$m1, &$m2, $closure)
    };
    ($m0:expr, $m1:expr, $m2:expr, $m3:expr, $closure:expr $(,)?) => {
        $crate::ghost::__observe4(&$m0, &$m1, &$m2, &$m3, $closure)
    };
    ($m0:expr, $m1:expr, $m2:expr, $m3:expr, $m4:expr, $closure:expr $(,)?) => {
        $crate::ghost::__observe5(&$m0, &$m1, &$m2, &$m3, &$m4, $closure)
    };
    ($m0:expr, $m1:expr, $m2:expr, $m3:expr, $m4:expr, $m5:expr, $closure:expr $(,)?) => {
        $crate::ghost::__observe6(&$m0, &$m1, &$m2, &$m3, &$m4, &$m5, $closure)
    };
    ($m0:expr, $m1:expr, $m2:expr, $m3:expr, $m4:expr, $m5:expr, $m6:expr, $closure:expr $(,)?) => {
        $crate::ghost::__observe7(&$m0, &$m1, &$m2, &$m3, &$m4, &$m5, &$m6, $closure)
    };
    ($m0:expr, $m1:expr, $m2:expr, $m3:expr, $m4:expr, $m5:expr, $m6:expr, $m7:expr, $closure:expr $(,)?) => {
        $crate::ghost::__observe8(&$m0, &$m1, &$m2, &$m3, &$m4, &$m5, &$m6, &$m7, $closure)
    };
    ($($rest:tt)*) => {
        ::std::compile_error!(
r#"Invalid syntax when calling macro `observe`.
Example usage:
    `observe!(|| { /* read-only property checks */ })`
    `observe!(ghost, |g: &T| { /* read-only checks over `g` and the environment */ })`
Up to 8 ghost states may be observed at once; the final argument must be the closure."#
        );
    };
}
