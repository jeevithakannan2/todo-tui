use app::App;
use ratatui::{
    crossterm::event::{self, Event, KeyEventKind},
    DefaultTerminal,
};
use std::io::Result;

mod app;
mod confirm;
mod handle_json;
mod helpers;
mod new_task;
mod settings;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let app = App::new();
    let app_result = run(&mut terminal, app);
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
