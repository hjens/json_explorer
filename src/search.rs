use crate::json_item::JsonItem;
use core::slice::IterMut;

pub fn update_search_results(json_items: IterMut<JsonItem>, search_string: &str) {
    let mut search_components = search_string.split("=");
    let name_search_str = search_components.next();
    let value_search_str = search_components.next();

    let name_parts = if name_search_str.unwrap_or("").contains(".") {
        name_search_str.unwrap().rsplit_once(".").unwrap()
    } else {
        ("", name_search_str.unwrap_or(""))
    };

    for item in json_items {
        item.name_is_search_result = search_in_name(&item.name, &item.breadcrumbs, name_parts);
        item.value_is_search_result = search_in_value(&item.value_str, value_search_str);

        // name_search_str != "" && value_search_str != "": only match if both are search results
        if not_empty(name_search_str) && not_empty(value_search_str) {
            if !(item.name_is_search_result && item.value_is_search_result) {
                item.name_is_search_result = false;
                item.value_is_search_result = false;
            }
        }
    }
}

fn search_in_name(name: &Option<String>, breadcrumbs: &str, name_parts: (&str, &str)) -> bool {
    match (name, name_parts) {
        (Some(n), ("", s)) => n.to_lowercase().contains(&s.to_lowercase()) && !s.is_empty(),
        (Some(n), (bs, ns)) => {
            n.to_lowercase().contains(&ns.to_lowercase())
                && breadcrumbs.contains(&bs.to_lowercase())
        }
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

fn not_empty(s: Option<&str>) -> bool {
    match s {
        None => false,
        Some(st) => !st.is_empty(),
    }
}
