use crate::internal;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize, Debug)]
struct AntithesisSetupData<'a, 'b> {
    status: &'a str,
    details: &'b Value,
}

#[derive(Serialize, Debug)]
struct SetupCompleteData<'a> {
    antithesis_setup: AntithesisSetupData<'a, 'a>,
}

/// Indicates to Antithesis that setup has completed. Call this function when your system and workload are fully initialized.
/// After this function is called, Antithesis will take a snapshot of your system and begin [injecting faults]( https://antithesis.com/docs/applications/reliability/fault_injection.html).
///
/// Calling this function multiple times or from multiple processes will have no effect.
/// Antithesis will treat the first time any process called this function as the moment that the setup was completed.
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
pub fn setup_complete(details: &Value) {
    let status = "complete";
    let antithesis_setup = AntithesisSetupData::<'_, '_> { status, details };

    let setup_complete_data = SetupCompleteData { antithesis_setup };

    internal::dispatch_output(&setup_complete_data)
}

/// Indicates to Antithesis that a certain event has been reached. It sends a structured log message to Antithesis that you may later use to aid debugging.
///
/// In addition to ``details``, you also provide ``name``, which is the name of the event that you are logging.
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
pub fn send_event(name: &str, details: &Value) {
    let trimmed_name = name.trim();
    let owned_name: String = if trimmed_name.is_empty() {
        "anonymous".to_owned()
    } else {
        trimmed_name.to_owned()
    };
    let json_event = json!({ owned_name: details });
    internal::dispatch_output(&json_event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_complete_without_details() {
        eprintln!("setup_complete");
        let details: Value = json!({});
        setup_complete(&details);
    }

    #[test]
    fn setup_complete_with_details() {
        let details: Value = json!({
            "name": "Meow Cat",
            "age": 11,
            "phones": [
                "+1 2126581356",
                "+1 2126581384"
            ]
        });
        setup_complete(&details);
    }

    #[test]
    fn send_event_without_details() {
        let details: Value = json!({});
        send_event("my event", &details);
    }

    #[test]
    fn send_event_with_details() {
        let details: Value = json!({
            "name": "Tweety Bird",
            "age": 4,
            "phones": [
                "+1 9734970340"
            ]
        });
        send_event("my event 2", &details);
    }

    #[test]
    fn send_event_unnamed_without_details() {
        let details: Value = json!({});
        send_event("", &details);
    }

    #[test]
    fn send_event_unnamed_with_details() {
        let details: Value = json!({
            "color": "red"
        });
        send_event("   ", &details);
    }
}
