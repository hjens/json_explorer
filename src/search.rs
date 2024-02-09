use crate::json_item::JsonItem;
use core::slice::IterMut;

pub fn update_search_results(json_items: IterMut<JsonItem>, search_string: &str) {
    let mut search_components = search_string.split("=");
    let name_search_str = search_components.next();
    let value_search_str = search_components.next();

    for item in json_items {
        item.name_is_search_result = search_in_name(&item.name, name_search_str);
        item.value_is_search_result = search_in_value(&item.value_str, value_search_str);
    }
}

fn search_in_name(name: &Option<String>, search_str: Option<&str>) -> bool {
    match (name, search_str) {
        (Some(n), Some(s)) => n.to_lowercase().contains(&s.to_lowercase()) && !s.is_empty(),
        _ => false,
    }
}

fn search_in_value(value: &String, search_str: Option<&str>) -> bool {
    match search_str {
        Some("") => false,
        Some("*") => !value.is_empty(),
        Some(s) => value.to_lowercase().contains(&s.to_lowercase()) && !value.is_empty(),
        None => false,
    }
}
