use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Flex,
    prelude::*,
    widgets::{Block, BorderType, Clear, Paragraph},
};
use tui_textarea::TextArea;

use crate::app::Todo;

pub struct NewTask<'a> {
    focus: Focus,
    pub mode: Mode,
    title: String,
    pub task: Task<'a>,
    pub completed: bool,
}

pub struct Task<'a> {
    title: TextArea<'a>,
    title_style: Style,
    body: TextArea<'a>,
    body_style: Style,
    pub todo: Todo,
}

#[derive(PartialEq)]
pub(crate) enum Mode {
    Normal,
    Insert,
}

#[derive(PartialEq)]
enum Focus {
    Title,
    Body,
    ConfirmPropmt,
}

const TITLE: &str = " TODO List ";

impl NewTask<'_> {
    pub fn new() -> Self {
        Self {
            focus: Focus::Title,
            mode: Mode::Normal,
            title: TITLE.to_string(),
            completed: false,
            task: Task {
                title: TextArea::default(),
                title_style: Style::default(),
                body: TextArea::default(),
                body_style: Style::default(),
                todo: Todo {
                    title: String::new(),
                    description: String::new(),
                },
            },
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]);
        let [title_area, body_area] = vertical.areas(frame.area());

        let (msg, style) = match self.mode {
            Mode::Normal => (
                vec![
                    " Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "i".bold(),
                    " to start editing. ".bold(),
                ],
                Style::default()
                    .add_modifier(Modifier::SLOW_BLINK)
                    .fg(Color::Reset),
            ),
            Mode::Insert => (
                vec![
                    " Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message. ".into(),
                ],
                Style::default().fg(Color::Reset),
            ),
        };

        let text = Line::from(msg).patch_style(style);

        if self.mode == Mode::Insert {
            match self.focus {
                Focus::Title => {
                    self.task.title_style = Style::default().reversed();
                }
                Focus::Body => {
                    self.task.body_style = Style::default().reversed();
                }
                _ => {}
            }
        }

        self.task.title.set_cursor_style(self.task.title_style);
        self.task.body.set_cursor_style(self.task.body_style);

        self.task.title.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Title "),
        );

        self.task.body.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Body ")
                .title_bottom(text.centered()),
        );

        match self.focus {
            Focus::Title => {
                let block = self
                    .task
                    .title
                    .block()
                    .unwrap()
                    .clone()
                    .border_style(Style::default().light_blue());
                self.task.title.set_block(block);
            }
            Focus::Body => {
                let block = self
                    .task
                    .body
                    .block()
                    .unwrap()
                    .clone()
                    .border_style(Style::default().light_blue());
                self.task.body.set_block(block);
            }
            Focus::ConfirmPropmt => {
                let popup_area = Self::popup_area(frame.area(), 50, 50);
                self.confirm_prompt(frame, popup_area);
            }
        }

        frame.render_widget(&self.task.title, title_area);
        frame.render_widget(&self.task.body, body_area);
    }

    fn confirm_prompt(&self, frame: &mut Frame, area: Rect) {
        let text = Line::from("Do you want to save this todo?")
            .patch_style(Style::default().fg(Color::Reset));
        let block = Block::bordered()
            .title(" Confirm ")
            .border_type(BorderType::Rounded);
        let confirm = Paragraph::new(text).block(block);
        frame.render_widget(Clear, area);
        frame.render_widget(confirm, area);
    }

    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Tab {
            self.focus = match self.focus {
                Focus::Title => {
                    self.task.title_style = Style::default();
                    Focus::Body
                }

                Focus::Body => {
                    self.task.body_style = Style::default();
                    Focus::Title
                }

                Focus::ConfirmPropmt => Focus::ConfirmPropmt,
            };
            return;
        }

        if self.focus == Focus::ConfirmPropmt {
            match key.code {
                KeyCode::Char('y') => {
                    let title_val = self.task.title.lines()[0].to_string();
                    let body_val = self
                        .task
                        .body
                        .lines()
                        .into_iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join("\n");
                    self.task.todo = Todo {
                        title: title_val,
                        description: body_val,
                    };
                    self.completed = true;
                }
                KeyCode::Char('n') => {
                    self.focus = Focus::Title;
                    self.mode = Mode::Normal;
                }
                _ => {}
            }
            return;
        }

        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('i') => {
                    if self.mode == Mode::Normal {
                        self.mode = Mode::Insert;
                        self.title = format!("{}(Insert Mode) ", TITLE);
                    }
                }
                KeyCode::Enter => {
                    self.mode = Mode::Normal;
                    self.focus = Focus::ConfirmPropmt;
                }
                _ => {}
            },
            Mode::Insert => match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                    self.title = TITLE.to_string();
                    self.task.title_style = Style::default();
                    self.task.body_style = Style::default();
                }
                _ => match self.focus {
                    Focus::Title => {
                        if key.code == KeyCode::Enter {}
                        self.task.title.input(key);
                    }
                    Focus::Body => {
                        self.task.body.input(key);
                    }
                    _ => {}
                },
            },
        }
    }
}
