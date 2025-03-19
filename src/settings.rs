use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    prelude::*,
    widgets::{Cell, Row, Table, TableState, Widget},
};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf, vec};

use crate::app::{GREEN_STYLE, PRIMARY_STYLE, RED_STYLE};

#[derive(Serialize, Deserialize, Clone)]
pub struct SettingOption {
    pub name: String,
    pub value: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Settings {
    pub options: Vec<SettingOption>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            options: vec![SettingOption {
                name: "Encryption".to_string(),
                value: false,
            }],
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}

fn get_config_dir() -> PathBuf {
    crate::helpers::get_project_dir().config_dir().to_path_buf()
}

fn get_config_path() -> PathBuf {
    get_config_dir().join("settings.json")
}

fn process_settings(loaded: &mut Settings) {
    let default = Settings::default();

    // Add any missing options.
    for default_opt in &default.options {
        if !loaded.options.iter().any(|o| o.name == default_opt.name) {
            loaded.options.push(default_opt.clone());
        }
    }

    // Remove any unwanted options.
    loaded
        .options
        .retain(|option| default.options.iter().any(|o| o.name == option.name));
}

pub fn load() -> io::Result<Settings> {
    let mut loaded: Settings = match fs::read_to_string(get_config_path()) {
        Ok(contents) => serde_json::from_str(&contents)?,
        Err(_) => Settings::new(),
    };

    process_settings(&mut loaded);
    save(&loaded)?;
    Ok(loaded)
}

pub fn save(settings: &Settings) -> io::Result<()> {
    fs::create_dir_all(get_config_dir())?;
    fs::write(get_config_path(), serde_json::to_string_pretty(settings)?)
}

pub fn exists() -> bool {
    get_config_path().exists()
}

pub struct EditSettings {
    state: TableState,
    settings: Settings,
}

impl EditSettings {
    pub fn new(settings: &Settings) -> Self {
        let first_column_selection = if settings.options[0].value {
            Some(1)
        } else {
            Some(2)
        };
        Self {
            settings: settings.clone(),
            state: TableState::default()
                .with_selected(Some(0))
                .with_selected_column(first_column_selection),
        }
    }

    fn previous_column(&mut self) {
        if self.state.selected_column() == Some(1) {
            return;
        }
        self.state.select_previous_column();
    }

    fn next_column(&mut self) {
        if self.state.selected_column() == Some(2) {
            return;
        }
        self.state.select_next_column();
    }

    fn modify_settings(&mut self) {
        if let Some(selected) = self.state.selected() {
            if let Some(option) = self.settings.options.get_mut(selected) {
                option.value = self.state.selected_column() == Some(1);
            }
        }
    }

    fn auto_select_column(&mut self) {
        if let Some(selected) = self.state.selected() {
            if let Some(option) = self.settings.options.get(selected) {
                let col = if option.value { Some(1) } else { Some(2) };
                self.state.select_column(col);
            }
        }
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Left => {
                self.previous_column();
                self.modify_settings();
            }
            KeyCode::Right => {
                self.next_column();
                self.modify_settings();
            }
            KeyCode::Up => {
                self.state.select_previous();
                self.auto_select_column();
            }
            KeyCode::Down => {
                if self.state.selected() == Some(self.settings.options.len() - 1) {
                    return false;
                }
                self.state.select_next();
                self.auto_select_column();
            }
            KeyCode::Char('q') | KeyCode::Enter => return true,
            _ => {}
        }
        false
    }
}

impl Widget for &mut EditSettings {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows: Vec<Row> = self
            .settings
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let selected = self.state.selected() == Some(i);
                let (yes_style, no_style) = match (selected, option.value) {
                    (true, true) => (GREEN_STYLE.bg(Color::DarkGray), Style::default()), // Current selection is yes
                    (true, false) => (Style::default(), RED_STYLE.bg(Color::DarkGray)), // Current selection is no
                    (false, true) => (GREEN_STYLE, Style::default()), // Selection is yes but not current
                    (false, false) => (Style::default(), RED_STYLE), // Selection is no but not current
                };

                Row::new(vec![
                    Cell::from(option.name.clone()),
                    Cell::from("Yes").style(yes_style),
                    Cell::from("No").style(no_style),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ],
        )
        .block(crate::helpers::rounded_block(" Settings ", PRIMARY_STYLE))
        .row_highlight_style(Style::default().bold());
        StatefulWidget::render(table, area, buf, &mut self.state);
    }
}
