use std::collections::HashMap;
use std::sync::atomic::{self, AtomicU64};
use std::sync::RwLock;

use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::{json, Value};

use crate::internal;

use super::AntithesisLocationInfo;

// Types and traits that model the SDK filtering of numerical guidance
// reporting. For assertions like "always (x < y)", we would like to only
// report the most extreme violations seen so far, which is implemented by
// having a `Guard` keep a watermark on the difference (x - y).
//
// The watermark is an f64 held as bits in an `AtomicU64` so that concurrent
// updates need only a CAS. `Diff` still converts every operand type's
// difference into f64.
const UNSET: u64 = 0x7ff8_0000_0000_0000;

pub struct Guard<const MAX: bool> {
    mark: AtomicU64,
}

impl<const MAX: bool> Guard<MAX> {
    pub const fn init() -> Self {
        Self { mark: AtomicU64::new(UNSET) }
    }

    /// Whether `gap` improves on the watermark, taking it if so.
    pub fn should_emit(&self, gap: f64) -> bool {
        let mut current = self.mark.load(atomic::Ordering::SeqCst);
        loop {
            if current != UNSET {
                let mark = f64::from_bits(current);
                if if MAX { mark >= gap } else { mark <= gap } {
                    return false;
                }
            }
            match self.mark.compare_exchange_weak(
                current,
                gap.to_bits(),
                atomic::Ordering::SeqCst,
                atomic::Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }
}

pub trait Diff {
    type Output;

    fn diff(&self, other: Self) -> Self::Output;

    fn is_finite(&self) -> bool;
}

// Convert each operand to f64 then subtract to match the platform's comparison
macro_rules! impl_diff {
    ($($t:ty)*) => {$(
        impl Diff for $t {
            type Output = f64;

            fn is_finite(&self) -> bool {
                (*self as f64).is_finite()
            }

            fn diff(&self, other: Self) -> Self::Output {
                *self as f64 - other as f64
            }
        }
    )*};
}

impl_diff! { usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 f32 f64 }

#[derive(Copy, Clone, Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum GuidanceType {
    Numeric,
    Boolean,
    Json,
}

#[derive(Serialize)]
struct GuidanceInfo<'a> {
    guidance_type: GuidanceType,
    message: &'a str,
    id: &'a str,
    location: AntithesisLocationInfo<'a>,
    maximize: bool,
    #[serde(skip_serializing_if = "Value::is_null")]
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

/// The catalog-registration event for one guidance declaration, emitted by
/// `antithesis_init()` after the assertion events.
// The explicit deref is needed on older compilers, which otherwise infer
// `T = str` from the field type and reject the call.
#[allow(clippy::explicit_auto_deref)]
pub(crate) fn catalog_event(info: &GuidanceCatalogInfo) -> Value {
    let location = AntithesisLocationInfo {
        class: info.class,
        function: *Lazy::force(info.function),
        file: info.file,
        begin_line: info.begin_line,
        begin_column: info.begin_column,
    };
    let guidance = GuidanceInfo {
        guidance_type: info.guidance_type,
        message: info.message,
        id: info.id,
        location,
        maximize: info.maximize,
        guidance_data: json!(null),
        hit: false,
    };
    json!({ "antithesis_guidance": guidance })
}

enum NumericGuidanceGate {
    Max(Guard<true>),
    Min(Guard<false>),
}

impl NumericGuidanceGate {
    fn new(maximize: bool) -> NumericGuidanceGate {
        if maximize {
            NumericGuidanceGate::Max(Guard::init())
        } else {
            NumericGuidanceGate::Min(Guard::init())
        }
    }

    fn should_emit(&self, gap: f64) -> bool {
        match self {
            NumericGuidanceGate::Max(guard) => guard.should_emit(gap),
            NumericGuidanceGate::Min(guard) => guard.should_emit(gap),
        }
    }
}

fn should_emit(id: &str, maximize: bool, gap: f64) -> bool {
    static GUIDANCE_TRACKER: Lazy<RwLock<HashMap<String, NumericGuidanceGate>>> =
        Lazy::new(|| RwLock::new(HashMap::new()));

    if let Some(gate) = GUIDANCE_TRACKER.read().unwrap().get(id) {
        return gate.should_emit(gap);
    }

    let mut tracker = GUIDANCE_TRACKER.write().unwrap();
    tracker
        .entry(id.to_owned())
        .or_insert_with(|| NumericGuidanceGate::new(maximize))
        .should_emit(gap)
}

/// Antithesis cannot steer toward NaN or an infinity, so a hit with such an
/// operand never reaches the watermark; the first skip is reported.
pub fn finite_operands<T: Diff>(left: &T, right: &T, id: &str) -> bool {
    let finite = left.is_finite() && right.is_finite();
    if !finite {
        crate::details::report_non_finite_guidance(id);
    }
    finite
}

/// This is a low-level method designed to be used by third-party frameworks.
/// Regular users of the assert package should not call it.
#[allow(clippy::too_many_arguments)]
pub fn numeric_guidance_raw<T: Diff<Output = f64> + Serialize + Copy>(
    left: T,
    right: T,
    maximize: bool,
    message: String,
    id: String,
    class: String,
    function: String,
    file: String,
    begin_line: u32,
    begin_column: u32,
    hit: bool,
) {
    let guidance_data = if hit {
        if !finite_operands(&left, &right, &id) || !should_emit(&id, maximize, left.diff(right)) {
            return;
        }
        let operands = crate::details::NumericOperands { left: &left, right: &right };
        match crate::details::guidance(&operands, &id) {
            Some(data) => data,
            None => return,
        }
    } else {
        Value::Null
    };

    guidance_impl(
        GuidanceType::Numeric,
        message.as_str(),
        id.as_str(),
        class.as_str(),
        function.as_str(),
        file.as_str(),
        begin_line,
        begin_column,
        maximize,
        guidance_data,
        hit,
    )
}

/// This is a low-level method designed to be used by third-party frameworks.
/// Regular users of the assert package should not call it.
#[allow(clippy::too_many_arguments)]
pub fn boolean_guidance_raw(
    guidance_data: Value,
    maximize: bool,
    message: String,
    id: String,
    class: String,
    function: String,
    file: String,
    begin_line: u32,
    begin_column: u32,
    hit: bool,
) {
    let guidance_data = if hit {
        match crate::details::guidance(&guidance_data, &id) {
            Some(data) => data,
            None => return,
        }
    } else {
        Value::Null
    };

    guidance_impl(
        GuidanceType::Boolean,
        message.as_str(),
        id.as_str(),
        class.as_str(),
        function.as_str(),
        file.as_str(),
        begin_line,
        begin_column,
        maximize,
        guidance_data,
        hit,
    )
}

/// Emits one guidance packet. `guidance_data` is already inspected (see
/// `details::guidance`), or null for a declaration; a caller whose data
/// failed to serialize does not get here. Deliberately not generic: the
/// packet construction and output compile once, in this crate, rather than
/// into every caller for every operand type.
#[allow(clippy::too_many_arguments)]
pub fn guidance_impl<'a>(
    guidance_type: GuidanceType,
    message: &'a str,
    id: &'a str,
    class: &'a str,
    function: &'a str,
    file: &'a str,
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
