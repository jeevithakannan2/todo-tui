use crate::{
    app::{PRIMARY_STYLE, SECONDARY_STYLE},
    handle_json::Todo,
};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, Clear},
};
use tui_textarea::TextArea;

pub struct NewTask<'a> {
    focus: Focus,
    mode: Mode,
    widgets: Widgets<'a>,
    pub todo: Todo,
    pub quit: bool,
    pub completed: bool,
}

struct Widgets<'a> {
    title: TextArea<'a>,
    date: TextArea<'a>,
    description: TextArea<'a>,
}

#[derive(PartialEq)]
enum Focus {
    Title,
    Date,
    Description,
    ConfirmPropmt,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}

impl Widget for &mut NewTask<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        self.render_border(area, buf);

        let area = area.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });

        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [title_area, date_area, description_area] = vertical.areas(area);

        self.set_cursor_style();
        self.widgets.title.render(title_area, buf);
        self.widgets.date.render(date_area, buf);
        self.widgets.description.render(description_area, buf);

        if self.focus == Focus::ConfirmPropmt {
            crate::confirm::Confirm::new(
                " Confirm Save".into(),
                "Do you want to save this task".into(),
            )
            .render(area, buf);
        }
    }
}

impl Widgets<'_> {
    pub fn new() -> Self {
        let mut title = TextArea::default();
        let mut description = TextArea::default();
        let mut date = TextArea::default();
        Widgets::set_block(&mut title, &mut date, &mut description);
        Self {
            title,
            date,
            description,
        }
    }

    pub fn from(title: Vec<String>, date: Vec<String>, description: Vec<String>) -> Self {
        let mut title = TextArea::new(title);
        let mut date = TextArea::new(date);
        let mut description = TextArea::new(description);
        Widgets::set_block(&mut title, &mut date, &mut description);
        Self {
            title,
            date,
            description,
        }
    }

    fn set_block(title: &mut TextArea, date: &mut TextArea, description: &mut TextArea) {
        // Helper function to create a bordered block
        fn get_block(title: &str) -> Block {
            Block::bordered()
                .title(title)
                .border_type(BorderType::Rounded)
        }

        title.set_block(get_block(" Title "));
        date.set_block(get_block(" Date "));
        description.set_block(get_block(" Description "));

        // Removes the underline when typed
        title.set_cursor_line_style(Style::default());
        date.set_cursor_line_style(Style::default());
        description.set_cursor_line_style(Style::default());

        title.set_placeholder_text("Enter your task title here");
        date.set_placeholder_text("Enter your task date here");
        description.set_placeholder_text("Enter your task description here");
    }
}

impl NewTask<'_> {
    pub fn new() -> Self {
        Self {
            focus: Focus::Title,
            mode: Mode::Normal,
            quit: false,
            completed: false,
            todo: Todo::new(),
            widgets: Widgets::new(),
        }
    }

    pub fn from(todo: Todo) -> Self {
        let description = todo.description.lines().map(|s| s.to_string()).collect();
        let date = vec![todo.date];
        let title = vec![todo.title];
        Self {
            focus: Focus::Title,
            mode: Mode::Normal,
            quit: false,
            completed: false,
            todo: Todo::from(Some(todo.id), None, None, None, None),
            widgets: Widgets::from(title, date, description),
        }
    }

    fn set_cursor_style(&mut self) {
        let cursor_style = if self.mode == Mode::Insert {
            match self.focus {
                Focus::Title => (
                    Style::default().reversed(),
                    Style::default(),
                    Style::default(),
                ),
                Focus::Date => (
                    Style::default(),
                    Style::default().reversed(),
                    Style::default(),
                ),
                Focus::Description => (
                    Style::default(),
                    Style::default(),
                    Style::default().reversed(),
                ),
                Focus::ConfirmPropmt => (Style::default(), Style::default(), Style::default()),
            }
        } else {
            (Style::default(), Style::default(), Style::default())
        };

        self.widgets.title.set_cursor_style(cursor_style.0);
        self.widgets.date.set_cursor_style(cursor_style.1);
        self.widgets.description.set_cursor_style(cursor_style.2);
    }

    fn render_border(&self, area: Rect, buf: &mut Buffer) {
        let style = match self.focus {
            Focus::ConfirmPropmt => SECONDARY_STYLE,
            _ => PRIMARY_STYLE,
        };
        crate::helpers::rounded_block(" New Task ", style).render(area, buf);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.focus == Focus::ConfirmPropmt {
            match key.code {
                KeyCode::Char('y') => {
                    let title_val = self.widgets.title.lines()[0].to_string();
                    let date_val: String = self.widgets.date.lines()[0].to_string();
                    let description_val = self
                        .widgets
                        .description
                        .lines()
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join("\n");
                    self.todo = Todo {
                        id: self.todo.id,
                        title: title_val,
                        date: date_val,
                        description: description_val,
                        completed: false,
                    };
                    self.quit = true;
                    self.completed = true;
                }
                KeyCode::Char('n') => self.focus = Focus::Title,
                _ => {}
            }
            return;
        }

        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('q') => {
                    self.focus = Focus::Title;
                    self.quit = true;
                }
                KeyCode::Char('i') => {
                    if self.mode == Mode::Normal {
                        self.mode = Mode::Insert;
                    }
                }
                KeyCode::Enter => {
                    self.mode = Mode::Normal;
                    self.focus = Focus::ConfirmPropmt;
                }
                _ => {}
            },
            Mode::Insert => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Tab | KeyCode::BackTab => {
                    self.focus = match self.focus {
                        Focus::Title => Focus::Date,
                        Focus::Date => Focus::Description,
                        Focus::Description => Focus::Title,
                        Focus::ConfirmPropmt => Focus::ConfirmPropmt,
                    }
                }
                _ => match self.focus {
                    Focus::Title => {
                        if key.code == KeyCode::Enter {
                            return;
                        }
                        self.widgets.title.input(key);
                    }
                    Focus::Date => {
                        self.widgets.date.input(key);
                    }
                    Focus::Description => {
                        self.widgets.description.input(key);
                    }
                    Focus::ConfirmPropmt => {}
                },
            },
        }
    }

    pub fn footer_text(&self) -> &str {
        match self.mode {
            Mode::Normal => match self.focus {
                Focus::Description | Focus::Title | Focus::Date => {
                    "[q] Quit without saving | [i] Insert Mode | [Enter] Save"
                }
                Focus::ConfirmPropmt => "[y] Yes | [n] No",
            },
            Mode::Insert => "[Esc] Normal Mode | [Tab] Switch Fields",
        }
    }
}
