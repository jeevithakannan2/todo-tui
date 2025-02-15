use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    style::Styled,
    widgets::{Block, BorderType, Clear, Paragraph},
};
use tui_textarea::TextArea;

use crate::app::{popup_area, Todo};

pub struct NewTask<'a> {
    focus: Focus,
    pub mode: Mode,
    title: String,
    pub task: Task<'a>,
    pub quit: bool,
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
            quit: false,
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

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);

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

        let text = Line::from(msg).centered().patch_style(style);
        let style = if self.mode == Mode::Normal && self.focus != Focus::ConfirmPropmt {
            Style::default().light_blue()
        } else {
            Style::default()
        };

        let main_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(Span::styled(" New Task ", Style::default().bold()).into_centered_line())
            .title_bottom(text)
            .border_style(style);

        frame.render_widget(&main_block, area);
        let area = main_block.inner(area);
        let vertical = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]);
        let [title_area, body_area] = vertical.areas(area);

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
                .title(" Body "),
        );

        let style = match self.mode {
            Mode::Normal => Style::default(),
            Mode::Insert => Style::default().light_blue(),
        };

        match self.focus {
            Focus::Title => {
                let block = self.task.title.block().unwrap().clone().border_style(style);
                self.task.title.set_block(block);
            }
            Focus::Body => {
                let block = self.task.body.block().unwrap().clone().border_style(style);
                self.task.body.set_block(block);
            }
            Focus::ConfirmPropmt => {
                let popup_area = popup_area(frame.area(), 25, 25);
                self.confirm_prompt(frame, popup_area);
            }
        }
        frame.render_widget(&self.task.title, title_area);
        frame.render_widget(&self.task.body, body_area);
    }

    fn confirm_prompt(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            " Press [".into(),
            "y".set_style(Style::default().green()),
            "] to confirm or [".into(),
            "n".set_style(Style::default().red()),
            "] to cancel ".into(),
        ];

        let text = Line::from(text).centered();
        let block = Block::bordered()
            .title(Line::from(" Confirm Save ").bold().centered())
            .border_type(BorderType::Rounded)
            .border_style(Style::default().light_blue())
            .title_bottom(text)
            .title_style(Style::default().reset());
        let confirm =
            Paragraph::new(Line::from("Do you want to save this task").centered()).block(block);
        frame.render_widget(Clear, area);
        frame.render_widget(confirm, area);
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
                    self.task.title = TextArea::default();
                    self.task.body = TextArea::default();
                    self.quit = true;
                    self.completed = true;
                    self.focus = Focus::Title;
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
                KeyCode::Char('q') => {
                    self.focus = Focus::Title;
                    self.quit = true;
                }
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
