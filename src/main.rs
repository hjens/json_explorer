use std::error::Error;
use std::io::Stdout;
use std::process::exit;
use std::{fs, io};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app_state::AppState;

mod ui;

mod app_state;
mod json_item;
mod parse_json;
mod theme;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let json_text: String;
    if args.len() == 2 {
        let input_file: String = args.nth(1).unwrap();
        json_text = fs::read_to_string(input_file).expect("Could not read from file");
    } else {
        println!("Usage: `jex [INPUT_FILE]`");
        exit(1);
    }
    let json_values = parse_json::parse_json_string(&json_text).expect("Could not parse json.");

    let mut app_state = AppState::new(json_values, "".to_string());
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = create_terminal();

    let res = ui::run_app(&mut terminal, &mut app_state);

    destroy_terminal(&mut terminal);

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn create_terminal() -> Terminal<CrosstermBackend<Stdout>> {
    enable_raw_mode().expect("Unable to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).expect("Unable to set up stdout");
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).expect("Unable to set up terminal")
}

fn destroy_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
    disable_raw_mode().expect("Unable to disable raw mode");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .expect("Unable to restore terminal");
    terminal.show_cursor().expect("Unable to show cursor");
}
