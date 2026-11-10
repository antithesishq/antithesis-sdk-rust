//! Details behaviour the cross-SDK conformance harness cannot observe from
//! the outside: how often the caller's serializer runs, whether the macro
//! argument is evaluated for suppressed emissions, what an omitted details
//! argument produces, and how guidance keys merge into caller details of
//! every shape (struct, bare value, none, or a serializer that fails).

use antithesis_sdk::{assert_always, assert_always_greater_than, assert_always_some};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use std::cell::Cell;

struct Counted<'a> {
    calls: &'a Cell<usize>,
    value: Value,
}

impl Serialize for Counted<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.calls.set(self.calls.get() + 1);
        self.value.serialize(serializer)
    }
}

struct Broken;

impl Serialize for Broken {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(
            "deliberate serialization failure",
        ))
    }
}

/// A bag whose only unserializable values sit under the keys a *numeric* rich
/// assertion generates, so the merge overwrites them before anything is
/// serialized. The boolean kinds generate their named bools instead, a
/// disjoint key set, hence the second fixture.
struct ShadowedNumeric;

impl Serialize for ShadowedNumeric {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("kept", &1)?;
        map.serialize_entry("left", &f64::NAN)?;
        map.serialize_entry("right", &Broken)?;
        map.end()
    }
}

#[derive(Serialize)]
struct ShadowedWithWarnings {
    right: Broken,
    ratio: f64,
    wide: u64,
}

struct ShadowedBoolean;

impl Serialize for ShadowedBoolean {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("kept", &1)?;
        map.serialize_entry("ready", &Broken)?;
        map.end()
    }
}

/// A broken value under a key *named* like one the merge generates, but nested
/// one level down where the merge never reaches. The bag is lost, because
/// shadowing is a top-level relation — which is also how the SDKs that merge
/// maps behave.
struct NestedLookalike;

impl Serialize for NestedLookalike {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("inner", &InnerRight)?;
        map.end()
    }
}

struct InnerRight;

impl Serialize for InnerRight {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("right", &Broken)?;
        map.end()
    }
}

/// One broken value the merge overwrites and one it does not, so the bag is
/// still lost: skipping the shadowed entry is not a licence to emit a partial
/// bag.
struct PartlyShadowed;

impl Serialize for PartlyShadowed {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("right", &Broken)?;
        map.serialize_entry("survives", &Broken)?;
        map.end()
    }
}

#[derive(Serialize)]
struct Context {
    nested: Value,
    left: i32,
    right: i32,
    ready: bool,
    ratio: f64,
    wide: u64,
}

#[test]
fn details_are_lazy_optional_and_merged_with_guidance() {
    #[cfg(feature = "full")]
    let output = {
        let path =
            std::env::temp_dir().join(format!("antithesis-details-{}.jsonl", std::process::id()));
        let _ = std::fs::remove_file(&path);
        std::env::set_var(antithesis_sdk::LOCAL_OUTPUT, &path);
        path
    };

    let ordinary_calls = Cell::new(0);
    let numeric_calls = Cell::new(0);
    let boolean_calls = Cell::new(0);
    let evaluations = Cell::new(0);
    let ordinary = Counted {
        calls: &ordinary_calls,
        value: json!(42),
    };
    let numeric = Counted {
        calls: &numeric_calls,
        value: json!([1, 2]),
    };
    let boolean = Counted {
        calls: &boolean_calls,
        value: json!(false),
    };
    for condition in [true, true, false, false] {
        assert_always!(condition, "lazy ordinary", {
            evaluations.set(evaluations.get() + 1);
            &ordinary
        });
        assert_always_greater_than!(if condition { 2 } else { 0 }, 1, "lazy numeric", {
            evaluations.set(evaluations.get() + 1);
            &numeric
        });
        assert_always_some!({ready: condition}, "lazy boolean", {
            evaluations.set(evaluations.get() + 1);
            &boolean
        });
    }
    let expected_calls = if cfg!(feature = "full") { 2 } else { 0 };
    assert_eq!(ordinary_calls.get(), expected_calls);
    assert_eq!(numeric_calls.get(), expected_calls);
    assert_eq!(boolean_calls.get(), expected_calls);
    assert_eq!(evaluations.get(), 12);

    let context = Context {
        nested: json!([null, {"key": "value"}]),
        left: 99,
        right: 99,
        ready: false,
        ratio: f64::NAN,
        wide: 1 << 60,
    };
    assert_always_greater_than!(2, 1, "struct numeric", &context);
    assert_always_some!({ready: true}, "struct boolean", &context);
    assert_always!(true, "omitted details");
    assert_always_greater_than!(2, 1, "numeric scalar details", &42);
    assert_always_some!({ready: true}, "boolean array details", &vec![1, 2]);
    assert_always_greater_than!(2, 1, "numeric without details");
    assert_always_some!({ready: true}, "boolean without details");
    assert_always_greater_than!(2, 1, "numeric broken details", &Broken);
    assert_always_some!({ready: true}, "boolean broken details", &Broken);
    assert_always_greater_than!(1u64 << 60, 1, "wide operands");
    // Broken values the merge is about to overwrite, and so never serializes.
    assert_always_greater_than!(2, 1, "numeric shadowed break", &ShadowedNumeric);
    assert_always_some!({ready: true}, "boolean shadowed break", &ShadowedBoolean);
    assert_always_greater_than!(
        2,
        1,
        "shadowed with warnings",
        &ShadowedWithWarnings {
            right: Broken,
            ratio: f64::NAN,
            wide: 1u64 << 60,
        }
    );
    // ... and one that survives the merge alongside a shadowed one.
    assert_always_greater_than!(2, 1, "numeric surviving break", &PartlyShadowed);
    assert_always_greater_than!(2, 1, "numeric nested lookalike", &NestedLookalike);

    #[cfg(feature = "full")]
    {
        let events: Vec<Value> = std::fs::read_to_string(&output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let assertions: Vec<&Value> = events
            .iter()
            .filter_map(|event| event.get("antithesis_assert"))
            .collect();
        // One error per broken guidance emission, each naming the emission it
        // belongs to
        let mut errors: Vec<(u64, &str)> = events
            .iter()
            .filter_map(|event| event.get("antithesis_error"))
            .map(|error| {
                (
                    error["code"].as_u64().unwrap(),
                    error["context"].as_str().unwrap(),
                )
            })
            .collect();
        errors.sort_unstable();
        // The two "shadowed break" bags are deliberately absent: every broken
        // value in them sits under a key the merge overwrites, so none is ever
        // handed to a serializer and there is nothing to report. The surviving
        // break is here because one of its broken values is *not* overwritten.
        let expected_errors = [
            "boolean broken details",
            "numeric broken details",
            "numeric nested lookalike",
            "numeric surviving break",
        ]
        .map(|name| (6001, name));
        assert_eq!(errors, expected_errors);
        // The caller's details and the guidance keys are inspected together;
        // each warning is reported once per process, by the first emission
        // that earns it, and failures consume neither.
        let warnings = |name: &str| {
            events
                .iter()
                .filter_map(|event| event.get("antithesis_warning"))
                .filter(|warning| warning["context"] == name)
                .map(|warning| warning["code"].as_u64().unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(warnings("struct numeric"), [6002, 6003]);
        assert!(warnings("struct boolean").is_empty());
        assert!(warnings("wide operands").is_empty());
        assert!(warnings("shadowed with warnings").is_empty());
        assert!(warnings("numeric shadowed break").is_empty());
        assert!(warnings("numeric broken details").is_empty());
        assert!(warnings("numeric scalar details").is_empty());
        let hits = |name: &str| {
            assertions
                .iter()
                .copied()
                .filter(|event| event["message"] == name && event["hit"] == true)
                .collect::<Vec<_>>()
        };
        let omitted = hits("omitted details");
        assert_eq!(omitted.len(), 1);
        assert!(omitted[0].get("details").is_none(), "{}", omitted[0]);
        assert_eq!(
            hits("numeric scalar details")[0]["details"],
            json!({"value": 42, "left": 2, "right": 1})
        );
        assert_eq!(
            hits("boolean array details")[0]["details"],
            json!({"value": [1, 2], "ready": true})
        );
        let numeric = &hits("struct numeric")[0]["details"];
        assert_eq!(numeric["left"], 2);
        assert_eq!(numeric["right"], 1);
        assert_eq!(numeric["nested"], context.nested);
        assert_eq!(numeric["ratio"], "NaN");
        assert_eq!(numeric["wide"], 1u64 << 60);
        let boolean = &hits("struct boolean")[0]["details"];
        assert_eq!(boolean["ready"], true);
        assert_eq!(boolean["nested"], context.nested);
        assert_eq!(
            hits("wide operands")[0]["details"],
            json!({"left": 1u64 << 60, "right": 1})
        );
        assert_eq!(
            hits("numeric without details")[0]["details"],
            json!({"left": 2, "right": 1})
        );
        assert_eq!(
            hits("boolean without details")[0]["details"],
            json!({"ready": true})
        );
        for name in ["numeric broken details", "boolean broken details"] {
            let hit = &hits(name)[0];
            assert!(hit.get("details").is_none(), "{hit}");
        }
        // A value the generated keys overwrite is never handed to a
        // serializer, so it cannot fail the bag — which is what the SDKs that
        // merge maps before serializing do for free, and what this one needs
        // `shadow::SkipShadowed` to reproduce. The surviving keys are emitted
        // and no antithesis_error is reported for these.
        assert_eq!(
            hits("numeric shadowed break")[0]["details"],
            json!({"kept": 1, "left": 2, "right": 1})
        );
        assert_eq!(
            hits("boolean shadowed break")[0]["details"],
            json!({"kept": 1, "ready": true})
        );
        assert_eq!(
            hits("shadowed with warnings")[0]["details"],
            json!({
                "left": 2, "right": 1, "ratio": "NaN", "wide": 1u64 << 60,
            })
        );
        // But a broken value the merge does *not* overwrite still loses the
        // whole bag: skipping the shadowed one is not partial emission.
        let surviving = &hits("numeric surviving break")[0];
        assert!(surviving.get("details").is_none(), "{surviving}");
        // Shadowing is a top-level relation: a "right" one level down is
        // ordinary data the merge never touches, so it still loses the bag.
        let nested = &hits("numeric nested lookalike")[0];
        assert!(nested.get("details").is_none(), "{nested}");
        std::fs::remove_file(output).unwrap();
    }
}
