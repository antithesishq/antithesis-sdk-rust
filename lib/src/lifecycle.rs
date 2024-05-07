//! The lifecycle module contains functions which inform the Antithesis
//! environment that particular test phases or milestones have been reached.

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

/// Call this function when your system and workload are fully initialized.
/// After this function is called, the Antithesis environment will take a
/// snapshot of your system and begin [injecting faults].
///
/// Calling this function multiple times, or from multiple processes, will
/// have no effect. Antithesis will treat the first time any process called
/// this function as the moment that the setup was completed.
pub fn setup_complete(details: &Value) {
    let status = "complete";
    let antithesis_setup = AntithesisSetupData::<'_, '_> { status, details };

    let setup_complete_data = SetupCompleteData { antithesis_setup };

    internal::dispatch_output(&setup_complete_data)
}

/// Causes an event with the name and details provided,
/// to be sent to the Fuzzer and Notebook
pub fn send_event(name: &str, details: &Value) {
    let trimmed_name = name.trim();
    let owned_name: String = if trimmed_name.is_empty() {
        "anonymous".to_owned()
    } else {
        trimmed_name.to_owned()
    };
    let json_event = json!({
        owned_name: details
    });
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
