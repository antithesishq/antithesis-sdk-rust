/// The assert module enables defining [test properties](https://antithesis.com/docs/using_antithesis/properties.html)
/// about your program or [workload](https://antithesis.com/docs/getting_started/workload.html).
///
/// Whenever the environment variable ANTITHESIS_SDK_LOCAL_OUTPUT is
/// set, these macros and functions will log to the file pointed
/// to by that variable using a structured JSON format defined in
/// the Antithesis SDK docs.
/// This allows you to make use of the Antithesis assertions module
/// in your regular testing, or even in production. In particular,
/// very few assertions frameworks offer a convenient way to define
/// [Sometimes assertions](https://antithesis.com/docs/best_practices/sometimes_assertions.html), but they can be quite useful even outside
/// Antithesis.
///
/// Each macro/function in this module takes a parameter called message, which is
/// a human readable identifier used to aggregate assertions.
/// Antithesis generates one test property per unique message and this test property will be named "message" in the [triage report](https://antithesis.com/docs/reports/triage.html).
///
/// Each macro/function also takes a parameter called details, which is a key-value map of optional additional information provided by the user to add context for assertion failures.
/// The information that is logged will appear in the logs section of a [triage report](https://antithesis.com/docs/reports/triage.html).
/// Normally the values passed to details are evaluated at runtime.
pub mod assert;

// External crates used in assertion macros
#[doc(hidden)]
pub use linkme;
#[doc(hidden)]
pub use once_cell;

/// The lifecycle module contains functions which inform the Antithesis
/// environment that particular test phases or milestones have been reached.
pub mod lifecycle;

/// The random module provides functions that request both structured and unstructured randomness from the Antithesis environment.
///
/// These functions should not be used to seed a conventional PRNG, and should not have their return values stored and used to make a decision at a later time.
/// Doing either of these things makes it much harder for the Antithesis platform to control the history of your program's execution, and also makes it harder for Antithesis to learn which inputs provided at which times are most fruitful.
/// Instead, you should call a function from the random package every time your program or [workload](https://antithesis.com/docs/getting_started/workload.html) needs to make a decision, at the moment that you need to make the decision.
///
/// These functions are also safe to call outside the Antithesis environment, where
/// they will fall back on values from the rust std library.
///
pub mod random;

mod internal;

/// Convenience to import all macros and functions
pub mod prelude;

/// Global initialization logic.  Performs registration of the
/// Antithesis assertion catalog.  This should be invoked as early as
/// possible during program execution (invoke first thing in main).
///
/// If invoked more than once, only the first call will result
/// in the assertion catalog being registered.  If not invoked at all,
/// the assertion catalog will be registered upon the first
/// assertion that is encountered at runtime.
///
/// Warning - if assertions are included in a program, and not
/// encountered at runtime, and antithesis_init() has not been
/// called, then the assertions will not be reported.
///
/// Example:
///
/// ```
/// use std::env;
/// use serde_json::{json};
/// use antithesis_sdk::{antithesis_init, assert_unreachable};
///
/// fn main() {
///     if (env::args_os().len() == 1888999778899) {
///         assert_unreachable!("Unable to provide trillions of arguments", &json!({}));
///     }
///     
///     // if antithesis_init() is omitted, the above unreachable will
///     // not be reported
///     antithesis_init();
/// }
/// ```
#[allow(clippy::needless_doctest_main)]
pub fn antithesis_init() {
    Lazy::force(&internal::LIB_HANDLER);
    Lazy::force(&assert::INIT_CATALOG);
}

pub use assert::{assert_impl, assert_raw, CatalogInfo};
use once_cell::sync::Lazy;

pub use crate::internal::LOCAL_OUTPUT;
