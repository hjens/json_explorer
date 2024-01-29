use std::cmp::min;
use std::iter::zip;

use crossterm::event::Event;
use ratatui::widgets::*;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::app_state::SearchState::{BrowsingSearch, NotSearching, Searching};
use crate::json_item::{JsonItem, JsonValueType};
use thousands::Separable;

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
    num_items_in_file: usize,
    top_index: usize,
}
// list_state.selected: index into visible_items
// self.selection_index(): index into items

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
            num_items_in_file: 0,
            top_index: 0,
        };
        let values: Vec<&JsonItem> = items
            .iter()
            .filter(|i| i.value != JsonValueType::ObjectEnd && i.value != JsonValueType::ArrayEnd)
            .collect();
        app_state.num_items_in_file = values.len();
        app_state.select_next(1);
        app_state
    }

    fn bottom_index(&self) -> usize {
        if self.list_height < 2 {
            1
        } else {
            let top = self.top_index as i32;
            let height = self.list_height as i32;
            let num_visible_items = self.visible_items.len() as i32;
            min(top + height - 1, num_visible_items) as usize
        }
    }

    pub fn display_items(&self) -> Vec<JsonItem> {
        self.visible_items[self.top_index..self.bottom_index()].to_vec()
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
                let f = self.selection_index().unwrap_or(0) as f32 / (self.items.len() - 1) as f32;
                format!(
                    " {} values in file | {:.0} %",
                    self.num_items_in_file.separate_with_spaces(),
                    f * 100.0
                )
            }
        }
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
        self.select_index(new_index);
    }

    pub fn select_next_object(&mut self) {
        let new_index = match self.list_state.selected() {
            None => Some(0),
            Some(selection_index) => {
                let indent = match self.visible_items[selection_index].value {
                    JsonValueType::Array
                    | JsonValueType::ArrayEnd
                    | JsonValueType::Object
                    | JsonValueType::ObjectEnd => self.visible_items[selection_index].indent,
                    _ => self.visible_items[selection_index].indent - 1,
                };
                self.visible_items
                    .iter()
                    .enumerate()
                    .find(|(index, item)| index > &selection_index && item.indent == indent)
                    .map(|(index, _item)| index)
            }
        }
        .unwrap_or(self.list_state.selected().unwrap_or(0));
        self.select_index(new_index);
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
        self.select_index(new_index);
    }

    pub fn select_previous_object(&mut self) {
        let new_index = match self.list_state.selected() {
            None => Some(0),
            Some(selection_index) => {
                let indent = match self.visible_items[selection_index].value {
                    JsonValueType::Array
                    | JsonValueType::ArrayEnd
                    | JsonValueType::Object
                    | JsonValueType::ObjectEnd => self.visible_items[selection_index].indent,
                    _ => self.visible_items[selection_index].indent - 1,
                };
                self.visible_items
                    .iter()
                    .enumerate()
                    .rfind(|(index, item)| index < &selection_index && item.indent == indent)
                    .map(|(index, _item)| index)
            }
        }
        .unwrap_or(self.list_state.selected().unwrap_or(0));
        self.select_index(new_index);
    }

    pub fn select_top(&mut self) {
        self.select_index(0);
    }

    pub fn select_bottom(&mut self) {
        self.select_index(self.visible_items.len() - 1);
    }

    pub fn select_top_of_screen(&mut self) {
        self.select_index(self.top_index);
    }

    pub fn select_middle_of_screen(&mut self) {
        let top = self.top_index as u16;
        let num_items = self.visible_items.len() as u16;
        let bottom = min(top + num_items - 1, top + self.list_height - 2);
        let index = (top + bottom) / 2;
        self.select_index((index) as usize);
    }

    pub fn select_bottom_of_screen(&mut self) {
        let top = self.top_index as u16;
        let num_items = self.visible_items.len() as u16;
        let index = min(top + num_items - 1, top + self.list_height - 2);
        self.select_index(index as usize);
    }

    pub fn toggle_collapsed(&mut self) {
        if let Some(index) = self.selection_index() {
            {
                let mut i = index;
                loop {
                    match &self.items[i].value {
                        JsonValueType::Array | JsonValueType::Object => {
                            self.items[i].collapsed = !self.items[i].collapsed;
                            if let Some(selection) = self.list_state.selected() {
                                let diff = index - i;
                                self.select_index(selection - diff);
                            }
                            self.recalculate_visible();
                            self.recalculate_selection_level();
                            break;
                        }
                        _ => {
                            if i == 0 {
                                break;
                            }
                            i -= 1;
                        }
                    }
                }
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
                    self.select_index(
                        self.visible_items
                            .iter()
                            .position(|item| item.line_number == line_number)
                            .unwrap_or(0),
                    );
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
        self.select_index(
            self.visible_items
                .iter()
                .position(|item| item.line_number == line_number)
                .unwrap_or(0),
        );
    }

    pub fn selection_index(&self) -> Option<usize> {
        self.list_state
            .selected()
            .map(|index| self.visible_items[index].line_number)
    }

    fn recalculate_visible(&mut self) {
        // TODO: optimize
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
            let bottom = self.bottom_index();
            // Loop through all items and calculate selection level
            for item in self.visible_items[self.top_index..bottom].iter_mut() {
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

    fn select_index(&mut self, index: usize) {
        self.list_state.select(Some(index));
        self.recalculate_scroll_position();
        self.recalculate_selection_level();
    }

    fn recalculate_scroll_position(&mut self) {
        if let Some(index) = self.list_state.selected() {
            if index < self.top_index {
                self.top_index = index;
            }
            if index >= self.bottom_index() {
                let mut new_top_index = (index as i32) - (self.list_height as i32) + 2;
                if new_top_index < 0 {
                    new_top_index = 0;
                }
                self.top_index = new_top_index as usize;
            }
        }
    }

    pub fn start_searching(&mut self) {
        self.uncollapse_all();
        self.search_state = Searching;
        self.search_input = Input::new("".to_string());
        self.update_search_results();
    }

    pub fn start_searching_for_name(&mut self) {
        if let Some(index) = self.selection_index() {
            if let Some(name) = self.visible_items[index].name.clone() {
                self.search_input = self.search_input.clone().with_value(name);
                self.start_searching();
                self.finish_searching();
            }
        }
    }

    pub fn cancel_searching(&mut self) {
        self.search_state = NotSearching;
        self.update_search_results();
    }
    pub fn finish_searching(&mut self) {
        self.update_search_results();
        self.search_state = match self.search_results().first() {
            Some(_) => BrowsingSearch(Some(0)),
            None => NotSearching,
        };
    }

    fn search_results(&self) -> Vec<usize> {
        self.visible_items
            .iter()
            .enumerate()
            .filter(|(_index, item)| item.name_is_search_result || item.value_is_search_result)
            .map(|(index, _item)| index)
            .collect()
    }

    pub fn update_search(&mut self, event: &Event) {
        let is_large_file = self.num_items_in_file > 1_00_000;
        self.search_input.handle_event(event);
        if !is_large_file {
            self.update_search_results();
        }
    }

    fn update_search_results(&mut self) {
        for item in self.visible_items.iter_mut() {
            item.update_is_search_result(
                self.search_input.value(),
                self.search_state != NotSearching,
            );
        }
        if self.search_state == Searching {
            let search_results = self.search_results();
            if !search_results.is_empty() {
                let new_index = 0;
                self.select_index(search_results[new_index]);
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
            self.select_index(search_results[new_index]);
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
            self.select_index(search_results[new_index]);
            self.search_state = BrowsingSearch(Some(new_index));
        }
    }
}
