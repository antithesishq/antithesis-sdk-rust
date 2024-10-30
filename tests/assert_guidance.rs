use antithesis_sdk::{antithesis_init, assert_always_greater_than, LOCAL_OUTPUT};
use serde_json::json;

mod common;
use common::SDKInput;

use crate::common::{AntithesisGuidance, GuidanceType};

#[test]
fn assert_guidance() {
    let output_file = "/tmp/antithesis-assert-guidance.json";
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    antithesis_init();

    for i in 0..10 {
        let x = if i % 2 == 0 { i } else { -i };
        assert_always_greater_than!(x, 0, "Positive x", &json!({"x": x}));
    }

    match common::read_jsonl_tags(output_file) {
        Ok(x) => {
            let mut did_register = false;
            let mut did_hit = false;
            for obj in x.iter() {
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
                    };
                    assert_eq!(*guidance_type, GuidanceType::Numeric);
                    assert_eq!(message, "Positive x");
                    assert_eq!(id, message);
                    assert!(location.begin_line > 0);
                    assert!(location.begin_column >= 0);
                    assert_eq!(location.class, "assert_guidance");
                    assert!(location.function.ends_with("::assert_guidance"));
                    assert!(location
                        .file
                        .ends_with("/tests/assert_guidance.rs"));
                }
                println!("{:?}", obj);
            }
            assert!(did_register);
            assert!(did_hit);
        }
        Err(e) => println!("{}", e),
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
}
