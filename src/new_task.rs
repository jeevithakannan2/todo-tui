use crate::{app::RED_STYLE, handle_json::Task};
use chrono::NaiveDate;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, Clear},
};
use tui_textarea::{CursorMove, TextArea};

#[derive(Clone)]
pub struct NewTask<'a> {
    focus: Focus,
    mode: Mode,
    widgets: Widgets<'a>,
    task: Task,
    pub quit: bool,
    pub completed: bool,
}

#[derive(Clone)]
struct Widgets<'a> {
    title: TextArea<'a>,
    date: TextArea<'a>,
    description: TextArea<'a>,
}

#[derive(PartialEq, Clone)]
enum Focus {
    Title,
    Date,
    Description,
}

#[derive(PartialEq, Clone)]
enum Mode {
    Normal,
    Insert,
}

impl Widget for &mut NewTask<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [title_area, date_area, description_area] = vertical.areas(area);

        self.set_cursor_style();
        self.widgets.title.render(title_area, buf);
        self.render_date(date_area, buf);
        self.widgets.description.render(description_area, buf);
    }
}

impl Widgets<'_> {
    pub fn new() -> Self {
        let mut title = TextArea::default();
        let mut date = TextArea::default();
        let mut description = TextArea::default();
        Self::set_block(&mut title, &mut date, &mut description);
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
        Self::set_block(&mut title, &mut date, &mut description);
        for widget in [&mut title, &mut date, &mut description].iter_mut() {
            widget.move_cursor(CursorMove::End);
        }
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
        // Date set_block will be updated in render_date fn
        description.set_block(get_block(" Description "));

        // Remove the underline when typing
        title.set_cursor_line_style(Style::default());
        date.set_cursor_line_style(Style::default());
        description.set_cursor_line_style(Style::default());

        title.set_placeholder_text("Enter your task title here");
        date.set_placeholder_text("Enter your task date here");
        description.set_placeholder_text("Enter your task description here");
    }
}

impl Focus {
    pub fn next(&self) -> Self {
        match self {
            Focus::Title => Focus::Date,
            Focus::Date => Focus::Description,
            Focus::Description => Focus::Title,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Focus::Title => Focus::Description,
            Focus::Date => Focus::Title,
            Focus::Description => Focus::Date,
        }
    }
}

impl NewTask<'_> {
    pub fn new() -> Self {
        Self {
            focus: Focus::Title,
            mode: Mode::Normal,
            quit: false,
            completed: false,
            task: Task::new(),
            widgets: Widgets::new(),
        }
    }

    fn render_date(&mut self, area: Rect, buf: &mut Buffer) {
        let date_val = self.widgets.date.lines()[0].to_string();
        let date = NaiveDate::parse_from_str(&date_val, "%Y-%m-%d");
        let style = match date {
            Ok(_) => Style::default(),
            Err(_) if !date_val.is_empty() => RED_STYLE,
            _ => Style::default(),
        };
        self.widgets.date.set_block(
            Block::bordered()
                .title(" Date - (YYYY-MM-DD) ")
                .border_type(BorderType::Rounded),
        );
        self.widgets.date.set_cursor_line_style(style);
        self.widgets.date.render(area, buf);
    }

    pub fn from(task: Task) -> Self {
        let description = task.description.lines().map(|s| s.to_string()).collect();
        let date = vec![task.date];
        let title = vec![task.title];
        Self {
            focus: Focus::Title,
            mode: Mode::Normal,
            quit: false,
            completed: false,
            task: Task::from(task.id),
            widgets: Widgets::from(title, date, description),
        }
    }

    fn set_cursor_style(&mut self) {
        let mut cursor_styles = (Style::default(), Style::default(), Style::default());
        if self.mode == Mode::Insert {
            match self.focus {
                Focus::Title => cursor_styles.0 = cursor_styles.0.reversed(),
                Focus::Date => cursor_styles.1 = cursor_styles.1.reversed(),
                Focus::Description => cursor_styles.2 = cursor_styles.2.reversed(),
            }
        }
        self.widgets.title.set_cursor_style(cursor_styles.0);
        self.widgets.date.set_cursor_style(cursor_styles.1);
        self.widgets.description.set_cursor_style(cursor_styles.2);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Tab => self.quit = true,
                KeyCode::Char('i') => self.mode = Mode::Insert,
                KeyCode::Enter => {
                    if self.widgets.date.style() == RED_STYLE {
                        return;
                    }
                    self.mode = Mode::Normal;
                    let title_val = self.widgets.title.lines()[0].to_string();
                    let date_val = self.widgets.date.lines()[0].to_string();
                    let description_val = self.widgets.description.lines().join("\n");
                    self.task = Task {
                        id: self.task.id,
                        title: title_val,
                        date: date_val,
                        description: description_val,
                        completed: false,
                    };
                    self.quit = true;
                    self.completed = true;
                }
                _ => {}
            },
            Mode::Insert => match key.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Tab => self.focus = self.focus.next(),
                KeyCode::BackTab => self.focus = self.focus.previous(),
                _ => match self.focus {
                    Focus::Title => {
                        if key.code != KeyCode::Enter {
                            self.widgets.title.input(key);
                        }
                    }
                    Focus::Date => {
                        if key.code != KeyCode::Enter {
                            self.widgets.date.input(key);
                        }
                    }
                    Focus::Description => {
                        self.widgets.description.input(key);
                    }
                },
            },
        }
    }

    pub fn get_task(&self) -> Task {
        self.task.clone()
    }

    pub fn footer_text(&self) -> Box<[&str]> {
        let footer = match self.mode {
            Mode::Normal => vec!["[Tab] Switch Fields", "[i] Insert Mode", "[Enter] Save"],
            Mode::Insert => vec!["[Esc] Normal Mode", "[Tab] Switch Fields"],
        };
        footer.into_boxed_slice()
    }
}
