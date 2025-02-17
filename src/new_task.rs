use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    style::Styled,
    widgets::{Block, BorderType, Clear, Paragraph, Wrap},
};
use tui_textarea::TextArea;

use crate::app::{popup_area, Todo, SECONDARY_STYLE};

pub struct NewTask<'a> {
    focus: Focus,
    pub mode: Mode,
    pub task: Task<'a>,
    pub quit: bool,
    pub completed: bool,
}

pub struct Task<'a> {
    title: TextArea<'a>,
    description: TextArea<'a>,
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
    Description,
    ConfirmPropmt,
}

impl NewTask<'_> {
    pub fn new() -> Self {
        Self {
            focus: Focus::Title,
            mode: Mode::Normal,
            quit: false,
            completed: false,
            task: Task {
                title: TextArea::default(),
                description: TextArea::default(),
                todo: Todo {
                    title: String::new(),
                    description: String::new(),
                    completed: false,
                },
            },
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);

        let style = if self.mode == Mode::Normal && self.focus != Focus::ConfirmPropmt {
            SECONDARY_STYLE
        } else {
            Style::default()
        };

        let hero_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" New Task ")
            .title_style(Style::default().reset().bold())
            .title_alignment(Alignment::Center)
            .border_style(style);

        frame.render_widget(&hero_block, area);
        let area = hero_block.inner(area);
        let vertical = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]);
        let [title_area, description_area] = vertical.areas(area);

        // cursor_style (title, description)
        let cursor_style = if self.mode == Mode::Insert {
            match self.focus {
                Focus::Title => (Style::default().reversed(), Style::default()),
                Focus::Description => (Style::default(), Style::default().reversed()),
                Focus::ConfirmPropmt => (Style::default(), Style::default()),
            }
        } else {
            (Style::default(), Style::default())
        };

        self.task.title.set_cursor_style(cursor_style.0);
        self.task.description.set_cursor_style(cursor_style.1);

        // Removes the underline when typed
        self.task.title.set_cursor_line_style(Style::default());
        self.task
            .description
            .set_cursor_line_style(Style::default());

        // border_style (title, description)
        let border_style = if self.mode == Mode::Normal {
            (Style::default(), Style::default())
        } else {
            match self.focus {
                Focus::Title => (SECONDARY_STYLE, Style::default()),
                Focus::Description => (Style::default(), SECONDARY_STYLE),
                Focus::ConfirmPropmt => (Style::default(), Style::default()),
            }
        };

        self.task.title.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(border_style.0)
                .title(" Title "),
        );

        self.task.description.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(border_style.1)
                .title(" Description "),
        );

        frame.render_widget(&self.task.title, title_area);
        frame.render_widget(&self.task.description, description_area);

        if self.focus == Focus::ConfirmPropmt {
            let popup_area = popup_area(area, 30, 25);
            self.confirm_prompt(frame, popup_area);
        }
    }

    fn confirm_prompt(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(Line::from(" Confirm Save ").bold().centered())
            .border_type(BorderType::Rounded)
            .border_style(SECONDARY_STYLE)
            .title_bottom(vec![
                " [ ".into(),
                "y".set_style(Style::default().green()),
                " ] ".into(),
            ])
            .title_bottom(vec![
                " [ ".into(),
                "n".set_style(Style::default().red()),
                " ] ".into(),
            ])
            .title_alignment(Alignment::Center)
            .title_style(Style::default().reset());
        let confirm = Paragraph::new(Line::from("Do you want to save this task").centered())
            .wrap(Wrap { trim: true })
            .block(block);
        frame.render_widget(Clear, area);
        frame.render_widget(confirm, area);
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if self.focus == Focus::ConfirmPropmt {
            match key.code {
                KeyCode::Char('y') => {
                    let title_val = self.task.title.lines()[0].to_string();
                    let description_val = self
                        .task
                        .description
                        .lines()
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join("\n");
                    self.task.todo = Todo {
                        title: title_val,
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
                        Focus::Title => Focus::Description,
                        Focus::Description => Focus::Title,
                        Focus::ConfirmPropmt => Focus::ConfirmPropmt,
                    }
                }
                _ => match self.focus {
                    Focus::Title => {
                        if key.code == KeyCode::Enter {
                            return;
                        }
                        self.task.title.input(key);
                    }
                    Focus::Description => {
                        self.task.description.input(key);
                    }
                    _ => {}
                },
            },
        }
    }
}
