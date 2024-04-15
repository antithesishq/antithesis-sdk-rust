pub mod env;


use std::fs;
use serde::{ Deserialize, Serialize };
use serde_json::{json, Value};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct RustLanguage {
    pub name: String, 
    pub version: String
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct AntithesisSdk {
    pub language: RustLanguage, 
    pub protocol_version: String,
    pub sdk_version: String 
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct AntithesisSetup {
    pub status: String,
    pub details: Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
pub struct Location {
    begin_column: i32,
    begin_line: i32,
    class: String,
    file: String,
    function: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct AntithesisAssert {
    assert_type: AssertType,
    condition: bool,
    display_type: String,
    hit: bool,
    must_hit: bool,
    id: String,
    message: String,
    location: Location,
    details: Value,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SDKInput {
    AntithesisSdk(AntithesisSdk),
    AntithesisAssert(AntithesisAssert),
    AntithesisSetup(AntithesisSetup),

    #[allow(dead_code)]
    SendEvent{event_name: String, details: Value }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum AssertType {
    Always,
    Sometimes,
    Reachability,
}

fn parse_lines(lines: Vec<&str>) -> Result<Vec<SDKInput>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();

   let default_event = "abc".to_owned(); 
    for line in lines {
        if line.len() < 1 { continue; }
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
                       let mut res = Some(SDKInput::SendEvent{event_name: default_event.clone(), details: json!({})});
                       for (event_name, details) in user_data {
                            res = Some(SDKInput::SendEvent{
                                event_name,
                                details,
                            });
                            break;
                       }; 
                       match res {
                           Some(x) => x,
                           None => SDKInput::SendEvent{event_name: default_event.clone(), details: json!({})}
                       }
                    },
                    _ => SDKInput::SendEvent{event_name: default_event.clone(), details: json!({})}
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
    
    let lines = contents.split("\n");
    let parsed = parse_lines(lines.collect())?;
    Ok(parsed) 
}
