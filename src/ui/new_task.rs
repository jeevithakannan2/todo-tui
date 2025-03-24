use chrono::{NaiveDate, NaiveTime};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, Clear},
};
use tui_textarea::{CursorMove, TextArea};

use crate::tasks::Task;

use super::RED_STYLE;

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
    time: TextArea<'a>,
    description: TextArea<'a>,
}

#[derive(PartialEq, Clone)]
enum Focus {
    Title,
    Date,
    Time,
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
            Constraint::Length(3),
            Constraint::Min(3),
        ]);
        let [title_area, date_area, time_area, description_area] = vertical.areas(area);

        self.set_cursor_style();
        self.widgets.title.render(title_area, buf);
        self.render_date(date_area, buf);
        self.render_time(time_area, buf);
        self.widgets.description.render(description_area, buf);
    }
}

impl Widgets<'_> {
    pub fn new() -> Self {
        let mut title = TextArea::default();
        let mut date = TextArea::default();
        let mut time = TextArea::default();
        let mut description = TextArea::default();
        let mut widgets = [&mut title, &mut date, &mut time, &mut description];
        Self::setup_widgets(&mut widgets);
        Self {
            title,
            date,
            time,
            description,
        }
    }

    pub fn from(
        title: Vec<String>,
        date: Vec<String>,
        time: Vec<String>,
        description: Vec<String>,
    ) -> Self {
        let mut title = TextArea::new(title);
        let mut date = TextArea::new(date);
        let mut time = TextArea::new(time);
        let mut description = TextArea::new(description);
        let mut widgets = [&mut title, &mut date, &mut time, &mut description];
        Self::setup_widgets(&mut widgets);
        Self {
            title,
            date,
            time,
            description,
        }
    }

    fn setup_widgets(widgets: &mut [&mut TextArea]) {
        // Helper function to create a bordered block
        fn get_block(title: &str) -> Block {
            Block::bordered()
                .title(title)
                .border_type(BorderType::Rounded)
        }

        let titles = [
            (" Title ", "Enter your task title"),
            (" Date - (DD MM YYYY) ", "Enter your task date"),
            (" Time - (HH MM) ", "Enter your estimated completion time"),
            (" Description ", "Enter your task description"),
        ];

        for (widget, (title, placeholder)) in widgets.iter_mut().zip(titles.iter()) {
            widget.set_block(get_block(title));
            widget.set_placeholder_text(*placeholder);
            widget.set_cursor_line_style(Style::default()); // Remove the underline when typing
            widget.move_cursor(CursorMove::End);
        }
    }
}

impl Focus {
    pub fn next(&self) -> Self {
        match self {
            Focus::Title => Focus::Date,
            Focus::Date => Focus::Time,
            Focus::Time => Focus::Description,
            Focus::Description => Focus::Title,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Focus::Title => Focus::Description,
            Focus::Date => Focus::Title,
            Focus::Description => Focus::Time,
            Focus::Time => Focus::Date,
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

    fn render_time(&mut self, area: Rect, buf: &mut Buffer) {
        let time_val = self.widgets.time.lines()[0].to_string();
        let time = NaiveTime::parse_from_str(&time_val, "%H %M");
        let style = match time {
            Ok(_) => Style::default(),
            Err(_) => RED_STYLE,
        };
        self.widgets.time.set_cursor_line_style(style);
        self.widgets.time.render(area, buf);
    }

    fn render_date(&mut self, area: Rect, buf: &mut Buffer) {
        let date_val = self.widgets.date.lines()[0].to_string();
        let date = NaiveDate::parse_from_str(&date_val, "%d %m %Y");
        let style = match date {
            Ok(_) => Style::default(),
            Err(_) => RED_STYLE,
        };
        self.widgets.date.set_cursor_line_style(style);
        self.widgets.date.render(area, buf);
    }

    pub fn from(task: Task) -> Self {
        let description = task.description.lines().map(|s| s.to_string()).collect();
        let date = vec![task.date];
        let time = vec![task.time];
        let title = vec![task.title];
        Self {
            focus: Focus::Title,
            mode: Mode::Normal,
            quit: false,
            completed: false,
            task: Task::from(task.id),
            widgets: Widgets::from(title, date, time, description),
        }
    }

    fn set_cursor_style(&mut self) {
        let mut cursor_styles = [Style::default(); 4];

        if self.mode == Mode::Insert {
            match self.focus {
                Focus::Title => cursor_styles[0] = cursor_styles[0].reversed(),
                Focus::Date => cursor_styles[1] = cursor_styles[1].reversed(),
                Focus::Time => cursor_styles[2] = cursor_styles[2].reversed(),
                Focus::Description => cursor_styles[3] = cursor_styles[3].reversed(),
            }
        }
        self.widgets.title.set_cursor_style(cursor_styles[0]);
        self.widgets.date.set_cursor_style(cursor_styles[1]);
        self.widgets.time.set_cursor_style(cursor_styles[2]);
        self.widgets.description.set_cursor_style(cursor_styles[3]);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Tab => self.quit = true,
                KeyCode::Char('i') => self.mode = Mode::Insert,
                KeyCode::Enter => {
                    if self.widgets.date.cursor_line_style() == RED_STYLE
                        || self.widgets.time.cursor_line_style() == RED_STYLE
                    {
                        return;
                    }
                    self.mode = Mode::Normal;
                    self.task = Task {
                        id: self.task.id,
                        title: self.widgets.title.lines()[0].to_string(),
                        date: self.widgets.date.lines()[0].to_string(),
                        time: self.widgets.time.lines()[0].to_string(),
                        description: self.widgets.description.lines().join("\n"),
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
                    Focus::Time => {
                        if key.code != KeyCode::Enter {
                            self.widgets.time.input(key);
                        }
                    }
                    Focus::Description => {
                        self.widgets.description.input(key);
                    }
                },
            },
        }
    }

    pub fn get_task(&self) -> &Task {
        &self.task
    }

    pub fn footer_text(&self) -> Box<[&str]> {
        let footer = match self.mode {
            Mode::Normal => vec!["[Tab] Switch Fields", "[i] Insert Mode", "[Enter] Save"],
            Mode::Insert => vec!["[Esc] Normal Mode", "[Tab] Switch Fields"],
        };
        footer.into_boxed_slice()
    }
}
