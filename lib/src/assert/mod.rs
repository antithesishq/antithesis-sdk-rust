#[cfg(feature = "full")]
use crate::internal;
#[cfg(feature = "full")]
use linkme::distributed_slice;
#[cfg(feature = "full")]
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::Value;
#[cfg(feature = "full")]
use serde_json::json;
use std::sync::atomic::AtomicU64;
#[cfg(feature = "full")]
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, atomic::Ordering},
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
    for info in ANTITHESIS_CATALOG.iter() {
        let f_name: &str = info.function.as_ref();
        assert_impl(
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
            &json!(null),
            None,
        );
    }
    for info in ANTITHESIS_GUIDANCE_CATALOG.iter() {
        guidance::guidance_impl(
            info.guidance_type,
            info.message,
            info.id,
            info.class,
            #[allow(clippy::explicit_auto_deref)]
            *Lazy::force(info.function),
            info.file,
            info.begin_line,
            info.begin_column,
            info.maximize,
            json!(null),
            false,
        )
    }
});

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

#[derive(Serialize, Debug)]
struct AssertionInfo<'a, S: Serialize> {
    assert_type: AssertType,
    display_type: &'a str,
    condition: bool,
    message: &'a str,
    location: AntithesisLocationInfo<'a>,
    hit: bool,
    must_hit: bool,
    id: &'a str,
    details: &'a S,
}

impl<'a, S: Serialize> AssertionInfo<'a, S> {
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
        details: &'a S,
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
impl<S: Serialize> AssertionInfo<'_, S> {
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
        let json_event = json!({ "antithesis_assert": &self });
        internal::dispatch_output(&json_event)
    }
}

#[cfg(not(feature = "full"))]
impl<S: Serialize> AssertionInfo<'_, S> {
    fn track_entry(&self, _info: Option<&TrackingInfo>) {
        return;
    }
}

/// This is a low-level method designed to be used by third-party frameworks.
/// Regular users of the assert package should not call it.
///
/// This is primarily intended for use by adapters from other
/// diagnostic tools that intend to output Antithesis-style
/// assertions.
///
/// Be certain to provide an assertion catalog entry
/// for each assertion issued with ``assert_raw()``.  Assertion catalog
/// entries are also created using ``assert_raw()``, by setting the value
/// of the ``hit`` parameter to false.
///
/// Please refer to the general Antithesis documentation regarding the
/// use of the [Fallback SDK](https://antithesis.com/docs/using_antithesis/sdk/fallback/assert/)
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
/// // {"assert_type":"always","display_type":"Always","condition":false,"message":"Never extra votes","location":{"class":"mycrate::stuff","function":"mycrate::tally_vote","file":"src/voting.rs","begin_line":20,"begin_column":3},"hit":false,"must_hit":true,"id":"42-1005","details":{}}
/// // {"assert_type":"always","display_type":"Always","condition":true,"message":"Never extra votes","location":{"class":"mycrate::stuff","function":"mycrate::tally_vote","file":"src/voting.rs","begin_line":20,"begin_column":3},"hit":true,"must_hit":true,"id":"42-1005","details":{"voters":1,"votes":1}}
/// // {"assert_type":"always","display_type":"Always","condition":false,"message":"Never extra votes","location":{"class":"mycrate::stuff","function":"mycrate::tally_vote","file":"src/voting.rs","begin_line":20,"begin_column":3},"hit":true,"must_hit":true,"id":"42-1005","details":{"voters":3,"votes":4}}
/// ```
#[allow(clippy::too_many_arguments)]
#[cfg(feature = "full")]
pub fn assert_raw(
    condition: bool,
    message: String,
    details: &Value,
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
    static ASSERT_TRACKER: Lazy<Mutex<HashMap<String, Arc<TrackingInfo>>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    // Establish TrackingInfo for this trackingKey when needed
    let info = {
        let mut tracker = ASSERT_TRACKER.lock().unwrap();
        if !tracker.contains_key(&id) {
            tracker.insert(id.clone(), Arc::new(TrackingInfo::default()));
        }
        tracker.get(&id).unwrap().clone()
    };

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

#[allow(clippy::too_many_arguments)]
#[cfg(not(feature = "full"))]
pub fn assert_raw(
    condition: bool,
    message: String,
    details: &Value,
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
pub fn assert_impl<'a, S: Serialize>(
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
    details: &S,
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

#[cfg(test)]
mod tests {
    use super::*;

    //--------------------------------------------------------------------------------
    // Tests for TrackingInfo
    //--------------------------------------------------------------------------------
    #[test]
    fn new_tracking_info() {
        let ti = TrackingInfo::new();
        assert_eq!(ti.pass_count.load(Ordering::SeqCst), 0);
        assert_eq!(ti.fail_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn default_tracking_info() {
        let ti: TrackingInfo = Default::default();
        assert_eq!(ti.pass_count.load(Ordering::SeqCst), 0);
        assert_eq!(ti.fail_count.load(Ordering::SeqCst), 0);
    }

    //--------------------------------------------------------------------------------
    // Tests for AssertionInfo
    //--------------------------------------------------------------------------------

    #[test]
    fn new_assertion_info_always() {
        let this_assert_type = AssertType::Always;
        let this_display_type = "Always";
        let this_condition = true;
        let this_message = "Always message";
        let this_class = "binary::always";
        let this_function = "binary::always::always_function";
        let this_file = "/home/user/binary/src/always_binary.rs";
        let this_begin_line = 10;
        let this_begin_column = 5;
        let this_hit = true;
        let this_must_hit = true;
        let this_id = "ID Always message";
        let this_details = json!({
            "color": "always red",
            "extent": 15,
        });

        let ai = AssertionInfo::new(
            this_assert_type,
            this_display_type,
            this_condition,
            this_message,
            this_class,
            this_function,
            this_file,
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id,
            &this_details,
        );
        assert_eq!(ai.display_type, this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message, this_message);
        assert_eq!(ai.location.class, this_class);
        assert_eq!(ai.location.function, this_function);
        assert_eq!(ai.location.file, this_file);
        assert_eq!(ai.location.begin_line, this_begin_line);
        assert_eq!(ai.location.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id, this_id);
        assert_eq!(ai.details, &this_details);
    }

    #[test]
    fn new_assertion_info_sometimes() {
        let this_assert_type = AssertType::Sometimes;
        let this_display_type = "Sometimes";
        let this_condition = true;
        let this_message = "Sometimes message";
        let this_class = "binary::sometimes";
        let this_function = "binary::sometimes::sometimes_function";
        let this_file = "/home/user/binary/src/sometimes_binary.rs";
        let this_begin_line = 11;
        let this_begin_column = 6;
        let this_hit = true;
        let this_must_hit = true;
        let this_id = "ID Sometimes message";
        let this_details = json!({
            "color": "sometimes red",
            "extent": 17,
        });

        let ai = AssertionInfo::new(
            this_assert_type,
            this_display_type,
            this_condition,
            this_message,
            this_class,
            this_function,
            this_file,
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id,
            &this_details,
        );
        assert_eq!(ai.display_type, this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message, this_message);
        assert_eq!(ai.location.class, this_class);
        assert_eq!(ai.location.function, this_function);
        assert_eq!(ai.location.file, this_file);
        assert_eq!(ai.location.begin_line, this_begin_line);
        assert_eq!(ai.location.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id, this_id);
        assert_eq!(ai.details, &this_details);
    }

    #[test]
    fn new_assertion_info_reachable() {
        let this_assert_type = AssertType::Reachability;
        let this_display_type = "Reachable";
        let this_condition = true;
        let this_message = "Reachable message";
        let this_class = "binary::reachable";
        let this_function = "binary::reachable::reachable_function";
        let this_file = "/home/user/binary/src/reachable_binary.rs";
        let this_begin_line = 12;
        let this_begin_column = 7;
        let this_hit = true;
        let this_must_hit = true;
        let this_id = "ID Reachable message";
        let this_details = json!({
            "color": "reachable red",
            "extent": 19,
        });

        let ai = AssertionInfo::new(
            this_assert_type,
            this_display_type,
            this_condition,
            this_message,
            this_class,
            this_function,
            this_file,
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id,
            &this_details,
        );
        assert_eq!(ai.display_type, this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message, this_message);
        assert_eq!(ai.location.class, this_class);
        assert_eq!(ai.location.function, this_function);
        assert_eq!(ai.location.file, this_file);
        assert_eq!(ai.location.begin_line, this_begin_line);
        assert_eq!(ai.location.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id, this_id);
        assert_eq!(ai.details, &this_details);
    }

    #[test]
    fn assert_impl_pass() {
        let this_assert_type = AssertType::Always;
        let this_display_type = "Always";
        let this_condition = true;
        let this_message = "Always message 2";
        let this_class = "binary::always";
        let this_function = "binary::always::always_function";
        let this_file = "/home/user/binary/src/always_binary.rs";
        let this_begin_line = 10;
        let this_begin_column = 5;
        let this_hit = true;
        let this_must_hit = true;
        let this_id = "ID Always message 2";
        let this_details = json!({
            "color": "always red",
            "extent": 15,
        });

        let tracker = TrackingInfo::new();

        let before_tracker = clone_tracker(&tracker);

        assert_impl(
            this_assert_type,
            this_display_type,
            this_condition,
            this_message,
            this_class,
            this_function,
            this_file,
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id,
            &this_details,
            Some(&tracker),
        );

        let after_tracker: TrackingInfo = clone_tracker(&tracker);

        if this_condition {
            assert_eq!(
                before_tracker.pass_count.load(Ordering::SeqCst) + 1,
                after_tracker.pass_count.load(Ordering::SeqCst)
            );
            assert_eq!(
                before_tracker.fail_count.load(Ordering::SeqCst),
                after_tracker.fail_count.load(Ordering::SeqCst)
            );
        } else {
            assert_eq!(
                before_tracker.fail_count.load(Ordering::SeqCst) + 1,
                after_tracker.fail_count.load(Ordering::SeqCst)
            );
            assert_eq!(
                before_tracker.pass_count.load(Ordering::SeqCst),
                after_tracker.pass_count.load(Ordering::SeqCst)
            );
        };
    }

    #[test]
    fn assert_impl_fail() {
        let this_assert_type = AssertType::Always;
        let this_display_type = "Always";
        let this_condition = false;
        let this_message = "Always message 3";
        let this_class = "binary::always";
        let this_function = "binary::always::always_function";
        let this_file = "/home/user/binary/src/always_binary.rs";
        let this_begin_line = 10;
        let this_begin_column = 5;
        let this_hit = true;
        let this_must_hit = true;
        let this_id = "ID Always message 3";
        let this_details = json!({
            "color": "always red",
            "extent": 15,
        });

        let tracker = TrackingInfo::new();

        let before_tracker = clone_tracker(&tracker);

        assert_impl(
            this_assert_type,
            this_display_type,
            this_condition,
            this_message,
            this_class,
            this_function,
            this_file,
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id,
            &this_details,
            Some(&tracker),
        );

        let after_tracker: TrackingInfo = clone_tracker(&tracker);

        if this_condition {
            assert_eq!(
                before_tracker.pass_count.load(Ordering::SeqCst) + 1,
                after_tracker.pass_count.load(Ordering::SeqCst)
            );
            assert_eq!(
                before_tracker.fail_count.load(Ordering::SeqCst),
                after_tracker.fail_count.load(Ordering::SeqCst)
            );
        } else {
            assert_eq!(
                before_tracker.fail_count.load(Ordering::SeqCst) + 1,
                after_tracker.fail_count.load(Ordering::SeqCst)
            );
            assert_eq!(
                before_tracker.pass_count.load(Ordering::SeqCst),
                after_tracker.pass_count.load(Ordering::SeqCst)
            );
        };
    }

    fn clone_tracker(old: &TrackingInfo) -> TrackingInfo {
        let tracking_data = TrackingInfo::new();
        tracking_data
            .pass_count
            .store(old.pass_count.load(Ordering::SeqCst), Ordering::SeqCst);
        tracking_data
            .fail_count
            .store(old.fail_count.load(Ordering::SeqCst), Ordering::SeqCst);
        tracking_data
    }
}
