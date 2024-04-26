use once_cell::sync::Lazy;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Mutex, Once};
use crate::internal;
use linkme::distributed_slice;

// Needed for AssertType
use std::fmt;
use std::str::FromStr;

mod macros;



#[distributed_slice]
pub static ANTITHESIS_CATALOG: [CatalogInfo];

static ASSERT_TRACKER: Lazy<Mutex<HashMap<String, TrackingInfo>>> = Lazy::new(|| Mutex::new(HashMap::new()));

static INIT_CATALOG: Once = Once::new();

pub struct TrackingInfo {
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


#[derive(PartialEq, Debug)]
enum AssertType {
    Always,
    Sometimes,
    Reachability,
}

const DEFAULT_ASSERT_TYPE: AssertType = AssertType::Reachability;

impl FromStr for AssertType {
    type Err = ();
    fn from_str(input: &str) -> Result<AssertType, Self::Err> {
        match input {
            "always"  => Ok(AssertType::Always),
            "sometimes"  => Ok(AssertType::Sometimes),
            "reachability"  => Ok(AssertType::Reachability),
            _  => Err(()),
        }
    }
}

impl fmt::Display for AssertType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            AssertType::Always => "always",
            AssertType::Sometimes => "sometimes",
            AssertType::Reachability => "reachability"
        };
        write!(f, "{text}")
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CatalogInfo {
    pub assert_type: &'static str,
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

#[derive(Debug)]
struct AssertionInfo {
    assert_type: String,
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
    details: Value,
}

impl AssertionInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assert_type: &str,
        display_type: &str,
        condition: bool,
        message: &str,
        class: &str,
        function: &str,
        file: & str,
        begin_line: u32,
        begin_column: u32,
        hit: bool,
        must_hit: bool,
        id: &str,
        details: &Value) -> Self {

        let derived_assert_type = match AssertType::from_str(assert_type) {
            Ok(converted_assert_type) => converted_assert_type,
            Err(_) => DEFAULT_ASSERT_TYPE
        };

        let assert_type_text = derived_assert_type.to_string();

        AssertionInfo{
            assert_type: assert_type_text,
            display_type: display_type.to_owned(),
            condition,
            message: message.to_owned(),
            class: class.to_owned(),
            function: function.to_owned(),
            file: file.to_owned(),
            begin_line,
            begin_column,
            hit,
            must_hit,
            id: id.to_owned(),
            details: details.clone(),
        }
    }


    // emit(tracker_entry, assertion) and determines if the assertion 
    // should actually be emitted:
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
            if !INIT_CATALOG.is_completed(){
                antithesis_init();
            }
            self.emit();
        }
    }

    fn emit(&self) {
        let location_info = json!({
            "class": self.class.as_str(),
            "function": self.function.as_str(),
            "file": self.file.as_str(),
            "begin_line": self.begin_line,
            "begin_column": self.begin_column,
        });
        let assertion_value = json!({
            "antithesis_assert": json!({
                "hit": self.hit,
                "must_hit": self.must_hit,
                "assert_type": self.assert_type.as_str(),
                "display_type": self.display_type.as_str(),
                "message": self.message.as_str(),
                "condition": self.condition,
                "id": self.id.as_str(),
                "location": location_info,
                "details": &self.details
            })
        });
        internal::dispatch_output(&assertion_value)
    }
}


#[allow(clippy::too_many_arguments)]
pub fn assert_raw(
        condition: bool,
        message: &str,
        details: &Value,
        class: &str,
        function: &str,
        file: &str,
        begin_line: u32,
        begin_column: u32,
        hit: bool,
        must_hit: bool,
        assert_type: &str,
        display_type: &str,
        id: &str) {

    assert_impl( assert_type, display_type, condition, message, class, function, file, begin_line, begin_column, hit, must_hit, id, details)
}

/// This is a low-level method designed to be used by third-party frameworks. 
/// Regular users of the assert package should not call it.
#[allow(clippy::too_many_arguments)]
pub fn assert_impl(
        assert_type: &str,
        display_type: &str,
        condition: bool,
        message: &str,
        class: &str,
        function: &str,
        file: &str,
        begin_line: u32,
        begin_column: u32,
        hit: bool,
        must_hit: bool,
        id: &str,
        details: &Value) {

    let assertion = AssertionInfo::new(assert_type, display_type, condition, message, class, function, file, begin_line, begin_column, hit, must_hit, id, details);
    let _ = &assertion.track_entry();
}

pub fn antithesis_init() {
    INIT_CATALOG.call_once(|| {
        let no_details: Value = json!({});
        for info in ANTITHESIS_CATALOG.iter() {
            let f_name = once_cell::sync::Lazy::<&'static str>::force(info.function);
            println!("CatAlog Item ==> fn: '{}' display_type: '{}' - '{}' {}[{}]", f_name, info.display_type, info.message, info.file, info.begin_line);
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
                &no_details
            );
        }
    })
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
    // Tests for AssertType
    //--------------------------------------------------------------------------------
    #[test]
    fn text_to_assert_type() {
        let always_assert_type = AssertType::from_str("always");
        assert_eq!(always_assert_type.unwrap(), AssertType::Always);

        let sometimes_assert_type = AssertType::from_str("sometimes");
        assert_eq!(sometimes_assert_type.unwrap(), AssertType::Sometimes);

        let reachability_assert_type = AssertType::from_str("reachability");
        assert_eq!(reachability_assert_type.unwrap(), AssertType::Reachability);

        let fallback_assert_type = AssertType::from_str("xyz");
        assert!(fallback_assert_type.is_err())
    }

    #[test]
    fn assert_type_to_text() {
        let always_text = AssertType::Always.to_string();
        assert_eq!(always_text, "always");

        let sometimes_text = AssertType::Sometimes.to_string();
        assert_eq!(sometimes_text, "sometimes");

        let reachability_text = AssertType::Reachability.to_string();
        assert_eq!(reachability_text, "reachability");
    }


    //--------------------------------------------------------------------------------
    // Tests for AssertionInfo
    //--------------------------------------------------------------------------------

    #[test]
    fn new_assertion_info_always() {
        let this_assert_type = "always";
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
            &this_details);
        assert_eq!(ai.assert_type, this_assert_type);
        assert_eq!(ai.display_type, this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message, this_message);
        assert_eq!(ai.class, this_class);
        assert_eq!(ai.function, this_function);
        assert_eq!(ai.file, this_file);
        assert_eq!(ai.begin_line, this_begin_line);
        assert_eq!(ai.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id, this_id);
        assert_eq!(ai.details, this_details);
    }

    #[test]
    fn new_assertion_info_sometimes() {
        let this_assert_type = "sometimes";
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
            &this_details);
        assert_eq!(ai.assert_type, this_assert_type);
        assert_eq!(ai.display_type, this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message, this_message);
        assert_eq!(ai.class, this_class);
        assert_eq!(ai.function, this_function);
        assert_eq!(ai.file, this_file);
        assert_eq!(ai.begin_line, this_begin_line);
        assert_eq!(ai.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id, this_id);
        assert_eq!(ai.details, this_details);
    }

    #[test]
    fn new_assertion_info_reachable() {
        let this_assert_type = "reachability";
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
            &this_details);
        assert_eq!(ai.assert_type, this_assert_type);
        assert_eq!(ai.display_type, this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message, this_message);
        assert_eq!(ai.class, this_class);
        assert_eq!(ai.function, this_function);
        assert_eq!(ai.file, this_file);
        assert_eq!(ai.begin_line, this_begin_line);
        assert_eq!(ai.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id, this_id);
        assert_eq!(ai.details, this_details);
    }

    #[test]
    fn assert_impl_pass() {
        let this_assert_type = "always";
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
        let this_assert_type = "always";
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
    fn new_assertion_info_invalid_assert_type() {
        let this_assert_type = "possibly";
        let this_display_type = "Possibly";
        let this_condition = true;
        let this_message = "Possibly message";
        let this_class = "binary::possibly";
        let this_function = "binary::possibly::possibly_function";
        let this_file = "/home/user/binary/src/possibly_binary.rs";
        let this_begin_line = 13;
        let this_begin_column = 8;
        let this_hit = true;
        let this_must_hit = true;
        let this_id = "ID Possibly message";
        let this_details = json!({
            "color": "possibly red",
            "extent": 21,
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
            &this_details);

        let fallback_assert_type = "reachability";

        assert_eq!(ai.assert_type, fallback_assert_type);
        assert_eq!(ai.display_type, this_display_type);
        assert_eq!(ai.condition, this_condition);
        assert_eq!(ai.message, this_message);
        assert_eq!(ai.class, this_class);
        assert_eq!(ai.function, this_function);
        assert_eq!(ai.file, this_file);
        assert_eq!(ai.begin_line, this_begin_line);
        assert_eq!(ai.begin_column, this_begin_column);
        assert_eq!(ai.hit, this_hit);
        assert_eq!(ai.must_hit, this_must_hit);
        assert_eq!(ai.id, this_id);
        assert_eq!(ai.details, this_details);
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
