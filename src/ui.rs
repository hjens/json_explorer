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

    frame.render_stateful_widget(list, chunks[0], &mut app_state.list_state);
    frame.render_widget(breadbrumbs, chunks[1]);
}

