use ratatui::widgets::ListState;
use serde_json::value::Number;

pub enum JsonValueType {
    Number(Number),
    String(String),
    Bool(bool),
    Array,
    ArrayEnd,
    Object,
    ObjectEnd,
    Null,
}

pub struct JsonItem {
    pub name: Option<String>,
    pub indent: usize,
    pub value: JsonValueType,
    pub collapsed: bool,
    pub visible: bool,
}

impl JsonItem {
    pub fn new(name: Option<String>, indent: usize, value: JsonValueType) -> JsonItem {
        JsonItem {
            name,
            indent,
            value,
            collapsed: false,
            visible: true,
        }
    }

    pub fn display_text(&self) -> String {
        let indent = " │  ".repeat(self.indent);
        let name_str = match &self.name {
            Some(name) => format!("{}: ", name),
            None => "".to_string()
        };
        match &self.value {
            JsonValueType::Number(num) => {
                format!("{} {} {}", indent, name_str, num)
            }
            JsonValueType::String(s) => {
                format!("{} {} \"{}\"", indent, name_str, s)
            }
            JsonValueType::Bool(b) => {
                format!("{} {} {}", indent, name_str, b)
            }
            JsonValueType::Array => {
                if self.collapsed {
                    format!("{} {} [...]", indent, name_str)
                } else {
                    format!("{} {} [", indent, name_str)
                }
            }
            JsonValueType::ArrayEnd => {
                format!("{} ]", indent)
            }
            JsonValueType::Object => {
                if self.collapsed {
                    format!("{} {} {{...}}", indent, name_str)
                } else {
                    format!("{} {} {{", indent, name_str)
                }
            }
            JsonValueType::ObjectEnd => {
                format!("{} }}", indent)
            }
            JsonValueType::Null => {
                format!("{} {} null", indent, name_str)
            }
        }
    }
}

pub struct AppState {
    pub list_state: ListState,
    pub items: Vec<JsonItem>,
}

impl AppState {
    pub fn new(items: Vec<JsonItem>) -> AppState {
        AppState {
            list_state: ListState::default(),
            items: items,
        }
    }

    pub fn visible_items(&self) -> Vec<&JsonItem> {
        self.items
            .iter()
            .filter(|i| i.visible)
            .collect()
    }

    pub fn select_next(&mut self) {
        let new_index = match self.list_state.selected() {
            None => {
                0
            }
            Some(index) => {
                (index + 1) % self.visible_items().len()
            }
        };
        self.list_state.select(Some(new_index));
    }

    pub fn select_previous(&mut self) {
        let new_index = match self.list_state.selected() {
            None => {
                0
            }
            Some(index) => {
                if index == 0 {
                    self.visible_items().len() - 1
                } else {
                    index - 1
                }
            }
        };
        self.list_state.select(Some(new_index));
    }

    pub fn toggle_collapsed(&mut self) {
        if let Some(index) = self.list_state.selected() {
            match &self.items[index].value {
                JsonValueType::Array | JsonValueType::Object => {
                    self.items[index].collapsed = !self.items[index].collapsed;
                    self.recalculate_visible();
                }
                _ => {}
            }
        }
    }

    fn recalculate_visible(&mut self) {
        let mut is_in_collapsed = false;
        let mut collapse_indent = 0;
        for item in self.items.iter_mut() {
            item.visible = true;
            if is_in_collapsed {
                if item.indent == collapse_indent {
                    is_in_collapsed = false;
                    continue;
                }
                item.visible = false;
            } else if item.collapsed {
                is_in_collapsed = true;
                collapse_indent = item.indent;
            }
        }
    }
}