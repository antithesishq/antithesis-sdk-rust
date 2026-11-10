#![cfg(feature = "full")]

use antithesis_sdk::{antithesis_init, assert_always_greater_than, LOCAL_OUTPUT};
use serde_json::{json, Value};

mod common;
use common::SDKInput;

use crate::common::{AntithesisGuidance, GuidanceType};

#[test]
fn assert_guidance() {
    let output_path = std::env::temp_dir().join(format!(
        "antithesis-assert-guidance-{}.jsonl",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&output_path);
    let output_file = output_path.to_str().unwrap();
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    antithesis_init();

    for i in 0..10 {
        let x = if i % 2 == 0 { i } else { -i };
        assert_always_greater_than!(x, 0, "Positive x", &json!({"x": x}));
    }

    match common::read_jsonl_tags(output_file) {
        Ok(x) => {
            let mut did_register = false;
            // The `left` of every hit, in order: greater_than minimizes the
            // gap, so a hit is emitted only when x - 0 falls below the
            // watermark so far.
            let mut hits = Vec::new();
            for obj in x.iter() {
                if let SDKInput::AntithesisGuidance(AntithesisGuidance {
                    guidance_type,
                    hit,
                    id,
                    message,
                    location,
                    guidance_data,
                    ..
                }) = obj
                {
                    if *hit {
                        assert_eq!(guidance_data["right"], json!(0));
                        hits.push(guidance_data["left"].clone());
                    } else {
                        assert!(guidance_data.is_null(), "a declaration carries no data");
                        did_register = true;
                    };
                    assert_eq!(*guidance_type, GuidanceType::Numeric);
                    assert_eq!(message, "Positive x");
                    assert_eq!(id, message);
                    assert!(location.begin_line > 0);
                    assert!(location.begin_column >= 0);
                    assert_eq!(location.class, "assert_guidance");
                    assert!(location.function.ends_with("::assert_guidance"));
                    assert!(location.file.ends_with("tests/assert_guidance.rs"));
                }
                println!("{:?}", obj);
            }
            assert!(did_register);
            let expected: Vec<Value> = [0, -1, -3, -5, -7, -9].iter().map(|x| json!(x)).collect();
            assert_eq!(hits, expected);
        }
        Err(e) => panic!("{}", e),
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
    std::fs::remove_file(output_path).unwrap();
}
