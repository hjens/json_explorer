use ratatui::prelude::*;
use serde_json::Number;

use crate::theme::THEME;

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
    pub value_str: String,
    pub line_number: usize,
    pub collapsed: bool,
    pub visible: bool,
    pub breadcrumbs: String,
    pub selection_level: Option<usize>,
    pub name_is_search_result: bool,
    pub value_is_search_result: bool,
    pub len: usize,
}

impl JsonItem {
    pub fn new(
        name: Option<String>,
        indent: usize,
        value: JsonValueType,
        breadcrumbs: String,
        len: usize,
    ) -> JsonItem {
        let value_str = match &value {
            JsonValueType::Number(n) => n.to_string(),
            JsonValueType::String(s) => s.to_string(),
            JsonValueType::Bool(b) => b.to_string(),
            _ => "".to_string(),
        };
        JsonItem {
            name,
            indent,
            value,
            value_str,
            line_number: 0,
            collapsed: false,
            visible: true,
            breadcrumbs,
            selection_level: None,
            name_is_search_result: false,
            value_is_search_result: false,
            len,
        }
    }

    fn indent_spans(&self) -> Vec<Span> {
        let mut output = vec![];
        for i in 0..self.indent {
            if Some(i) == self.selection_level {
                output.push(Span::styled(
                    "  │ ",
                    Style::default().fg(THEME.selection_level_indicator_color),
                ));
            } else {
                output.push(Span::styled(
                    "  │ ",
                    Style::default().fg(THEME.indent_color),
                ));
            }
        }
        output
    }

    pub fn display_text(&self, selection_index: Option<usize>) -> Line {
        let line_number = Span::styled(
            format!("{:8} ", self.line_number),
            Style::default().fg(Color::DarkGray),
        );
        let selection_span = if selection_index == Some(self.line_number) {
            Span::styled("▶ ", Style::default().fg(THEME.selection_indicator_color))
        } else {
            Span::raw("  ")
        };
        let indents = self.indent_spans();

        let name_str = match &self.name {
            Some(name) => format!("{}: ", name),
            None => "".to_string(),
        };
        let name_span = Span::styled(
            name_str.clone(),
            Style::default()
                .fg(THEME.name_color)
                .bg(match self.name_is_search_result {
                    true => THEME.search_indicator_color,
                    false => Color::default(),
                }),
        );
        let value_bg = match self.value_is_search_result {
            true => THEME.search_indicator_color,
            false => Color::default(),
        };
        let name_value = match &self.value {
            JsonValueType::Number(num) => {
                let value_span = Span::styled(
                    format!("{}", num),
                    Style::default().fg(THEME.number_color).bg(value_bg),
                );
                vec![name_span, value_span]
            }
            JsonValueType::String(s) => {
                let value_span = Span::styled(
                    format!("\"{}\"", s),
                    Style::default().fg(THEME.string_color).bg(value_bg),
                );
                vec![name_span, value_span]
            }
            JsonValueType::Bool(b) => {
                let value_span = Span::styled(
                    format!("{}", b),
                    Style::default().fg(THEME.bool_color).bg(value_bg),
                );
                vec![name_span, value_span]
            }
            JsonValueType::Array => {
                if self.collapsed {
                    vec![
                        name_span,
                        Span::from("["),
                        Span::styled(
                            format!("{} items", self.len),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::from("]"),
                    ]
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
                    vec![
                        name_span,
                        Span::from("{"),
                        Span::styled(
                            format!("{} items", self.len),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::from("}"),
                    ]
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
                let value_span = Span::styled("null", Style::default().fg(THEME.null_color));
                vec![name_span, value_span]
            }
        };
        Line::from([vec![line_number], indents, vec![selection_span], name_value].concat())
    }
}
