use serde::{Deserialize, Serialize};
use std::{fs, io};

use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, Clear},
};
use tui_textarea::TextArea;

pub struct NewSettings<'a> {
    focus: Focus,
    mode: Mode,
    widgets: Widgets<'a>,
    pub settings: Settings,
    pub quit: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub git_settings: GitSettings,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GitSettings {
    pub repo: String,
    pub branch: String,
}

struct Widgets<'a> {
    repo: TextArea<'a>,
    branch: TextArea<'a>,
}

#[derive(PartialEq)]
enum Focus {
    Repo,
    Branch,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}

use directories::ProjectDirs;

pub fn load_settings() -> io::Result<Settings> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    let data = fs::read_to_string(format!(
        "{}/settings.json",
        dir.config_dir().to_str().unwrap()
    ))
    .unwrap_or_else(|_| "{}".to_string());
    let settings: Settings =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(settings)
}

pub fn save_settings(settings: &Settings) -> io::Result<()> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    fs::DirBuilder::new()
        .recursive(true)
        .create(dir.config_dir())
        .unwrap();
    let path = format!("{}/settings.json", dir.config_dir().to_str().unwrap());
    let data = serde_json::to_string(settings)?;
    fs::write(path, data)
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            git_settings: GitSettings {
                repo: String::new(),
                branch: String::new(),
            },
        }
    }
}

impl Settings {
    pub fn from(repo: Option<String>, branch: Option<String>) -> Self {
        Self {
            git_settings: GitSettings {
                repo: repo.unwrap_or_default(),
                branch: branch.unwrap_or_default(),
            },
        }
    }
}

impl Widget for &mut NewSettings<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        self.render_border(area, buf);

        let area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        let vertical = Layout::vertical([Constraint::Length(3), Constraint::Length(3)]);
        let [repo_area, branch_area] = vertical.areas(area);

        self.set_cursor_style();
        self.widgets.repo.render(repo_area, buf);
        self.widgets.branch.render(branch_area, buf);
    }
}

impl Widgets<'_> {
    pub fn new() -> Self {
        let mut repo = TextArea::default();
        let mut branch = TextArea::default();
        Widgets::set_block(&mut repo, &mut branch);
        Self { repo, branch }
    }

    pub fn from(repo: Vec<String>, branch: Vec<String>) -> Self {
        let mut repo = TextArea::new(repo);
        let mut branch = TextArea::new(branch);
        Widgets::set_block(&mut repo, &mut branch);
        Self { repo, branch }
    }

    fn set_block(repo: &mut TextArea, branch: &mut TextArea) {
        // Helper function to create a bordered block
        fn get_block(repo: &str) -> Block {
            Block::bordered()
                .title(repo)
                .border_type(BorderType::Rounded)
        }

        repo.set_block(get_block(" Repo "));
        branch.set_block(get_block(" Branch "));

        // Removes the underline when typed
        repo.set_cursor_line_style(Style::default());
        branch.set_cursor_line_style(Style::default());

        repo.set_placeholder_text("Enter your repo here");
        branch.set_placeholder_text("Enter your branch here");
    }
}

impl NewSettings<'_> {
    pub fn new() -> Self {
        Self {
            focus: Focus::Repo,
            mode: Mode::Normal,
            quit: false,
            settings: Settings {
                git_settings: GitSettings {
                    repo: String::new(),
                    branch: String::new(),
                },
            },
            widgets: Widgets::new(),
        }
    }

    pub fn from(settings: &Settings) -> Self {
        let repo = &settings.git_settings.repo;
        let branch = &settings.git_settings.branch;

        Self {
            focus: Focus::Repo,
            mode: Mode::Normal,
            quit: false,
            settings: Settings::from(Some(repo.clone()), Some(branch.clone())),
            widgets: Widgets::from(vec![repo.to_owned()], vec![branch.to_owned()]),
        }
    }

    fn set_cursor_style(&mut self) {
        let cursor_style = if self.mode == Mode::Insert {
            match self.focus {
                Focus::Repo => (Style::default().reversed(), Style::default()),
                Focus::Branch => (Style::default(), Style::default().reversed()),
            }
        } else {
            (Style::default(), Style::default())
        };

        self.widgets.repo.set_cursor_style(cursor_style.0);
        self.widgets.branch.set_cursor_style(cursor_style.1);
    }

    fn render_border(&self, area: Rect, buf: &mut Buffer) {
        crate::helpers::rounded_block(" Settings ").render(area, buf);
    }

    pub fn to_settings(&self) -> Settings {
        Settings {
            git_settings: GitSettings {
                repo: self.widgets.repo.lines()[0].to_string(),
                branch: self.widgets.branch.lines()[0].to_string(),
            },
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('q') => {
                    self.focus = Focus::Repo;
                    self.quit = true;
                }
                KeyCode::Char('i') => {
                    if self.mode == Mode::Normal {
                        self.mode = Mode::Insert;
                    }
                }
                _ => {}
            },
            Mode::Insert => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Tab | KeyCode::BackTab => {
                    self.focus = match self.focus {
                        Focus::Repo => Focus::Branch,
                        Focus::Branch => Focus::Repo,
                    }
                }
                _ => match self.focus {
                    Focus::Repo => {
                        if key.code == KeyCode::Enter {
                            return;
                        }
                        self.widgets.repo.input(key);
                    }
                    Focus::Branch => {
                        self.widgets.branch.input(key);
                    }
                },
            },
        }
    }

    pub fn footer_text(&self) -> &str {
        match self.mode {
            Mode::Normal => match self.focus {
                Focus::Repo | Focus::Branch => "[q] Save and Quit | [i] Insert Mode",
            },
            Mode::Insert => "[Esc] Normal Mode | [Tab] Switch Fields",
        }
    }
}
