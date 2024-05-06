use once_cell::sync::Lazy;
use serde_json::{Value, json};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Mutex};
use crate::internal;
use linkme::distributed_slice;

mod macros;


/// Catalog of all antithesis assertions provided
#[distributed_slice]
pub static ANTITHESIS_CATALOG: [CatalogInfo];

// Only need an ASSET_TRACKER if there are actually assertions 'hit' 
// (i.e. encountered and invoked at runtime).
//
// Typically runtime assertions use the macros always!(), sometimes!(), etc.
// or, a client is using the 'raw' interface 'assert_raw' at runtime.
//
pub(crate) static ASSERT_TRACKER: Lazy<Mutex<HashMap<String, TrackingInfo>>> = 
   Lazy::new(|| Mutex::new(HashMap::new()));

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
            &no_details
        );
    }
});

pub(crate) struct TrackingInfo {
    pub pass_count: u64,
    pub fail_count: u64,
}

impl Default for TrackingInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingInfo {
    pub fn new() -> Self {
        TrackingInfo{
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
#[derive(Debug)]
pub struct CatalogInfo {
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
        details: &Value) -> Self {

        let location = AntithesisLocationInfo {
            class,
            function,
            file,
            begin_line,
            begin_column,
        };

        AssertionInfo{
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
            return
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
        internal::dispatch_output(&self);
    }
}


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
        id: String) {

    assert_impl( assert_type, display_type, condition, message, class, function, file, begin_line, begin_column, hit, must_hit, id, details)
}

/// This is a low-level method designed to be used by third-party frameworks. 
/// Regular users of the assert package should not call it.
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
        details: &Value) {

    let assertion = AssertionInfo::new(assert_type, display_type, condition, message, class, function, file, begin_line, begin_column, hit, must_hit, id, details);
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
            &this_details);
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
            &this_details);
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
            &this_details);
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
            &this_details);

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
            &this_details);

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
