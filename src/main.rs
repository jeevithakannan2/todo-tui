use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyEventKind},
};
use std::io::Result;
use ui::App;

mod auth;
mod cli;
mod config;
mod helpers;
mod tasks;
mod theme;
mod ui;

fn main() -> Result<()> {
    cli::handle_arguments()?;
    let mut terminal = ratatui::init();
    let config = crate::config::Config::load();
    let app_result = run(&mut terminal, App::new(!config.exists(), config));
    ratatui::restore();
    app_result
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
