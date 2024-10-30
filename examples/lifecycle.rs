use serde_json::{json, Value};

use antithesis_sdk::prelude::*;

/// Demonstrates the lifecycle functions available in the Antithesis SDK
///
/// The lifecycle module informs the Antithesis environment that particular test phases 
/// or milestones have been reached. It provides two main functions:
///
/// - `setup_complete`: Indicates to Antithesis that setup has completed and the system/workload
///   is fully initialized. After this is called, Antithesis will take a snapshot and begin
///   injecting faults. Only the first call has any effect.
///
/// - `send_event`: Indicates that a certain event has occurred, providing greater information
///   about event ordering during testing. The event name and details will appear in triage reports.
///
/// Both functions accept optional details to provide additional context that will be logged.
fn lifecycle_demo() {
    // Create some example JSON values to pass as details
    let bird_value: Value = json!({
        "name": "Tweety Bird", 
        "age": 4,
        "phones": [
            "+1 9734970340"
        ]
    });
    let cat_value: Value = json!({
        "name": "Meow Cat",
        "age": 11, 
        "phones": [
            "+1 2126581356",
            "+1 2126581384"
        ]
    });

    let tiger: Value = json!(2457);

    // todo: Multiple setup_complete calls have no effect - only first one matters
    // I am confused? https://antithesis.com/docs/generated/sdk/golang/lifecycle/ why is it
    // being called multiple times?
    lifecycle::setup_complete(&tiger);
    lifecycle::setup_complete(&bird_value);
    lifecycle::setup_complete(&cat_value);

    // Send an event with a name and details that will appear in triage reports
    let info_value: Value = json!({
        "month": "January",
        "day": 32
    });
    lifecycle::send_event("user_info", &info_value);
}

fn main() {
    antithesis_init();
    lifecycle_demo();
}
