use std::cmp::min;
use std::iter::zip;

use crossterm::event::Event;
use ratatui::{prelude::*, widgets::*};
use serde_json::value::Number;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use crate::app_state::SearchState::{BrowsingSearch, NotSearching, Searching};

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
        if search_string.is_empty() || !is_searching {
            self.value_is_search_result = false;
            self.name_is_search_result = false;
        } else {
            self.name_is_search_result = self.name.clone().unwrap_or("".to_string()).contains(search_string);
            self.value_is_search_result = match &self.value {
                JsonValueType::Number(n) => n.to_string().contains(search_string),
                JsonValueType::String(s) => s.contains(search_string),
                JsonValueType::Bool(b) => b.to_string().contains(search_string),
                _ => false
            };
        }
    }
}

#[derive(PartialEq)]
pub enum SearchState {
    NotSearching,
    Searching,
    BrowsingSearch(Option<usize>),
}

pub struct AppState {
    pub list_state: ListState,
    pub items: Vec<JsonItem>,
    pub filename: String,
    pub list_height: u16,
    pub search_state: SearchState,
    pub search_input: Input,
}

impl AppState {
    pub fn new(items: Vec<JsonItem>, filename: String) -> AppState {
        AppState {
            list_state: ListState::default(),
            items,
            filename,
            list_height: 0,
            search_state: SearchState::NotSearching,
            search_input: Input::new("".to_string()),
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

    pub fn select_top_of_screen(&mut self) {
        self.list_state.select(Some(self.list_state.offset()));
        self.recalculate_selection_level();
    }

    pub fn select_middle_of_screen(&mut self) {
        let top = self.list_state.offset() as u16;
        let num_items = self.visible_items().len() as u16;
        let bottom = min(top + num_items - 1, top + self.list_height - 2);
        let index = (top + bottom) / 2;
        self.list_state.select(Some((index) as usize));
        self.recalculate_selection_level();
    }

    pub fn select_bottom_of_screen(&mut self) {
        let top = self.list_state.offset() as u16;
        let num_items = self.visible_items().len() as u16;
        let index = min(top + num_items - 1, top + self.list_height - 2);
        self.list_state.select(Some(index as usize));
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

    pub fn uncollapse_all(&mut self) {
        for item in self.items.iter_mut() {
            item.collapsed = false;
        }
        self.recalculate_visible();
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

    pub fn start_searching(&mut self) {
        self.uncollapse_all();
        self.search_state = Searching;
        self.update_search_results();
    }

    pub fn cancel_searching(&mut self) {
        self.search_state = NotSearching;
        self.update_search_results();
    }
    pub fn finish_searching(&mut self) {
        self.search_state = match self.search_results().first() {
            Some(_) => BrowsingSearch(Some(0)),
            None => NotSearching
        };
    }

    fn search_results(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_index, item)| item.name_is_search_result || item.value_is_search_result)
            .map(|(index, _item)| index)
            .collect()
    }

    pub fn update_search(&mut self, event: &Event) {
        self.search_input.handle_event(event);
        self.update_search_results();
    }

    fn update_search_results(&mut self) {
        for item in self.items.iter_mut() {
            item.update_is_search_result(self.search_input.value(), self.search_state != NotSearching);
        }
        match self.search_state {
            Searching => {
                let search_results = self.search_results();
                if !search_results.is_empty() {
                    let new_index = 0;
                    self.list_state.select(Some(search_results[new_index]));
                    self.recalculate_selection_level();
                }
            }
            _ => {}
        }
    }

    pub fn search_text(&self) -> &str {
        self.search_input.value()
    }

    pub fn next_search_result(&mut self) {
        match self.search_state {
            BrowsingSearch(Some(index)) => {
                let search_results = self.search_results();
                let new_index = (index + 1) % search_results.len();
                self.list_state.select(Some(search_results[new_index]));
                self.recalculate_selection_level();
                self.search_state = BrowsingSearch(Some(new_index));
            }
            _ => {}
        }
    }

    pub fn previous_search_result(&mut self) {
        match self.search_state {
            BrowsingSearch(Some(index)) => {
                let search_results = self.search_results();
                let new_index = match index {0 => search_results.len() - 1, _=> index - 1};
                self.list_state.select(Some(search_results[new_index]));
                self.recalculate_selection_level();
                self.search_state = BrowsingSearch(Some(new_index));
            }
            _ => {}
        }
    }
}