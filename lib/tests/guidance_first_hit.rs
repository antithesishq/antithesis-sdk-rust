#![cfg(feature = "full")]

//! The first hit for a guidance id must emit whatever its gap is.

use antithesis_sdk::{
    antithesis_init, assert_always_greater_than, assert_always_less_than, LOCAL_OUTPUT,
};

mod common;
use crate::common::AntithesisGuidance;
use common::SDKInput;

#[test]
fn first_hit_emits_even_when_its_gap_overflows() {
    let output_path = std::env::temp_dir().join(format!(
        "antithesis-guidance-first-hit-{}.jsonl", std::process::id()
    ));
    let _ = std::fs::remove_file(&output_path);
    let output_file = output_path.to_str().unwrap();
    let prev_v = common::env::set_var(LOCAL_OUTPUT, output_file);
    antithesis_init();

    // always_greater_than minimizes, so its watermark was seeded +infinity,
    // and this gap is +infinity.
    assert_always_greater_than!(f64::MAX, -f64::MAX, "overflowing gap, minimizing");
    // always_less_than maximizes, seeded -infinity, and this gap is -infinity.
    assert_always_less_than!(-f64::MAX, f64::MAX, "overflowing gap, maximizing");

    let events = common::read_jsonl_tags(output_file).expect("reading the emitted events");
    for message in ["overflowing gap, minimizing", "overflowing gap, maximizing"] {
        let hits = events
            .iter()
            .filter(|obj| {
                matches!(
                    obj,
                    SDKInput::AntithesisGuidance(AntithesisGuidance { hit: true, message: m, .. })
                        if m == message
                )
            })
            .count();
        assert_eq!(hits, 1, "{message:?} should have emitted its first hit exactly once");
    }
    common::env::restore_var(LOCAL_OUTPUT, prev_v);
    std::fs::remove_file(output_path).unwrap();
}
