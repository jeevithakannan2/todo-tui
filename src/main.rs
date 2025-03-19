use app::App;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyEventKind},
};
use std::io::Result;

mod app;
mod auth;
mod cli;
mod confirm;
mod tasks;
mod helpers;
mod new_task;
mod settings;
mod theme;

fn main() -> Result<()> {
    cli::handle_arguments()?;
    let mut terminal = ratatui::init();
    let settings = settings::load().unwrap();
    let app_result = run(
        &mut terminal,
        App::new(!settings::exists(), settings),
    );
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
