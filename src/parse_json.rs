use std::fs;

use serde_json::{Result, Value};

use crate::app_state::{JsonItem, JsonValueType};

fn parse_json(root_value: &Value, output: &mut Vec<JsonItem>, title: Option<String>, indent: usize, breadcrumbs: String) {
    match root_value {
        Value::Object(map) => {
            output.push(JsonItem::new(title, indent, JsonValueType::Object, breadcrumbs.clone()));
            for (key, value) in map {
                parse_json(value, output, Some(key.to_string()), indent + 1, make_breadcrumbs(&breadcrumbs, key));
            }
            output.push(JsonItem::new(None, indent, JsonValueType::ObjectEnd, breadcrumbs.clone()));
        }
        Value::Array(arr) => {
            output.push(JsonItem::new(title.clone(), indent, JsonValueType::Array, breadcrumbs.clone()));
            for (index, value) in arr.iter().enumerate() {
                parse_json(value, output, None, indent + 1, make_breadcrumbs(&breadcrumbs, &index.to_string()));
            }
            output.push(JsonItem::new(None, indent, JsonValueType::ArrayEnd, breadcrumbs.clone()));
        }
        Value::Number(n) => {
            output.push(JsonItem::new(title, indent, JsonValueType::Number(n.clone()), breadcrumbs.clone()));
        }
        Value::Bool(b) => {
            output.push(JsonItem::new(title, indent, JsonValueType::Bool(*b), breadcrumbs.clone()));
        }
        Value::String(s) => {
            output.push(JsonItem::new(title, indent, JsonValueType::String(s.clone()), breadcrumbs.clone()));
        }
        Value::Null => {
            output.push(JsonItem::new(title, indent, JsonValueType::Null, breadcrumbs.clone()));
        }
    }
}

fn make_breadcrumbs(root: &str, new: &str) -> String {
    match root {
        "" => new.to_string(),
        _ => format!("{} ▶ {}", root, new)
    }
}

pub fn parse_json_file() -> Result<Vec<JsonItem>> {
    let text = fs::read_to_string("/Users/hannes/Documents/PycharmProjects/json_explorer/sample.json").expect("File must exist");
    let root_value: Value = serde_json::from_str(&text)?;

    let mut json_vec = Vec::new();
    parse_json(&root_value, &mut json_vec, None, 0, "".to_string());
    Ok(json_vec)
}

