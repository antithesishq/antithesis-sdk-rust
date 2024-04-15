use serde_json::{json, Value};
use crate::internal;

pub fn setup_complete(details: &Value) {
    let setup_value = json!({
        "antithesis_setup": json!({
            "status": "complete",
            "details": details
        })
    });
    internal::dispatch_output(&setup_value)
}

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
        eprintln!("setrup_complete");
        let details: Value = json!({});
        setup_complete(&details);
        assert!(true)
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
        assert!(true)
    }

    #[test]
    fn send_event_without_details() {
        let details: Value = json!({});
        send_event("my event", &details);
        assert!(true)
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
        assert!(true)
    }

    #[test]
    fn send_event_unnamed_without_details() {
        let details: Value = json!({});
        send_event("", &details);
        assert!(true)
    }

    #[test]
    fn send_event_unnamed_with_details() {
        let details: Value = json!({
            "color": "red"
        });
        send_event("   ", &details);
        assert!(true)
    }
}
