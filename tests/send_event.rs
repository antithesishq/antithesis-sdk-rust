use antithesis_sdk::{lifecycle, LOCAL_OUTPUT};
use serde_json::json;

mod common;
use common::SDKInput;

// Expected output in /tmp/antithesis-send-event.json
// Note: Actual version info in antithesis_sdk can vary
//
// {
//   "antithesis_sdk": {
//     "language": {
//       "name": "Rust",
//       "version": "1.69.0"
//     },
//     "sdk_version": "0.1.2",
//     "protocol_version": "1.0.0"
//   }
// }
// {
//   "logging": {
//     "tag": "last value",
//     "x": 100
//   }
// }

#[test]
fn send_event() {
    let output_file = "/tmp/antithesis-send-event.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    let details = json!({
        "x": 100,
        "tag": "last value"
    });

    // only added to force the antithesis_sdk info to be generated
    lifecycle::send_event("logging", &details);

    // verify the output has landed in the expected file
    match common::read_jsonl_tags(output_file) {
        Ok(x) => {
            for obj in x.iter() {
                if let SDKInput::SendEvent {
                    event_name,
                    details,
                } = obj
                {
                    assert_eq!(event_name, "logging");
                    assert_eq!(&details["x"], 100);
                    assert_eq!(&details["tag"], "last value");
                }
            }
        }
        Err(e) => println!("{}", e),
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}
