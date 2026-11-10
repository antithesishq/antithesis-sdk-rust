#![cfg(feature = "full")]

use antithesis_sdk::{
    assert_always, assert_always_greater_than, assert_always_some, details::WithGuidance,
};
use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use std::{cell::Cell, collections::BTreeMap};

struct Counted<'a, T> {
    calls: &'a Cell<usize>,
    value: T,
}

impl<T: Serialize> Serialize for Counted<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.calls.set(self.calls.get() + 1);
        self.value.serialize(serializer)
    }
}

#[derive(Serialize)]
struct Shadowed<'a> {
    left: Counted<'a, f64>,
    right: Counted<'a, u64>,
    kept: Counted<'a, u32>,
}

#[derive(Serialize)]
enum Variant {
    #[serde(rename = "left")]
    Left(f64),
    Inner {
        left: f64,
    },
}

#[test]
fn diagnostics_describe_only_the_emitted_details() {
    let path = std::env::temp_dir().join(format!(
        "antithesis-details-diagnostics-{}.jsonl",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    std::env::set_var(antithesis_sdk::LOCAL_OUTPUT, &path);

    let calls = Cell::new(0);
    let skipped_calls = Cell::new(0);
    let kept_calls = Cell::new(0);
    let shadowed = Counted {
        calls: &calls,
        value: Shadowed {
            left: Counted {
                calls: &skipped_calls,
                value: f64::NAN,
            },
            right: Counted {
                calls: &skipped_calls,
                value: 1 << 60,
            },
            kept: Counted {
                calls: &kept_calls,
                value: 7,
            },
        },
    };
    assert_always_greater_than!(2, 1, "shadowed struct", &shadowed);
    assert_eq!(calls.get(), 1);
    assert_eq!(skipped_calls.get(), 0);
    assert_eq!(kept_calls.get(), 1);
    let map = BTreeMap::from([("left", f64::INFINITY), ("right", f64::NAN)]);
    assert_always_greater_than!(2, 1, "shadowed map", &Some(map));
    assert_always_greater_than!(2, 1, "shadowed variant", &Variant::Left(f64::NAN));
    assert_always_some!({value: true}, "shadowed scalar", &f64::NAN);
    assert_always_some!({value: true}, "shadowed sequence", &vec![1u64 << 60]);

    // Only now, with both once-per-process warnings still unclaimed, has the
    // silence above shown that shadowed values are never inspected.
    assert_always_greater_than!(f64::INFINITY, f64::NEG_INFINITY, "infinite operands");
    assert_always_greater_than!(f32::NAN, 0.0f32, "nan operand");
    assert_always_greater_than!(2, 1, "retained variant", &Variant::Inner { left: f64::NAN });
    assert_always_greater_than!(2, 1, "retained scalar", &f64::INFINITY);
    assert_always_greater_than!(2, 1, "retained sequence", &vec![f64::NEG_INFINITY]);

    // Numeric map keys must retain serde_json's string conversion, including
    // when that string matches a generated key.
    let numeric_keys = BTreeMap::from([(1, f64::NAN), (2, f64::INFINITY)]);
    let details = WithGuidance {
        details: &numeric_keys,
        guidance: || json!({"1": true}),
    };
    assert_always!(true, "numeric keys", &details);

    let events: Vec<Value> = std::fs::read_to_string(&path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        !events
            .iter()
            .any(|event| event.get("antithesis_error").is_some()),
        "{events:?}"
    );
    // The shadowed cases run while both once-per-process warnings are still
    // unclaimed, so their silence shows the shadowed values were never
    // inspected. "infinite operands" then claims both, 6002 for its replaced
    // details and 6004 for its skipped guidance, and nothing after it warns.
    for (name, expected, expected_warnings) in [
        (
            "shadowed struct",
            json!({"left": 2, "right": 1, "kept": 7}),
            vec![],
        ),
        ("shadowed map", json!({"left": 2, "right": 1}), vec![]),
        ("shadowed variant", json!({"left": 2, "right": 1}), vec![]),
        ("shadowed scalar", json!({"value": true}), vec![]),
        ("shadowed sequence", json!({"value": true}), vec![]),
        (
            "infinite operands",
            json!({"left": "Infinity", "right": "-Infinity"}),
            vec![6002, 6004],
        ),
        ("nan operand", json!({"left": "NaN", "right": 0.0}), vec![]),
        (
            "retained variant",
            json!({"left": 2, "right": 1, "Inner": {"left": "NaN"}}),
            vec![],
        ),
        (
            "retained scalar",
            json!({"left": 2, "right": 1, "value": "Infinity"}),
            vec![],
        ),
        (
            "retained sequence",
            json!({"left": 2, "right": 1, "value": ["-Infinity"]}),
            vec![],
        ),
        ("numeric keys", json!({"1": true, "2": "Infinity"}), vec![]),
    ] {
        let hits: Vec<_> = events
            .iter()
            .filter(|event| {
                event["antithesis_assert"]["id"] == name
                    && event["antithesis_assert"]["hit"] == true
            })
            .collect();
        assert_eq!(hits.len(), 1, "{name}");
        assert_eq!(hits[0]["antithesis_assert"]["details"], expected, "{name}");
        if name == "infinite operands" || name == "nan operand" {
            assert!(!events.iter().any(|event| {
                event["antithesis_guidance"]["id"] == name
                    && event["antithesis_guidance"]["hit"] == true
            }), "{name}");
        }
        let warnings: Vec<_> = events
            .iter()
            .filter_map(|event| event.get("antithesis_warning"))
            .filter(|event| event["context"] == name)
            .map(|event| event["code"].as_u64().unwrap())
            .collect();
        assert_eq!(warnings, expected_warnings, "{name}");
    }
    std::fs::remove_file(path).unwrap();
}
