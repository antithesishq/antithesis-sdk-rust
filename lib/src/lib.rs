#[macro_use]
extern crate lazy_static;

pub mod assert;
pub mod lifecycle;
pub mod random;
mod internal;
pub use assert::assert_impl;
