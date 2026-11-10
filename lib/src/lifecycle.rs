#[cfg(feature = "full")]
use crate::{details, internal};
use serde::Serialize;
#[cfg(feature = "full")]
use serde_json::{json, Map, Value};

#[cfg(feature = "full")]
#[derive(Serialize, Debug)]
struct AntithesisSetupData<'a> {
    status: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Map<String, Value>>,
}

#[cfg(feature = "full")]
#[derive(Serialize, Debug)]
struct SetupCompleteData<'a> {
    antithesis_setup: AntithesisSetupData<'a>,
}

/// Indicates to Antithesis that setup has completed. Call this function when
/// your system and workload are fully initialized. After this function is
/// called, Antithesis will take a snapshot of your system and begin
/// [injecting faults]( https://antithesis.com/docs/environment/fault_injection/).
///
/// Calling this function multiple times or from multiple processes will have no
/// effect. Antithesis will treat the first time any process called this
/// function as the moment that the setup was completed.
///
/// Details may be any borrowed value implementing `Serialize`. Objects are
/// preserved; other non-null JSON values are wrapped under `"value"`, for
/// example `3` becomes `{"value": 3}`. Pass `&()` or `&Value::Null` for no
/// details; an explicit empty object is preserved. Serialization errors emit an
/// `antithesis_error` message and omit details; the setup event is still
/// emitted. With the `full` feature disabled, details are not serialized.
///
/// # Example
///
/// ```
/// use serde_json::{json, Value};
/// use antithesis_sdk::lifecycle;
///
/// let (num_nodes, main_id) = (10, "n-001");
///
/// let startup_data: Value = json!({
///     "num_nodes": num_nodes,
///     "main_node_id": main_id,
/// });
///
/// lifecycle::setup_complete(&startup_data);
/// ```
pub fn setup_complete<T: Serialize + ?Sized>(details: &T) {
    #[cfg(feature = "full")]
    {
        let antithesis_setup = AntithesisSetupData {
            status: "complete",
            details: details::object(details, "setup_complete"),
        };
        internal::dispatch_output(&SetupCompleteData { antithesis_setup });
    }
    #[cfg(not(feature = "full"))]
    let _ = details;
}

/// Indicates to Antithesis that a certain event has been reached. It sends a
/// structured log message to Antithesis that you may later use to aid
/// debugging.
///
/// In addition to ``details``, you also provide ``name``, which is the name of
/// the event that you are logging.
///
/// Details may be any borrowed value implementing `Serialize`. Objects are
/// preserved; other non-null JSON values are wrapped under `"value"`, for
/// example `3` becomes `{"value": 3}`. Null and serialization errors produce an
/// empty event body `{}`. Serialization errors also emit an `antithesis_error`
/// message. With the `full` feature disabled, details are not serialized.
///
/// # Example
///
/// ```
/// use serde_json::{json, Value};
/// use antithesis_sdk::lifecycle;
///
/// let info_value: Value = json!({
///     "month": "July",
///     "day": 17
/// });
///
/// lifecycle::send_event("start_day", &info_value);
/// ```
pub fn send_event<T: Serialize + ?Sized>(name: &str, details: &T) {
    // The name is passed through verbatim, like every other SDK: renaming
    // or trimming here would make the same program emit different events
    // depending on which SDK it was written against.
    #[cfg(feature = "full")]
    {
        let json_event = json!({ name: details::object(details, name).unwrap_or_default() });
        internal::dispatch_output(&json_event);
    }
    #[cfg(not(feature = "full"))]
    let _ = (name, details);
}
