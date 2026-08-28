use antithesis_sdk::{antithesis_init, assert_always_greater_than, LOCAL_OUTPUT};
use serde_json::json;

mod common;
use common::{AntithesisGuidance, GuidanceType, SDKInput};

// A non-literal but const-evaluable `&'static str`
const MESSAGE: &str = concat!("Positive ", "x");

#[test]
fn assert_guidance_const_message() {
    let output_file = "/tmp/antithesis-assert-guidance-const-message.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    antithesis_init();

    for i in 0..10 {
        let x = if i % 2 == 0 { i } else { -i };
        assert_always_greater_than!(x, 0, MESSAGE, &json!({"x": x}));
    }

    let mut did_register = false; // guidance catalog registration entry (hit == false)
    let mut did_hit = false; // runtime guidance emission (hit == true)
    match common::read_jsonl_tags(output_file) {
        Ok(entries) => {
            for obj in entries.iter() {
                if let SDKInput::AntithesisGuidance(AntithesisGuidance {
                    guidance_type,
                    hit,
                    id,
                    message,
                    location,
                    ..
                }) = obj
                {
                    if *hit {
                        did_hit = true;
                    } else {
                        did_register = true;
                    }
                    assert_eq!(*guidance_type, GuidanceType::Numeric);
                    assert_eq!(message, MESSAGE);
                    assert_eq!(id, MESSAGE);
                    assert_eq!(location.class, "assert_guidance_const_message");
                    assert!(location.function.ends_with("::assert_guidance_const_message"));
                }
            }
        }
        Err(e) => panic!("could not read SDK output: {e}"),
    }

    assert!(did_register, "guidance catalog registration entry (hit=false) missing");
    assert!(did_hit, "runtime guidance emission (hit=true) missing");

    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}
