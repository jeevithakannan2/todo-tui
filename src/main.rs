use app::App;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyEventKind},
};
use std::io::Result;

#[cfg(feature = "encryption")]
use crate::cli::Args;
#[cfg(feature = "encryption")]
use auth::PasswordPrompt;
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

trait WidgetHandler {
    fn handle_key(&mut self, key: event::KeyEvent) -> bool;
}

#[cfg(feature = "encryption")]
impl WidgetHandler for PasswordPrompt<'_> {
    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        self.handle_key(key)
    }
}

impl WidgetHandler for App<'_> {
    fn handle_key(&mut self, key: event::KeyEvent) -> bool {
        self.handle_key(key)
    }
}

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    #[cfg(feature = "encryption")]
    enc(&mut terminal)?;
    let app_result = run_loop(&mut terminal, App::new());
    ratatui::restore();
    app_result
}

#[cfg(feature = "encryption")]
fn enc(terminal: &mut DefaultTerminal) -> Result<()> {
    let args = Args::parse();
    if args.reset {
        handle_json::reset()?;
        auth::delete_stored_password();
    }
    let password = auth::get_password();
    if password.is_empty() || args.reset {
        run_loop(terminal, PasswordPrompt::new())?;
    }
    if args.change_password {
        let tasks = handle_json::load_tasks_encrypted().unwrap_or_else(|_| Vec::new());
        run_loop(terminal, PasswordPrompt::new())?;
        handle_json::save_tasks_ecrypted(&tasks).unwrap();
    }
    Ok(())
}

fn run_loop<T>(terminal: &mut DefaultTerminal, mut widget: T) -> Result<()>
where
    T: WidgetHandler,
    for<'a> &'a mut T: ratatui::prelude::Widget,
{
    loop {
        terminal.draw(|frame| frame.render_widget(&mut widget, frame.area()))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press && key.kind != KeyEventKind::Repeat {
                continue;
            }
            if widget.handle_key(key) {
                break;
            }
        }
    }
    Ok(())
}
