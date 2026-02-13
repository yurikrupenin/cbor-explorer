mod app;
mod cbor_tree;
mod cbor_parser;
mod config;
mod theme;
mod ui;

use app::App;
use clap::Parser;
use color_eyre::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cbor-explorer")]
#[command(author = "MiniMax Agent")]
#[command(version = "0.1.0")]
#[command(about = "A TUI application for exploring CBOR files", long_about = None)]
struct Args {
    /// Path to the CBOR file to explore
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new(&args.file)?;
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(());
            }

            if app.popups == app::PopupMode::ThemeSelect {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => app.move_theme_selection_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_theme_selection_down(),
                    KeyCode::Enter => app.confirm_theme_selection(),
                    KeyCode::Esc => app.cancel_theme_selection(),
                    // Still allow numeric selection for quick access
                    KeyCode::Char(c) => {
                        if let Some(digit) = c.to_digit(10) {
                             if digit > 0 {
                                 app.apply_theme((digit - 1) as usize);
                             }
                        }
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char(config::keys::QUIT) => return Ok(()),
                    KeyCode::Tab => app.toggle_focus(),
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Left | KeyCode::Char('h') => app.move_left(),
                    KeyCode::Right | KeyCode::Char('l') => app.move_right(),
                    KeyCode::Enter => app.toggle_expand(),
                    KeyCode::Char(' ') => app.toggle_popup(),
                    KeyCode::Home | KeyCode::Char('g') => app.go_to_start(),
                    KeyCode::End | KeyCode::Char('G') => app.go_to_end(),
                    KeyCode::PageUp => app.page_up(),
                    KeyCode::PageDown => app.page_down(),
                    KeyCode::Char(config::keys::EXPAND_ALL) => app.expand_all(),
                    KeyCode::Char(config::keys::COLLAPSE_ALL) => app.collapse_all(),
                    KeyCode::Char(config::keys::HELP) => app.toggle_help(),
                    KeyCode::Char(config::keys::TOGGLE_HEX_INT) => app.toggle_hex_integers(),
                    KeyCode::Char(config::keys::THEME_SELECT) => app.open_theme_dialog(),
                    _ => {}
                }
            }
        }
    }
}
