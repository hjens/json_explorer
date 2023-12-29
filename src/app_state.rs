use ratatui::{prelude::*, widgets::*};
use serde_json::value::Number;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct JsonItem {
    pub name: Option<String>,
    pub indent: usize,
    pub value: JsonValueType,
    pub collapsed: bool,
    pub visible: bool,
    pub breadcrumbs: String,
}

impl JsonItem {
    pub fn new(name: Option<String>, indent: usize, value: JsonValueType, breadcrumbs: String) -> JsonItem {
        JsonItem {
            name,
            indent,
            value,
            collapsed: false,
            visible: true,
            breadcrumbs
        }
    }


    pub fn display_text(&self) -> Line {
        let indent = "│  ".repeat(self.indent);
        let indent_span = Span::styled(indent.clone(), Style::default().fg(Color::DarkGray));
        let name_str = match &self.name {
            Some(name) => format!("{}: ", name),
            None => "".to_string()
        };
        let name_span = Span::styled(name_str.clone(), Style::default().fg(Color::Yellow));
        match &self.value {
            JsonValueType::Number(num) => {
                let value_span = Span::styled(format!("{}", num), Style::default().fg(Color::Red));
                Line::from(vec![indent_span, name_span, value_span])
            }
            JsonValueType::String(s) => {
                let value_span = Span::styled(format!("\"{}\"", s), Style::default().fg(Color::Blue));
                Line::from(vec![indent_span, name_span, value_span])
            }
            JsonValueType::Bool(b) => {
                let value_span = Span::styled(format!("{}", b), Style::default().fg(Color::Green));
                Line::from(vec![indent_span, name_span, value_span])
            }
            JsonValueType::Array => {
                if self.collapsed {
                    let brackets_span = Span::from(format!("[...]"));
                    Line::from(vec![indent_span, name_span, brackets_span])
                } else {
                    let brackets_span = Span::from(format!("["));
                    Line::from(vec![indent_span, name_span, brackets_span])
                }
            }
            JsonValueType::ArrayEnd => {
                let brackets_span = Span::from(format!("]"));
                Line::from(vec![indent_span, brackets_span])
            }
            JsonValueType::Object => {
                if self.collapsed {
                    let brackets_span = Span::from(format!("{{...}}"));
                    Line::from(vec![indent_span, name_span, brackets_span])
                } else {
                    let brackets_span = Span::from(format!("{{"));
                    Line::from(vec![indent_span, name_span, brackets_span])
                }
            }
            JsonValueType::ObjectEnd => {
                let brackets_span = Span::from(format!("}}"));
                Line::from(vec![indent_span, brackets_span])
            }
            JsonValueType::Null => {
                let value_span = Span::styled("null", Style::default().fg(Color::Gray));
                Line::from(vec![indent_span, name_span, value_span])
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
            items,
        }
    }

    pub fn breadbrumbs_text(&self) -> String {
        match self.selection_index() {
            Some(index) => self.items[index].breadcrumbs.clone(),
            None => "".to_string()
        }
    }

    pub fn visible_items(&self) -> Vec<JsonItem> {
        // TODO: cache this
        self.items
            .iter()
            .filter(|i| i.visible)
            .cloned()
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
        if let Some(index) = self.selection_index() {
            match &self.items[index].value {
                JsonValueType::Array | JsonValueType::Object => {
                    self.items[index].collapsed = !self.items[index].collapsed;
                    self.recalculate_visible();
                }
                _ => {}
            }
        }
    }

   fn visible_indices(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_index, value)| value.visible)
            .map(|(index, _value)| index)
            .collect()
    }

    fn selection_index(&self) -> Option<usize> {
        self.list_state
            .selected()
            .map(|index| self.visible_indices()[index])
    }

    fn recalculate_visible(&mut self) {
        let mut is_in_collapsed = false;
        let mut collapse_indent = 0;
        for item in self.items.iter_mut() {
            item.visible = true;
            if is_in_collapsed {
                if item.indent == collapse_indent {
                    is_in_collapsed = false;
                    item.visible = false;  // closing bracket
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