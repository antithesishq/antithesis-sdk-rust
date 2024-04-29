/// The assert module allows you to define new test properties 
/// for your program or workload.
///
/// If the environment variable ANTITHESIS_SDK_LOCAL_OUTPUT is 
/// set, these macros and functions will log to the file pointed 
/// to by that variable using a structured JSON format defined in 
/// the Antithesis SDK docs.
/// This allows you to make use of the Antithesis assertions module 
/// in your regular testing, or even in production. In particular, 
/// very few assertions frameworks offer a convenient way to define 
/// [Sometimes assertions], but they can be quite useful even outside 
/// Antithesis.
///
/// Each macro/function in this module takes a parameter called message. 
/// This value of this parameter will become part of the name of 
/// the test property defined by the function, and will be viewable 
/// in your [triage report], so it should be human interpretable. 
/// Assertions in different parts of your code with the same message 
/// value will be grouped into the same test property, but if one of 
/// them fails you will be able to see which file and line number are 
/// associated with each failure.
///
/// Each macro/function also takes a parameter called details. 
/// This parameter allows you to optionally provide a JSON representation
/// of context information that will be viewable in the 'details' 
/// tab for any example or counterexample of the associated property.
pub mod assert;

// External crates used in assertion macros
#[doc(hidden)]
pub use once_cell;
#[doc(hidden)]
pub use linkme;

/// The lifecycle module contains functions which inform the Antithesis 
/// environment that particular test phases or milestones have been reached.
pub mod lifecycle;


/// The random module provides an interface that allows your program 
/// to ask the Antithesis platform for random entropy. These functions 
/// are also safe to call outside the Antithesis environment, where 
/// they will fall back on values from the rust std library
///
/// These functions should not be used to seed a conventional PRNG, 
/// and should not have their return values stored and used to make a 
/// decision at a later time. Doing either of these things makes it 
/// much harder for the Antithesis platform to control the history of 
/// your program's execution, and also makes it harder for Antithesis 
/// to learn which inputs provided at which times are most fruitful. 
/// Instead, you should call a function from the random package every 
/// time your program or workload needs to make a decision, at the 
/// moment that you need to make the decision.
pub mod random;

mod internal;

/// Convenience to import all macros and functions
pub mod prelude;

/// Global initialization logic
pub fn antithesis_init() {
    Lazy::force(&internal::LIB_HANDLER);
    Lazy::force(&assert::INIT_CATALOG);
}

pub use assert::{assert_impl, assert_raw, CatalogInfo};
use once_cell::sync::Lazy;
