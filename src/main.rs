use cbx::app::App;
use cbx::run_app;
use clap::Parser;
use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cbx")]
#[command(author = "Yuri Krupenin <yuri.krupenin@gmail.com>")]
#[command(version)]
#[command(about = "A TUI application for inspecting CBOR data", long_about = None)]
struct Args {
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    // Check if file exists and is readable
    if let Err(err) = std::fs::File::open(&args.file) {
        eprintln!(
            "Error: Failed to open file '{}': {}",
            args.file.display(),
            err
        );
        std::process::exit(1);
    }

    if args.file.is_dir() {
        eprintln!("Error: '{}' is a directory.", args.file.display());
        std::process::exit(1);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::load_from_file(&args.file)?;
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
