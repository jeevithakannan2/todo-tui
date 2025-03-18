use app::App;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyEventKind},
};
use std::io::Result;

#[cfg(feature = "encryption")]
use crate::cli::Args;
#[cfg(feature = "encryption")]
use clap::Parser;

mod app;
mod confirm;
mod handle_json;
mod helpers;
mod new_task;
mod theme;

#[cfg(feature = "encryption")]
mod auth;
#[cfg(feature = "encryption")]
mod cli;

fn main() -> Result<()> {
    #[cfg(feature = "encryption")]
    let args = Args::parse();
    #[cfg(feature = "encryption")]
    enc(args)?;
    let mut terminal = ratatui::init();
    let app_result = run(&mut terminal, App::new());
    ratatui::restore();
    app_result
}

#[cfg(feature = "encryption")]
fn enc(args: Args) -> Result<()> {
    if args.reset {
        handle_json::reset()?;
        auth::generate_key();
    }
    if args.generate_key {
        auth::generate_key();
    }
    Ok(())
}

fn run(terminal: &mut DefaultTerminal, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|frame| frame.render_widget(&mut app, frame.area()))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
                continue;
            }
            if app.handle_key(key) {
                break;
            }
        }
    }
    Ok(())
}
