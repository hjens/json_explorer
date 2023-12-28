use std::fs;
use serde_json::{Result, Value};

use crate::app_state::{JsonItem, JsonValueType};

fn parse_json(root_value: &Value, output: &mut Vec<JsonItem>, title: Option<String>, indent: usize) {
    match root_value {
        Value::Object(map) => {
            output.push(JsonItem::new(title, indent, JsonValueType::Object));
            for (key, value) in map {
                parse_json(value, output, Some(key.to_string()), indent + 1);
            }
            output.push(JsonItem::new(None, indent, JsonValueType::ObjectEnd));
        }
        Value::Array(arr) => {
            output.push(JsonItem::new(title, indent, JsonValueType::Array));
            for value in arr {
                parse_json(value, output, None, indent + 1);
            }
            output.push(JsonItem::new(None, indent, JsonValueType::ArrayEnd));
        }
        Value::Number(n) => {
            output.push(JsonItem::new(title, indent, JsonValueType::Number(n.clone())));
        }
        Value::Bool(b) => {
            output.push(JsonItem::new(title, indent, JsonValueType::Bool(*b)));
        }
        Value::String(s) => {
            output.push(JsonItem::new(title, indent, JsonValueType::String(s.clone())));
        }
        Value::Null => {
            output.push(JsonItem::new(title, indent, JsonValueType::Null));
        }
    }
}

pub fn parse_json_file() -> Result<Vec<JsonItem>> {
    let text = fs::read_to_string("/Users/hannes/Documents/PycharmProjects/json_explorer/sample.json").expect("File must exist");
    let root_value: Value = serde_json::from_str(&text)?;

    let mut json_vec = Vec::new();
    parse_json(&root_value, &mut json_vec, None, 0);
    Ok(json_vec)
}

