use std::sync::atomic::{self, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize};

use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::{json, Value};

use crate::internal;

use super::AntithesisLocationInfo;

// Types and traits that model the SDK filtering of numerical guidance reporting.
// For assertions like "always (x < y)", we would like to only report the most extreme
// violations seen so far, which is implemented by having a `Guard` that keep a maximizing
// watermark on the difference (x - y).
// The `AtomicMinMax` trait requirement allows multiple concurrent update to the watermark.

// NOTE: The structures setup in this modules allow `Guard` to be generic over the numeric
// type (or even any partially ordered type).
// But due to some limitation of stable Rust, we are only instanciating `Guard<f64>` by
// converting the result of all `x - y` into `f64`.
// See the impl `numeric_guidance_helper` for more details on the limitation.
// Once that is lifted, some implementations of `Diff` can be changed to truly take advantage
// of the zero-cost polymorphism that `Guard` provides.
pub struct Guard<const MAX: bool, T: AtomicMinMax> {
    mark: T::Atomic,
}

pub trait Extremal {
    const MIN: Self;
    const MAX: Self;
}

impl<const MAX: bool, T: AtomicMinMax> Guard<MAX, T>
where T::Atomic: Extremal {
    pub const fn init() -> Self {
        let mark = if MAX { T::Atomic::MIN } else { T::Atomic::MAX };
        Self { mark }
    }
}

pub trait AtomicMinMax {
    type Atomic;
    fn fetch_min(current: &Self::Atomic, other: Self, ordering: atomic::Ordering) -> Self;
    fn fetch_max(current: &Self::Atomic, other: Self, ordering: atomic::Ordering) -> Self;
}

impl<const MAX: bool, T: AtomicMinMax + PartialOrd + Copy> Guard<MAX, T> {
    pub fn should_emit(&self, new: T) -> bool {
        use std::cmp::Ordering::*;
        if MAX {
            let max = T::fetch_max(&self.mark, new, atomic::Ordering::SeqCst);
            matches!(max.partial_cmp(&new), None | Some(Less | Equal))
        } else {
            let min = T::fetch_min(&self.mark, new, atomic::Ordering::SeqCst);
            matches!(min.partial_cmp(&new), None | Some(Greater | Equal))
        }
    }
}

pub trait Diff {
    type Output;

    fn diff(&self, other: Self) -> Self::Output;
}

macro_rules! impl_extremal {
    ($($t:ty)*) => {$(
        impl Extremal for $t {
            const MIN: $t = <$t>::MIN;
            const MAX: $t = <$t>::MAX;
        }
    )*}
}

impl_extremal! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 f32 f64 }

macro_rules! impl_extremal_atomic {
    ($(($t:ty, $raw_t:ty))*) => {$(
        #[allow(clippy::declare_interior_mutable_const)]
        impl Extremal for $t {
            const MIN: $t = <$t>::new(<$raw_t>::MIN);
            const MAX: $t = <$t>::new(<$raw_t>::MAX);
        }
    )*}
}

impl_extremal_atomic! { (AtomicUsize, usize) (AtomicU8, u8) (AtomicU16, u16) (AtomicU32, u32) (AtomicU64, u64) (AtomicIsize, isize) (AtomicI8, i8) (AtomicI16, i16) (AtomicI32, i32) (AtomicI64, i64) }

// For atomic floats, their minimal/maximal elements are `-inf` and `+inf` respectively.

#[allow(clippy::declare_interior_mutable_const)]
impl Extremal for AtomicF32 {
    const MIN: Self = AtomicF32(AtomicU32::new(0xff800000));
    const MAX: Self = AtomicF32(AtomicU32::new(0x7f800000));
}

#[allow(clippy::declare_interior_mutable_const)]
impl Extremal for AtomicF64 {
    const MIN: Self = AtomicF64(AtomicU64::new(0xfff0000000000000));
    const MAX: Self = AtomicF64(AtomicU64::new(0x7ff0000000000000));
}

macro_rules! impl_atomic_min_max {
    ($(($t:ty, $atomic_t:ty))*) => {$(
        impl AtomicMinMax for $t {
            type Atomic = $atomic_t;

            fn fetch_min(current: &Self::Atomic, other: Self, ordering: atomic::Ordering) -> Self {
                current.fetch_min(other, ordering)
            }

            fn fetch_max(current: &Self::Atomic, other: Self, ordering: atomic::Ordering) -> Self {
                current.fetch_max(other, ordering)
            }
        }
    )*};
}

impl_atomic_min_max! { (usize, AtomicUsize) (u8, AtomicU8) (u16, AtomicU16) (u32, AtomicU32) (u64, AtomicU64) (isize, AtomicIsize) (i8, AtomicI8) (i16, AtomicI16) (i32, AtomicI32) (i64, AtomicI64) }

macro_rules! impl_atomic_min_max_float {
    ($(($t:ty, $atomic_t:ident, $store_t:ty))*) => {$(
        pub struct $atomic_t($store_t);

        impl AtomicMinMax for $t {
            type Atomic = $atomic_t;

            // TODO: Check the atomic orderings are used properly in general.
            // Right now we are always passing SeqCst, which should be fine.
            fn fetch_min(current: &Self::Atomic, other: Self, ordering: atomic::Ordering) -> Self {
                <$t>::from_bits(current.0.fetch_update(ordering, ordering, |x| Some(<$t>::from_bits(x).min(other).to_bits())).unwrap())
            }

            fn fetch_max(current: &Self::Atomic, other: Self, ordering: atomic::Ordering) -> Self {
                <$t>::from_bits(current.0.fetch_update(ordering, ordering, |x| Some(<$t>::from_bits(x).max(other).to_bits())).unwrap())
            }
        }
    )*};
}

impl_atomic_min_max_float! { (f32, AtomicF32, AtomicU32) (f64, AtomicF64, AtomicU64)}

macro_rules! impl_diff_unsigned {
    ($($t:ty)*) => {$(
        impl Diff for $t {
            type Output = f64;

            fn diff(&self, other: Self) -> Self::Output {
                if *self < other {
                    -((other - self) as f64)
                } else {
                    (self - other) as f64
                }
            }
        }
    )*};
}

impl_diff_unsigned! { usize u8 u16 u32 u64 u128 }

macro_rules! impl_diff_signed {
    ($(($t:ty, $unsigned_t:ty))*) => {$(
        impl Diff for $t {
            type Output = f64;

            fn diff(&self, other: Self) -> Self::Output {
                if *self < other {
                    // For correctness, see
                    // https://github.com/rust-lang/rust/blob/11e760b7f4e4aaa11bf51a64d4bb7f1171f6e466/library/core/src/num/int_macros.rs#L3443-L3456
                    -((other as $unsigned_t).wrapping_sub(*self as $unsigned_t) as f64)
                } else {
                    (*self as $unsigned_t).wrapping_sub(other as $unsigned_t) as f64
                }
            }
        }
    )*};
}

impl_diff_signed! { (isize, usize) (i8, u8) (i16, u16) (i32, u32) (i64, u64) (i128, u128) }

macro_rules! impl_diff_float {
    ($($t:ty)*) => {$(
        impl Diff for $t {
            type Output = f64;

            fn diff(&self, other: Self) -> Self::Output {
                (self - other) as f64
            }
        }
    )*};
}

impl_diff_float! { f32 f64 }

#[derive(Copy, Clone, Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum GuidanceType {
    Numeric,
    Boolean,
    Json,
}

#[derive(Serialize)]
struct GuidanceInfo {
    guidance_type: GuidanceType,
    message: String,
    id: String,
    location: AntithesisLocationInfo,
    maximize: bool,
    guidance_data: Value,
    hit: bool,
}

pub struct GuidanceCatalogInfo {
    pub guidance_type: GuidanceType,
    pub message: &'static str,
    pub id: &'static str,
    pub class: &'static str,
    pub function: &'static Lazy<&'static str>,
    pub file: &'static str,
    pub begin_line: u32,
    pub begin_column: u32,
    pub maximize: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn guidance_impl(
    guidance_type: GuidanceType,
    message: String,
    id: String,
    class: String,
    function: String,
    file: String,
    begin_line: u32,
    begin_column: u32,
    maximize: bool,
    guidance_data: Value,
    hit: bool,
) {
    let location = AntithesisLocationInfo { class, function, file, begin_line, begin_column };
    let guidance = GuidanceInfo {
        guidance_type, message, id, location, maximize, guidance_data, hit
    };

    internal::dispatch_output(&json!({ "antithesis_guidance": guidance }));
}
