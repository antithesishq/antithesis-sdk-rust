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

#[cfg(feature = "full")]
use std::collections::HashMap;
#[cfg(feature = "full")]
use std::sync::Mutex;

use self::guidance::GuidanceCatalogInfo;

mod macros;
#[doc(hidden)]
pub mod guidance;

/// Catalog of all antithesis assertions provided
#[doc(hidden)]
#[distributed_slice]
#[cfg(feature = "full")]
pub static ANTITHESIS_CATALOG: [AssertionCatalogInfo];

/// Catalog of all antithesis guidances provided
#[doc(hidden)]
#[distributed_slice]
#[cfg(feature = "full")]
pub static ANTITHESIS_GUIDANCE_CATALOG: [GuidanceCatalogInfo];

// Only need an ASSET_TRACKER if there are actually assertions 'hit'
// (i.e. encountered and invoked at runtime).
//
// Typically runtime assertions use the macros ``always!``, ``sometimes!``, etc.
// or, a client is using the 'raw' interface ``assert_raw`` at runtime.
//
#[cfg(feature = "full")]
pub(crate) static ASSERT_TRACKER: Lazy<Mutex<HashMap<String, TrackingInfo>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(feature = "full")]
pub(crate) static INIT_CATALOG: Lazy<()> = Lazy::new(|| {
    let no_details: Value = json!({});
    for info in ANTITHESIS_CATALOG.iter() {
        let f_name: &str = info.function.as_ref();
        assert_impl(
            info.assert_type,
            info.display_type.to_owned(),
            info.condition,
            info.message.to_owned(),
            info.class.to_owned(),
            f_name.to_owned(),
            info.file.to_owned(),
            info.begin_line,
            info.begin_column,
            false, /* hit */
            info.must_hit,
            info.id.to_owned(),
            &no_details,
        );
    }
    for info in ANTITHESIS_GUIDANCE_CATALOG.iter() {
        guidance::guidance_impl(
            info.guidance_type,
            info.message.to_owned(),
            info.id.to_owned(),
            info.class.to_owned(),
            Lazy::force(info.function).to_string(),
            info.file.to_owned(),
            info.begin_line,
            info.begin_column,
            info.maximize,
            json!(null),
            false,
        )
    }
});

#[cfg(feature = "full")]
pub(crate) struct TrackingInfo {
    pub pass_count: u64,
    pub fail_count: u64,
}

#[cfg(feature = "full")]
impl Default for TrackingInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "full")]
impl TrackingInfo {
    pub fn new() -> Self {
        TrackingInfo {
            pass_count: 0,
            fail_count: 0,
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
struct AntithesisLocationInfo {
    class: String,
    function: String,
    file: String,
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
struct AssertionInfo {
    assert_type: AssertType,
    display_type: String,
    condition: bool,
    message: String,
    location: AntithesisLocationInfo,
    hit: bool,
    must_hit: bool,
    id: String,
    details: Value,
}

impl AssertionInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assert_type: AssertType,
        display_type: String,
        condition: bool,
        message: String,
        class: String,
        function: String,
        file: String,
        begin_line: u32,
        begin_column: u32,
        hit: bool,
        must_hit: bool,
        id: String,
        details: &Value,
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
            details: details.clone(),
        }
    }
} 

#[cfg(feature = "full")]
impl AssertionInfo {
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

    // Verify that the TrackingInfo for self in
    // ASSERT_TRACKER has been updated according to self.condition
    fn track_entry(&self) {
        // Requirement: Catalog entries must always will emit()
        if !self.hit {
            self.emit();
            return;
        }

        // Establish TrackingInfo for this trackingKey when needed
        let mut tracker = ASSERT_TRACKER.lock().unwrap();
        let info = tracker.entry(self.id.clone()).or_default();
        // Record the condition in the associated TrackingInfo entry,
        // and emit the assertion when first seeing a condition
        let emitting = if self.condition {
            info.pass_count += 1;
            info.pass_count == 1
        } else {
            info.fail_count += 1;
            info.fail_count == 1
        };
        drop(tracker); // release the lock asap
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
impl AssertionInfo {
    fn track_entry(&self) {
        return
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
/// use of the [Fallback SDK](https://antithesis.com/docs/using_antithesis/sdk/fallback/assert.html)
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
    )
}

#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub fn assert_impl(
    assert_type: AssertType,
    display_type: String,
    condition: bool,
    message: String,
    class: String,
    function: String,
    file: String,
    begin_line: u32,
    begin_column: u32,
    hit: bool,
    must_hit: bool,
    id: String,
    details: &Value,
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

    let _ = &assertion.track_entry();
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
        assert_eq!(ti.pass_count, 0);
        assert_eq!(ti.fail_count, 0);
    }

    #[test]
    fn default_tracking_info() {
        let ti: TrackingInfo = Default::default();
        assert_eq!(ti.pass_count, 0);
        assert_eq!(ti.fail_count, 0);
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
            this_display_type.to_owned(),
            this_condition,
            this_message.to_owned(),
            this_class.to_owned(),
            this_function.to_owned(),
            this_file.to_owned(),
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id.to_owned(),
            &this_details,
        );
        assert_eq!(ai.display_type.as_str(), this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message.as_str(), this_message);
        assert_eq!(ai.location.class.as_str(), this_class);
        assert_eq!(ai.location.function.as_str(), this_function);
        assert_eq!(ai.location.file.as_str(), this_file);
        assert_eq!(ai.location.begin_line, this_begin_line);
        assert_eq!(ai.location.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id.as_str(), this_id);
        assert_eq!(ai.details, this_details);
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
            this_display_type.to_owned(),
            this_condition,
            this_message.to_owned(),
            this_class.to_owned(),
            this_function.to_owned(),
            this_file.to_owned(),
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id.to_owned(),
            &this_details,
        );
        assert_eq!(ai.display_type.as_str(), this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message.as_str(), this_message);
        assert_eq!(ai.location.class.as_str(), this_class);
        assert_eq!(ai.location.function.as_str(), this_function);
        assert_eq!(ai.location.file.as_str(), this_file);
        assert_eq!(ai.location.begin_line, this_begin_line);
        assert_eq!(ai.location.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id.as_str(), this_id);
        assert_eq!(ai.details, this_details);
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
            this_display_type.to_owned(),
            this_condition,
            this_message.to_owned(),
            this_class.to_owned(),
            this_function.to_owned(),
            this_file.to_owned(),
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id.to_owned(),
            &this_details,
        );
        assert_eq!(ai.display_type.as_str(), this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message.as_str(), this_message);
        assert_eq!(ai.location.class.as_str(), this_class);
        assert_eq!(ai.location.function.as_str(), this_function);
        assert_eq!(ai.location.file.as_str(), this_file);
        assert_eq!(ai.location.begin_line, this_begin_line);
        assert_eq!(ai.location.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id.as_str(), this_id);
        assert_eq!(ai.details, this_details);
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

        let before_tracker = tracking_info_for_key(this_id);

        assert_impl(
            this_assert_type,
            this_display_type.to_owned(),
            this_condition,
            this_message.to_owned(),
            this_class.to_owned(),
            this_function.to_owned(),
            this_file.to_owned(),
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id.to_owned(),
            &this_details,
        );

        let after_tracker = tracking_info_for_key(this_id);

        if this_condition {
            assert_eq!(before_tracker.pass_count + 1, after_tracker.pass_count);
            assert_eq!(before_tracker.fail_count, after_tracker.fail_count);
        } else {
            assert_eq!(before_tracker.fail_count + 1, after_tracker.fail_count);
            assert_eq!(before_tracker.pass_count, after_tracker.pass_count);
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

        let before_tracker = tracking_info_for_key(this_id);

        assert_impl(
            this_assert_type,
            this_display_type.to_owned(),
            this_condition,
            this_message.to_owned(),
            this_class.to_owned(),
            this_function.to_owned(),
            this_file.to_owned(),
            this_begin_line,
            this_begin_column,
            this_hit,
            this_must_hit,
            this_id.to_owned(),
            &this_details,
        );

        let after_tracker = tracking_info_for_key(this_id);

        if this_condition {
            assert_eq!(before_tracker.pass_count + 1, after_tracker.pass_count);
            assert_eq!(before_tracker.fail_count, after_tracker.fail_count);
        } else {
            assert_eq!(before_tracker.fail_count + 1, after_tracker.fail_count);
            assert_eq!(before_tracker.pass_count, after_tracker.pass_count);
        };
    }

    fn tracking_info_for_key(key: &str) -> TrackingInfo {
        // Establish TrackingInfo for this trackingKey when needed
        let mut tracking_data = TrackingInfo::new();

        let tracking_key: String = key.to_owned();
        match ASSERT_TRACKER.lock().unwrap().get(&tracking_key) {
            None => tracking_data,
            Some(ti) => {
                tracking_data.pass_count = ti.pass_count;
                tracking_data.fail_count = ti.fail_count;
                tracking_data
            }
        }
    }
}
