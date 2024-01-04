use ratatui::{prelude::*};
use serde_json::Number;

#[derive(Clone, PartialEq)]
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
    pub name_is_search_result: bool,
    pub value_is_search_result: bool,
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
            name_is_search_result: false,
            value_is_search_result: false,
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
            return Line::from("-");
        }

        let line_number = Span::styled(format!("{:4} ", self.line_number), Style::default().fg(Color::DarkGray));
        let indents = self.indent_spans();
        let name_str = match &self.name {
            Some(name) => format!("{}: ", name),
            None => "".to_string()
        };
        let name_span = Span::styled(name_str.clone(), Style::default().fg(Color::Yellow).bg(
            match self.name_is_search_result {
                true => Color::LightCyan,
                false => Color::default()
            }
        ));
        let value_bg = match self.value_is_search_result {
            true => Color::LightCyan,
            false => Color::default()
        };
        let name_value = match &self.value {
            JsonValueType::Number(num) => {
                let value_span = Span::styled(format!("{}", num), Style::default().fg(Color::Red).bg(value_bg));
                vec![name_span, value_span]
            }
            JsonValueType::String(s) => {
                let value_span = Span::styled(format!("\"{}\"", s), Style::default().fg(Color::Blue).bg(value_bg));
                vec![name_span, value_span]
            }
            JsonValueType::Bool(b) => {
                let value_span = Span::styled(format!("{}", b), Style::default().fg(Color::Green).bg(value_bg));
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

    pub fn update_is_search_result(&mut self, search_string: &str, is_searching: bool) {
        let is_result = |s: &str| -> bool {
            s.to_lowercase().contains(&search_string.to_lowercase())
        };
        if search_string.is_empty() || !is_searching {
            self.value_is_search_result = false;
            self.name_is_search_result = false;
        } else {
            self.name_is_search_result = is_result(&self.name.clone().unwrap_or("".to_string()));
            self.value_is_search_result = match &self.value {
                JsonValueType::Number(n) => is_result(&n.to_string()),
                JsonValueType::String(s) => is_result(s),
                JsonValueType::Bool(b) => is_result(&b.to_string()),
                _ => false
            };
        }
    }
}
