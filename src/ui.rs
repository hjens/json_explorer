use std::io;

use crossterm::{
    event::{self, Event, KeyCode},
};
use crossterm::terminal::size;
use ratatui::{Frame, Terminal};
use ratatui::{
    backend::Backend,
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::Block,
};
use ratatui::widgets::{Borders, BorderType, List, ListItem};

use crate::app_state::AppState;

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app_state: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app_state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('j') => {
                    app_state.select_next();
                }
                KeyCode::Char('k') => {
                    app_state.select_previous();
                }
                KeyCode::Char('c') => {
                    app_state.toggle_collapsed();
                }
                _ => {}
            }
        }
    }
}

fn render(frame: &mut Frame, app_state: &mut AppState) {
    let size = frame.size();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Main block with round corners")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    frame.render_widget(block, size);

    let visible_items = app_state.visible_items();
    let list_items: Vec<ListItem> = visible_items
        .iter()
        .map(|i| ListItem::new(i.display_text()))
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Json file"))
        .highlight_style(
            Style::default()
                .bg(Color::LightMagenta)
                .add_modifier(Modifier::BOLD)
        );

    frame.render_stateful_widget(list, size, &mut app_state.list_state);
}

