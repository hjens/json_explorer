use std::cmp::min;
use std::iter::zip;

use crossterm::event::Event;
use ratatui::widgets::*;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::app_state::SearchState::{BrowsingSearch, NotSearching, Searching};
use crate::json_item::{JsonItem, JsonValueType};

#[derive(PartialEq)]
pub enum SearchState {
    NotSearching,
    Searching,
    BrowsingSearch(Option<usize>),
}

pub struct AppState {
    pub list_state: ListState,
    pub items: Vec<JsonItem>,
    pub visible_items: Vec<JsonItem>,
    pub filename: String,
    pub list_height: u16,
    pub search_state: SearchState,
    pub search_input: Input,
}

impl AppState {
    pub fn new(items: Vec<JsonItem>, filename: String) -> AppState {
        let mut app_state = AppState {
            list_state: ListState::default(),
            items: items.clone(),
            visible_items: items.clone(),
            filename,
            list_height: 0,
            search_state: NotSearching,
            search_input: Input::new("".to_string()),
        };
        app_state.select_next(1);
        app_state
    }

    pub fn status_text(&self) -> String {
        match self.search_state {
            Searching => {
                let num_results = self.search_results().len();
                format!("{} results", num_results)
            }
            BrowsingSearch(Some(index)) => {
                let num_results = self.search_results().len();
                format!("Result {} of {}", index + 1, num_results)
            }
            _ => {
                let values: Vec<&JsonItem> = self
                    .items
                    .iter()
                    .filter(|i| {
                        i.value != JsonValueType::ObjectEnd && i.value != JsonValueType::ArrayEnd
                    })
                    .collect();
                format!("{} values in file", values.len())
            }
        }
    }

    pub fn scroll_position(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    pub fn breadbrumbs_text(&self) -> String {
        match self.selection_index() {
            Some(index) => self.items[index].breadcrumbs.clone(),
            None => "".to_string(),
        }
    }

    pub fn select_next(&mut self, step: usize) {
        let new_index = match self.list_state.selected() {
            None => 0,
            Some(index) => min(index + step, self.visible_items.len() - 1),
        };
        self.list_state.select(Some(new_index));
        self.recalculate_selection_level();
    }

    pub fn select_next_object(&mut self) {
        let new_index = match self.list_state.selected() {
            None => Some(0),
            Some(selection_index) => {
                let indent = self.visible_items[selection_index].indent;
                self.visible_items
                    .iter()
                    .enumerate()
                    .find(|(index, item)| index > &selection_index && item.indent == indent)
                    .map(|(index, _item)| index)
            }
        }
        .unwrap_or(self.list_state.selected().unwrap_or(0));
        self.list_state.select(Some(new_index));
        self.recalculate_selection_level();
    }

    pub fn select_previous(&mut self, step: usize) {
        let new_index = match self.list_state.selected() {
            None => 0,
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

    pub fn select_previous_object(&mut self) {
        let new_index = match self.list_state.selected() {
            None => Some(0),
            Some(selection_index) => {
                let indent = self.visible_items[selection_index].indent;
                self.visible_items
                    .iter()
                    .enumerate()
                    .rfind(|(index, item)| index < &selection_index && item.indent == indent)
                    .map(|(index, _item)| index)
            }
        }
        .unwrap_or(self.list_state.selected().unwrap_or(0));
        self.list_state.select(Some(new_index));
        self.recalculate_selection_level();
    }

    pub fn select_top(&mut self) {
        self.list_state.select(Some(0));
        self.recalculate_selection_level();
    }

    pub fn select_bottom(&mut self) {
        self.list_state.select(Some(self.visible_items.len() - 1));
        self.recalculate_selection_level();
    }

    pub fn select_top_of_screen(&mut self) {
        self.list_state.select(Some(self.list_state.offset()));
        self.recalculate_selection_level();
    }

    pub fn select_middle_of_screen(&mut self) {
        let top = self.list_state.offset() as u16;
        let num_items = self.visible_items.len() as u16;
        let bottom = min(top + num_items - 1, top + self.list_height - 2);
        let index = (top + bottom) / 2;
        self.list_state.select(Some((index) as usize));
        self.recalculate_selection_level();
    }

    pub fn select_bottom_of_screen(&mut self) {
        let top = self.list_state.offset() as u16;
        let num_items = self.visible_items.len() as u16;
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

    pub fn collapse_level(&mut self) {
        if let Some(index) = self.selection_index() {
            match &self.items[index].value {
                JsonValueType::Array | JsonValueType::Object => {
                    let indent = self.items[index].indent;
                    let line_number = self.items[index].line_number;
                    for item in self.items.iter_mut() {
                        if item.indent == indent
                            && (item.value == JsonValueType::Array
                                || item.value == JsonValueType::Object)
                        {
                            item.collapsed = true;
                        }
                    }
                    self.recalculate_visible();
                    self.list_state.select(
                        self.visible_items
                            .iter()
                            .position(|item| item.line_number == line_number),
                    );
                    self.recalculate_selection_level();
                }
                _ => {}
            }
        }
    }

    pub fn uncollapse_all(&mut self) {
        let line_number = self.visible_items[self.list_state.selected().unwrap_or(0)].line_number;
        for item in self.items.iter_mut() {
            item.collapsed = false;
        }
        self.recalculate_visible();
        self.list_state.select(
            self.visible_items
                .iter()
                .position(|item| item.line_number == line_number),
        );
        self.recalculate_selection_level();
    }

    pub fn selection_index(&self) -> Option<usize> {
        self.list_state
            .selected()
            .map(|index| self.visible_indices()[index])
            .map(|index| self.visible_items[index].line_number)
    }

    fn recalculate_visible(&mut self) {
        let mut is_in_collapsed = false;
        let mut collapse_indent = 0;
        for item in self.items.iter_mut() {
            item.visible = true;
            if is_in_collapsed {
                if item.indent == collapse_indent {
                    is_in_collapsed = false;
                    item.visible = false; // closing bracket
                    continue;
                }
                item.visible = false;
            } else if item.collapsed {
                is_in_collapsed = true;
                collapse_indent = item.indent;
            }
        }
        self.visible_items = self.items.iter().filter(|i| i.visible).cloned().collect();
    }

    fn recalculate_selection_level(&mut self) {
        if let Some(index) = self.selection_index() {
            // For non-containers, strip away the last component of the breadcrumbs
            let selection_breadcrumbs = match self.items[index].value {
                JsonValueType::Number(_)
                | JsonValueType::Bool(_)
                | JsonValueType::String(_)
                | JsonValueType::Null => match self.items[index].breadcrumbs.rsplit_once(" ▶ ") {
                    Some((val, _)) => val.to_string(),
                    None => "".to_string(),
                },
                _ => self.items[index].breadcrumbs.clone(),
            };
            let mut selection_level: usize;
            // Loop through all items and calculate selection level
            for item in self.visible_items.iter_mut() {
                if item.breadcrumbs.starts_with(&selection_breadcrumbs) {
                    selection_level = 0;
                    // How many components of the breadcrumbs match?
                    for (p1, p2) in zip(
                        selection_breadcrumbs.split(" ▶ "),
                        item.breadcrumbs.split(" ▶ "),
                    ) {
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
            None => NotSearching,
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
            item.update_is_search_result(
                self.search_input.value(),
                self.search_state != NotSearching,
            );
        }
        if self.search_state == Searching {
            let search_results = self.search_results();
            if !search_results.is_empty() {
                let new_index = 0;
                self.list_state.select(Some(search_results[new_index]));
                self.recalculate_selection_level();
            }
        }
    }

    pub fn search_text(&self) -> &str {
        self.search_input.value()
    }

    pub fn next_search_result(&mut self) {
        if let BrowsingSearch(Some(index)) = self.search_state {
            let search_results = self.search_results();
            let new_index = (index + 1) % search_results.len();
            self.list_state.select(Some(search_results[new_index]));
            self.recalculate_selection_level();
            self.search_state = BrowsingSearch(Some(new_index));
        }
    }

    pub fn previous_search_result(&mut self) {
        if let BrowsingSearch(Some(index)) = self.search_state {
            let search_results = self.search_results();
            let new_index = match index {
                0 => search_results.len() - 1,
                _ => index - 1,
            };
            self.list_state.select(Some(search_results[new_index]));
            self.recalculate_selection_level();
            self.search_state = BrowsingSearch(Some(new_index));
        }
    }
}
