use crate::details::Details;
#[cfg(feature = "full")]
use crate::internal;
#[cfg(feature = "full")]
use linkme::distributed_slice;
#[cfg(feature = "full")]
use once_cell::sync::Lazy;
use serde::Serialize;
#[cfg(feature = "full")]
use serde_json::json;
use serde_json::Value;
use std::sync::atomic::AtomicU64;
#[cfg(feature = "full")]
use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc, RwLock},
};

#[doc(hidden)]
#[cfg(feature = "full")]
pub mod guidance;
mod macros;

/// Catalog of all antithesis assertions provided
#[doc(hidden)]
#[distributed_slice]
#[cfg(feature = "full")]
pub static ANTITHESIS_CATALOG: [AssertionCatalogInfo];

/// Catalog of all antithesis guidances provided
#[doc(hidden)]
#[distributed_slice]
#[cfg(feature = "full")]
pub static ANTITHESIS_GUIDANCE_CATALOG: [self::guidance::GuidanceCatalogInfo];

#[cfg(feature = "full")]
pub(crate) static INIT_CATALOG: Lazy<()> = Lazy::new(|| {
    for event in crate::catalog::registrations() {
        internal::dispatch_output(&event);
    }
});

/// The catalog-registration event for one assertion declaration, emitted by
/// both `antithesis_init()` and `catalog::write`.
#[cfg(feature = "full")]
pub(crate) fn catalog_event(info: &AssertionCatalogInfo) -> Value {
    let f_name: &str = info.function.as_ref();
    let details = json!(null);
    let assertion = AssertionInfo::new(
        info.assert_type,
        info.display_type,
        info.condition,
        info.message,
        info.class,
        f_name,
        info.file,
        info.begin_line,
        info.begin_column,
        false, /* hit */
        info.must_hit,
        info.id,
        &details,
    );
    json!({ "antithesis_assert": &assertion })
}

pub struct TrackingInfo {
    pub pass_count: AtomicU64,
    pub fail_count: AtomicU64,
}

impl Default for TrackingInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingInfo {
    pub const fn new() -> Self {
        TrackingInfo {
            pass_count: AtomicU64::new(0),
            fail_count: AtomicU64::new(0),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize)]
#[serde(rename_all(serialize = "lowercase"))]
pub enum AssertType {
    Always,
    Sometimes,
    Reachability,
}

#[derive(Serialize, Debug)]
struct AntithesisLocationInfo<'a> {
    class: &'a str,
    function: &'a str,
    file: &'a str,
    begin_line: u32,
    begin_column: u32,
}

/// Internal representation for assertion catalog
#[doc(hidden)]
#[derive(Debug)]
#[cfg(feature = "full")]
pub struct AssertionCatalogInfo {
    pub assert_type: AssertType,
    pub display_type: &'static str,
    pub condition: bool,
    pub message: &'static str,
    pub class: &'static str,
    pub function: &'static Lazy<&'static str>,
    pub file: &'static str,
    pub begin_line: u32,
    pub begin_column: u32,
    pub must_hit: bool,
    pub id: &'static str,
}

#[derive(Serialize)]
struct AssertionInfo<'a> {
    assert_type: AssertType,
    display_type: &'a str,
    condition: bool,
    message: &'a str,
    location: AntithesisLocationInfo<'a>,
    hit: bool,
    must_hit: bool,
    id: &'a str,
    // Serialized separately so declarations never evaluate the details serializer.
    #[serde(skip)]
    details: &'a dyn Details,
}

impl<'a> AssertionInfo<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assert_type: AssertType,
        display_type: &'a str,
        condition: bool,
        message: &'a str,
        class: &'a str,
        function: &'a str,
        file: &'a str,
        begin_line: u32,
        begin_column: u32,
        hit: bool,
        must_hit: bool,
        id: &'a str,
        details: &'a dyn Details,
    ) -> Self {
        let location = AntithesisLocationInfo {
            class,
            function,
            file,
            begin_line,
            begin_column,
        };

        AssertionInfo {
            assert_type,
            display_type,
            condition,
            message,
            location,
            hit,
            must_hit,
            id,
            details,
        }
    }
}

#[cfg(feature = "full")]
impl AssertionInfo<'_> {
    // AssertionInfo::track_entry() determines if the assertion should
    // actually be emitted:
    //
    // [X] If this is an assertion catalog
    // registration (assertion.hit == false) then it is emitted.
    //
    // [X] if `condition` is true increment the tracker_entry.pass_count,
    // otherwise increment the tracker_entry.fail_count.
    //
    // [X] if `condition` is true and tracker_entry_pass_count == 1 then
    // actually emit the assertion.
    //
    // [X] if `condition` is false and tracker_entry_fail_count == 1 then
    // actually emit the assertion.

    fn track_entry(&self, info: Option<&TrackingInfo>) {
        // Requirement: Catalog entries must always will emit()
        if !self.hit {
            self.emit();
            return;
        }

        // Record the condition in the associated TrackingInfo entry,
        // and emit the assertion when first seeing a condition
        let emitting = match (info, self.condition) {
            (None, _) => true,
            (Some(info), true) => {
                let prior_value = info.pass_count.fetch_add(1, Ordering::SeqCst);
                prior_value == 0
            }
            (Some(info), false) => {
                let prior_value = info.fail_count.fetch_add(1, Ordering::SeqCst);
                prior_value == 0
            }
        };
        if emitting {
            Lazy::force(&INIT_CATALOG);
            self.emit();
        }
    }

    fn emit(&self) {
        let mut assertion = serde_json::to_value(self).unwrap();
        if self.hit {
            if let Some(details) = self.details.object(self.id) {
                assertion["details"] = Value::Object(details);
            }
        }
        let json_event = json!({ "antithesis_assert": assertion });
        internal::dispatch_output(&json_event)
    }
}

#[cfg(not(feature = "full"))]
impl AssertionInfo<'_> {
    fn track_entry(&self, _info: Option<&TrackingInfo>) {
        return;
    }
}

/// This is a low-level method designed to be used by third-party frameworks.
/// Regular users of the assert package should not call it.
///
/// This is primarily intended for use by adapters from other diagnostic tools
/// that intend to output Antithesis-style assertions.
///
/// Be certain to provide an assertion catalog entry for each assertion issued
/// with ``assert_raw()``.  Assertion catalog entries are also created using
/// ``assert_raw()``, by setting the value of the ``hit`` parameter to false.
///
/// Details may be any borrowed value implementing `Serialize`, handled as
/// the assertion macros handle theirs: objects are preserved, other non-null
/// values are wrapped under `"value"`, `&Value::Null` means no details, and
/// a serialization error emits an `antithesis_error` naming the assertion
/// `id` while the assertion itself is still emitted without details.
/// Catalog entries (`hit == false`) never serialize details.
///
/// Please refer to the general Antithesis documentation regarding the use of
/// the
/// [Fallback SDK](https://antithesis.com/docs/using_antithesis/sdk/fallback/assert/)
/// for additional information.
///
///
///
/// # Example
///
/// ```
/// use serde_json::{json};
/// use antithesis_sdk::{assert, random};
///
/// struct Votes {
///     num_voters: u32,
///     candidate_1: u32,
///     candidate_2: u32,
/// }
///
/// fn main() {
///     establish_catalog();
///    
///     let mut all_votes = Votes {
///         num_voters: 0,
///         candidate_1: 0,
///         candidate_2: 0,
///     };
///
///     for _voter in 0..100 {
///         tally_vote(&mut all_votes, random_bool(), random_bool());
///     }
/// }
///
/// fn random_bool() -> bool {
///     let v1 = random::get_random() % 2;
///     v1 == 1
/// }
///
/// fn establish_catalog() {
///     assert::assert_raw(
///         false,                            /* condition */
///         "Never extra votes".to_owned(),   /* message */
///         &json!({}),                       /* details */
///         "mycrate::stuff".to_owned(),      /* class */
///         "mycrate::tally_vote".to_owned(), /* function */
///         "src/voting.rs".to_owned(),       /* file */
///         20,                               /* line */
///         3,                                /* column */
///         false,                            /* hit */
///         true,                             /* must_hit */
///         assert::AssertType::Always,       /* assert_type */
///         "Always".to_owned(),              /* display_type */
///         "42-1005".to_owned()              /* id */
///     );
/// }
///
/// fn tally_vote(votes: &mut Votes, candidate_1: bool, candidate_2: bool) {
///     if candidate_1 || candidate_2 {
///         votes.num_voters += 1;
///     }
///     if candidate_1 {
///         votes.candidate_1 += 1;
///     };
///     if candidate_2 {
///         votes.candidate_2 += 1;
///     };
///
///     let num_votes = votes.candidate_1 + votes.candidate_2;
///     assert::assert_raw(
///         num_votes == votes.num_voters,    /* condition */
///         "Never extra votes".to_owned(),   /* message */
///         &json!({                          /* details */
///             "votes": num_votes,
///             "voters": votes.num_voters
///         }),                        
///         "mycrate::stuff".to_owned(),      /* class */
///         "mycrate::tally_vote".to_owned(), /* function */
///         "src/voting.rs".to_owned(),       /* file */
///         20,                               /* line */
///         3,                                /* column */
///         true,                             /* hit */
///         true,                             /* must_hit */
///         assert::AssertType::Always,       /* assert_type */
///         "Always".to_owned(),              /* display_type */
///         "42-1005".to_owned()              /* id */
///     );
/// }
///
/// // Run example with output to /tmp/x7.json
/// // ANTITHESIS_SDK_LOCAL_OUTPUT=/tmp/x7.json cargo test --doc
/// //
/// // Example output from /tmp/x7.json
/// // Contents may vary due to use of random::get_random()
/// //
/// // {"antithesis_sdk":{"language":{"name":"Rust","version":"1.69.0"},"sdk_version":"0.1.2","protocol_version":"1.0.0"}}
/// // {"assert_type":"always","display_type":"Always","condition":false,"message":"Never extra votes","location":{"class":"mycrate::stuff","function":"mycrate::tally_vote","file":"src/voting.rs","begin_line":20,"begin_column":3},"hit":false,"must_hit":true,"id":"42-1005"}
/// // {"assert_type":"always","display_type":"Always","condition":true,"message":"Never extra votes","location":{"class":"mycrate::stuff","function":"mycrate::tally_vote","file":"src/voting.rs","begin_line":20,"begin_column":3},"hit":true,"must_hit":true,"id":"42-1005","details":{"voters":1,"votes":1}}
/// // {"assert_type":"always","display_type":"Always","condition":false,"message":"Never extra votes","location":{"class":"mycrate::stuff","function":"mycrate::tally_vote","file":"src/voting.rs","begin_line":20,"begin_column":3},"hit":true,"must_hit":true,"id":"42-1005","details":{"voters":3,"votes":4}}
/// ```
#[allow(clippy::too_many_arguments)]
#[cfg(feature = "full")]
pub fn assert_raw<T: Details>(
    condition: bool,
    message: String,
    details: &T,
    class: String,
    function: String,
    file: String,
    begin_line: u32,
    begin_column: u32,
    hit: bool,
    must_hit: bool,
    assert_type: AssertType,
    display_type: String,
    id: String,
) {
    // Only this thin shim is generic (and so compiled into the caller's
    // crate); the tracker lookup and everything downstream are erased
    // behind `dyn Details` and compiled once, here.
    let info = raw_tracker_entry(&id);
    assert_impl(
        assert_type,
        display_type.as_str(),
        condition,
        message.as_str(),
        class.as_str(),
        function.as_str(),
        file.as_str(),
        begin_line,
        begin_column,
        hit,
        must_hit,
        id.as_str(),
        details,
        Some(&*info),
    )
}

// The per-id tracking entries of raw assertions (the macros hold theirs in
// a static per call site).
#[cfg(feature = "full")]
fn raw_tracker_entry(id: &str) -> Arc<TrackingInfo> {
    static ASSERT_TRACKER: Lazy<RwLock<HashMap<String, Arc<TrackingInfo>>>> =
        Lazy::new(|| RwLock::new(HashMap::new()));

    // The read guard must drop before the write lock is taken.
    let existing = ASSERT_TRACKER.read().unwrap().get(id).cloned();
    match existing {
        Some(info) => info,
        None => ASSERT_TRACKER
            .write()
            .unwrap()
            .entry(id.to_owned())
            .or_default()
            .clone(),
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(not(feature = "full"))]
pub fn assert_raw<T: Details>(
    condition: bool,
    message: String,
    details: &T,
    class: String,
    function: String,
    file: String,
    begin_line: u32,
    begin_column: u32,
    hit: bool,
    must_hit: bool,
    assert_type: AssertType,
    display_type: String,
    id: String,
) {
    assert_impl(
        assert_type,
        display_type.as_str(),
        condition,
        message.as_str(),
        class.as_str(),
        function.as_str(),
        file.as_str(),
        begin_line,
        begin_column,
        hit,
        must_hit,
        id.as_str(),
        details,
        None,
    )
}

#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub fn assert_impl<'a>(
    assert_type: AssertType,
    display_type: &'a str,
    condition: bool,
    message: &'a str,
    class: &'a str,
    function: &'a str,
    file: &'a str,
    begin_line: u32,
    begin_column: u32,
    hit: bool,
    must_hit: bool,
    id: &'a str,
    details: &dyn Details,
    info: Option<&TrackingInfo>,
) {
    let assertion = AssertionInfo::new(
        assert_type,
        display_type,
        condition,
        message,
        class,
        function,
        file,
        begin_line,
        begin_column,
        hit,
        must_hit,
        id,
        details,
    );

    let _ = &assertion.track_entry(info);
}
