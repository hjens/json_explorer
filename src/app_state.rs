use std::cmp::min;
use std::iter::zip;

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
    pub line_number: usize,
    pub collapsed: bool,
    pub visible: bool,
    pub breadcrumbs: String,
    pub selection_level: Option<usize>,
}

impl JsonItem {
    pub fn new(name: Option<String>, indent: usize, value: JsonValueType, breadcrumbs: String) -> JsonItem {
        JsonItem {
            name,
            indent,
            value,
            line_number: 0,
            collapsed: false,
            visible: true,
            breadcrumbs,
            selection_level: None,
        }
    }

    fn indent_spans(&self) -> Vec<Span> {
        let mut output = vec![];
        for i in 0..self.indent {
            if i < 1 {
                output.push(Span::raw("   "));
            } else if Some(i) == self.selection_level {
                output.push(Span::styled("│   ", Style::default().fg(Color::Cyan)));
            } else {
                output.push(Span::styled("│   ", Style::default().fg(Color::DarkGray)));
            }
        }
        output
    }


    pub fn display_text(&self, item_index: i32, selection_index: i32, terminal_height: i32) -> Line {
        if (item_index - selection_index).abs() > terminal_height {
            // TODO: fixa när vissa är kollapsade
            return Line::from("-");
        }

        let line_number = Span::styled(format!("{:4} ", self.line_number), Style::default().fg(Color::DarkGray));
        let indents = self.indent_spans();
        let name_str = match &self.name {
            Some(name) => format!("{}: ", name),
            None => "".to_string()
        };
        let name_span = Span::styled(name_str.clone(), Style::default().fg(Color::Yellow));
        let name_value = match &self.value {
            JsonValueType::Number(num) => {
                let value_span = Span::styled(format!("{}", num), Style::default().fg(Color::Red));
                vec![name_span, value_span]
            }
            JsonValueType::String(s) => {
                let value_span = Span::styled(format!("\"{}\"", s), Style::default().fg(Color::Blue));
                vec![name_span, value_span]
            }
            JsonValueType::Bool(b) => {
                let value_span = Span::styled(format!("{}", b), Style::default().fg(Color::Green));
                vec![name_span, value_span]
            }
            JsonValueType::Array => {
                if self.collapsed {
                    let brackets_span = Span::from("[...]");
                    vec![name_span, brackets_span]
                } else {
                    let brackets_span = Span::from("[");
                    vec![name_span, brackets_span]
                }
            }
            JsonValueType::ArrayEnd => {
                let brackets_span = Span::from("]");
                vec![brackets_span]
            }
            JsonValueType::Object => {
                if self.collapsed {
                    let brackets_span = Span::from("{...}");
                    vec![name_span, brackets_span]
                } else {
                    let brackets_span = Span::from("{");
                    vec![name_span, brackets_span]
                }
            }
            JsonValueType::ObjectEnd => {
                let brackets_span = Span::from("}");
                vec![brackets_span]
            }
            JsonValueType::Null => {
                let value_span = Span::styled("null", Style::default().fg(Color::Gray));
                vec![name_span, value_span]
            }
        };
        Line::from([vec![line_number], indents, name_value].concat())
    }
}

pub struct AppState {
    pub list_state: ListState,
    pub items: Vec<JsonItem>,
    pub filename: String,
}

impl AppState {
    pub fn new(items: Vec<JsonItem>, filename: String) -> AppState {
        AppState {
            list_state: ListState::default(),
            items,
            filename,
        }
    }

    pub fn scroll_position(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
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

    pub fn select_next(&mut self, step: usize) {
        let new_index = match self.list_state.selected() {
            None => {
                0
            }
            Some(index) => {
                min(index + step, self.visible_items().len() - 1)
            }
        };
        self.list_state.select(Some(new_index));
        self.recalculate_selection_level();
    }

    pub fn select_previous(&mut self, step: usize) {
        let new_index = match self.list_state.selected() {
            None => {
                0
            }
            Some(index) => {
                if index > step {
                    index - step
                } else {
                    0
                }
            }
        };
        self.list_state.select(Some(new_index));
        self.recalculate_selection_level();
    }

    pub fn select_top(&mut self) {
        self.list_state.select(Some(0));
        self.recalculate_selection_level();
    }

    pub fn select_bottom(&mut self) {
        self.list_state.select(Some(self.visible_items().len() - 1));
        self.recalculate_selection_level();
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

    pub fn selection_index(&self) -> Option<usize> {
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

    fn recalculate_selection_level(&mut self) {
        if let Some(index) = self.selection_index() {
            // For non-containers, strip away the last component of the breadcrumbs
            let selection_breadcrumbs = match self.items[index].value {
                JsonValueType::Number(_) | JsonValueType::Bool(_) | JsonValueType::String(_) | JsonValueType::Null => {
                    match self.items[index].breadcrumbs.rsplit_once(" ▶ ") {
                        Some((val, _)) => val.to_string(),
                        None => "".to_string()
                    }
                }
                _ => self.items[index].breadcrumbs.clone()
            };
            let mut selection_level: usize;
            // Loop through all items and calculate selection level
            for item in self.items.iter_mut() {
                if item.breadcrumbs.starts_with(&selection_breadcrumbs) {
                    selection_level = 0;
                    // How many components of the breadcrumbs match?
                    for (p1, p2) in zip(selection_breadcrumbs.split(" ▶ "), item.breadcrumbs.split(" ▶ ")) {
                        if p1 == p2 {
                            selection_level += 1;
                        } else {
                            break;
                        }
                    }
                    item.selection_level = Some(selection_level);
                } else {
                    item.selection_level = None;
                }
            }
        }
    }
}