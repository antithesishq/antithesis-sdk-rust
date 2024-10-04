#![allow(dead_code)]
pub mod env;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;

#[derive(Deserialize, Debug)]
pub struct RustLanguage {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct AntithesisSdk {
    pub language: RustLanguage,
    pub protocol_version: String,
    #[allow(dead_code)]
    pub sdk_version: String,
}

#[derive(Deserialize, Debug)]
pub struct AntithesisSetup {
    pub status: String,
    pub details: Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Location {
    pub begin_column: i32,
    pub begin_line: i32,
    pub class: String,
    pub file: String,
    pub function: String,
}

#[derive(Deserialize, Debug)]
pub struct AntithesisAssert {
    pub assert_type: AssertType,
    pub condition: bool,
    pub display_type: String,
    pub hit: bool,
    pub must_hit: bool,
    pub id: String,
    pub message: String,
    pub location: Location,
    pub details: Value,
}

#[derive(Deserialize, Debug)]
pub struct AntithesisGuidance {
    pub guidance_type: GuidanceType,
    pub message: String,
    pub id: String,
    pub location: Location,
    pub maximize: bool,
    pub guidance_data: Value,
    pub hit: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SDKInput {
    AntithesisSdk(AntithesisSdk),
    AntithesisAssert(AntithesisAssert),
    AntithesisGuidance(AntithesisGuidance),
    AntithesisSetup(AntithesisSetup),
    SendEvent { event_name: String, details: Value },
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssertType {
    Always,
    Sometimes,
    Reachability,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GuidanceType {
    Numeric,
    Boolean,
    Json,
}

fn parse_lines(lines: Vec<&str>) -> Result<Vec<SDKInput>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();

    let default_event = "abc".to_owned();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let parsed: SDKInput = match serde_json::from_str(line) {
            Ok(x) => x,
            Err(_e) => {
                // println!("{}", line);
                // println!("PARSING: {:?}", e);
                let temp: Value = serde_json::from_str(line)?;
                // should be Object(Map<String, Value>)
                // in this case the Map has just one entry (top-level name used by SendEvent())
                match temp {
                    Value::Object(user_data) => {
                        // let mut result = None;
                        let mut res = Some(SDKInput::SendEvent {
                            event_name: default_event.clone(),
                            details: json!({}),
                        });
                        if let Some((event_name, details)) = user_data.into_iter().next() {
                            res = Some(SDKInput::SendEvent {
                                event_name,
                                details,
                            });
                        }
                        match res {
                            Some(x) => x,
                            None => SDKInput::SendEvent {
                                event_name: default_event.clone(),
                                details: json!({}),
                            },
                        }
                    }
                    _ => SDKInput::SendEvent {
                        event_name: default_event.clone(),
                        details: json!({}),
                    },
                }
            }
        };
        result.push(parsed);
    }
    Ok(result)
}

pub fn read_jsonl_tags(jsonl_file: &str) -> Result<Vec<SDKInput>, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(jsonl_file)?;
    // .expect("Should have been able to read the file");

    let lines = contents.split('\n');
    let parsed = parse_lines(lines.collect())?;
    Ok(parsed)
}
