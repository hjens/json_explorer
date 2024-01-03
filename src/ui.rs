use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
};
use ratatui::{Frame, Terminal};
use ratatui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    widgets::Block,
};
use ratatui::{prelude::*, widgets::*};

use crate::app_state::AppState;
use crate::app_state::SearchState;

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app_state: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app_state))?;

        if let Event::Key(key) = event::read()? {
            match app_state.search_state {
                SearchState::Searching => {
                    match key.code {
                        KeyCode::Enter => {
                            app_state.finish_searching();
                        }
                        KeyCode::Esc => {
                            app_state.cancel_searching();
                        }
                        _ => {
                            app_state.update_search(&Event::Key(key));
                        }
                    }
                }
                SearchState::BrowsingSearch(_) => {
                    match key.code {
                        KeyCode::Char('n') => {
                            app_state.next_search_result();
                        }
                        KeyCode::Char('N') => {
                            app_state.previous_search_result();
                        }
                        KeyCode::Esc => {
                            app_state.cancel_searching();
                        }
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    }
                }
                SearchState::NotSearching => {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') => {
                            app_state.select_next(1);
                        }
                        KeyCode::Char('k') => {
                            app_state.select_previous(1);
                        }
                        KeyCode::Char('c') => {
                            app_state.toggle_collapsed();
                        }
                        KeyCode::Char('g') => {
                            app_state.select_top();
                        }
                        KeyCode::Char('G') => {
                            app_state.select_bottom();
                        }
                        KeyCode::Char('H') => {
                            app_state.select_top_of_screen();
                        }
                        KeyCode::Char('M') => {
                            app_state.select_middle_of_screen();
                        }
                        KeyCode::Char('L') => {
                            app_state.select_bottom_of_screen();
                        }
                        KeyCode::Char(' ') => {
                            if let Ok(size) = terminal.size() {
                                if size.height > 5 {
                                    app_state.select_next((size.height - 5) as usize);
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if let Ok(size) = terminal.size() {
                                if size.height > 5 {
                                    app_state.select_previous((size.height - 5) as usize);
                                }
                            }
                        }
                        KeyCode::Char('/') => {
                            app_state.start_searching();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let size = frame.size();

    let chunks = match app_state.search_state {
        SearchState::Searching => {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3)
                ])
                .split(size)
        }
        _ => {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(5),
                    Constraint::Length(3)
                ])
                .split(size)
        }
    };
    let list_chunk = match app_state.search_state {
        SearchState::Searching => chunks[1],
        _ => chunks[0]
    };
    app_state.list_height = list_chunk.height;

    let breadbrumbs = Paragraph::new(Text::styled(
        app_state.breadbrumbs_text(),
        Style::default().fg(Color::Green),
    ))
        .block(Block::default().borders(Borders::ALL));

    let visible_items = app_state.visible_items();
    let selection_index = app_state.list_state.selected().unwrap_or(0) as i32;
    let list_items: Vec<ListItem> = visible_items
        .iter()
        .enumerate()
        .map(|(index, item)| ListItem::new(item.display_text(
            index as i32,
            selection_index,
            list_chunk.height as i32,
        )))
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::TOP).title(app_state.filename.clone()))
        .highlight_style(
            Style::default()
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD)
        );

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));
    let mut scrollbar_state = ScrollbarState::new(visible_items.iter().len()).position(app_state.scroll_position());

    let search = Paragraph::new(app_state.search_text().to_string().clone())
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title("Search:"));

    let breadcrumbs_chunk = match app_state.search_state {
        SearchState::Searching => chunks[2],
        _ => chunks[1]
    };

    frame.render_stateful_widget(list, list_chunk, &mut app_state.list_state);
    frame.render_stateful_widget(
        scrollbar,
        list_chunk.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
    frame.render_widget(breadbrumbs, breadcrumbs_chunk);
    if app_state.search_state == SearchState::Searching {
        frame.render_widget(search, chunks[0]);
    }

    // Place cursor
    let width = size.width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = app_state.search_input.visual_scroll(width as usize);
    let cursor_y = 1;
    match app_state.search_state {
        SearchState::Searching => {
            frame.set_cursor(
                ((app_state.search_input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
                cursor_y as u16
            )
        }
        _ => {}  // hide the cursor
    }
}
