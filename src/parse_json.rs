use serde_json::{Result, Value};

use crate::app_state::{JsonItem, JsonValueType};

fn parse_json(root_value: &Value, output: &mut Vec<JsonItem>, title: Option<String>, indent: usize, breadcrumbs: String) {
    match root_value {
        Value::Object(map) => {
            output.push(JsonItem::new(title, indent, JsonValueType::Object, breadcrumbs.clone()));
            for (key, value) in map {
                parse_json(value, output, Some(key.to_string()), indent + 1, make_breadcrumbs(&breadcrumbs, key, JsonValueType::Object));
            }
            output.push(JsonItem::new(None, indent, JsonValueType::ObjectEnd, breadcrumbs.clone()));
        }
        Value::Array(arr) => {
            output.push(JsonItem::new(title.clone(), indent, JsonValueType::Array, breadcrumbs.clone()));
            for (index, value) in arr.iter().enumerate() {
                parse_json(value, output, None, indent + 1, make_breadcrumbs(&breadcrumbs, &index.to_string(), JsonValueType::Array));
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

fn make_breadcrumbs(root: &str, new: &str, value_type: JsonValueType) -> String {
    match root {
        "" => new.to_string(),
        _ => match value_type {
            JsonValueType::Array => format!("{} ▶ [{}]", root, new),
            JsonValueType::Object => format!("{} ▶ {}", root, new),
            _ => format!("{} ▶ {}", root, new),
        }
    }
}

pub fn parse_json_string(json_string: &str) -> Result<Vec<JsonItem>> {
    let root_value: Value = serde_json::from_str(json_string)?;

    let mut json_vec = Vec::new();
    parse_json(&root_value, &mut json_vec, None, 0, "".to_string());
    for (index, item) in json_vec.iter_mut().enumerate() {
        item.line_number = index;
    }
    Ok(json_vec)
}

