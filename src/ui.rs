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

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app_state: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app_state))?;

        if let Event::Key(key) = event::read()? {
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
                _ => {}
            }
        }
    }
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let size = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3)
        ])
        .split(size);

    let breadbrumbs = Paragraph::new(Text::styled(
        app_state.breadbrumbs_text(),
        Style::default().fg(Color::Green),
    ))
        .block(Block::default().borders(Borders::ALL));

    let visible_items = app_state.visible_items();
    let top_index = app_state.top_index();
    let bottom_index = app_state.bottom_index(chunks[0].height as usize);
    let list_items: Vec<ListItem> = visible_items
        .iter()
        .enumerate()
        .map(|(index, item)| ListItem::new(item.display_text(top_index, bottom_index, index)))
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::TOP).title("Json file"))
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


    frame.render_stateful_widget(list, chunks[0], &mut app_state.list_state);
    frame.render_stateful_widget(
        scrollbar,
        chunks[0].inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
    frame.render_widget(breadbrumbs, chunks[1]);
}

