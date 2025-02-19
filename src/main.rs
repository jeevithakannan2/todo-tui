use app::App;
use ratatui::{
    crossterm::event::{self, Event, KeyEventKind},
    DefaultTerminal,
};
use std::io::Result;

mod app;
mod confirm;
mod handle_json;
mod new_task;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let app = App::new();
    let app_result = run(&mut terminal, app);
    ratatui::restore();
    app_result
}

fn run(terminal: &mut DefaultTerminal, mut app: App) -> Result<()> {
    let mut should_exit = false;
    while !should_exit {
        terminal.draw(|frame| frame.render_widget(&mut app, frame.area()))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
                continue;
            }

            should_exit = app.on_key(key);
        }
    }
    Ok(())
}
