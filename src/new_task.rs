use crate::{app::SECONDARY_STYLE, handle_json::Todo};
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
    description: TextArea<'a>,
}

#[derive(PartialEq)]
enum Focus {
    Title,
    Description,
    ConfirmPropmt,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}

impl Widgets<'_> {
    pub fn new() -> Self {
        let mut title = TextArea::default();
        let mut description = TextArea::default();
        // Removes the underline when typed
        title.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Title "),
        );

        description.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Description "),
        );
        title.set_cursor_line_style(Style::default());
        description.set_cursor_line_style(Style::default());
        Self { title, description }
    }

    pub fn from(title: Vec<String>, description: Vec<String>) -> Self {
        let mut title = TextArea::new(title);
        let mut description = TextArea::new(description);
        title.set_cursor_line_style(Style::default());
        description.set_cursor_line_style(Style::default());
        title.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Title "),
        );

        description.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Description "),
        );
        title.set_cursor_line_style(Style::default());
        description.set_cursor_line_style(Style::default());
        Self { title, description }
    }

    pub fn set_cursor_style(&mut self, mode: &Mode, focus: &Focus) {
        // cursor_style (title, description)
        let cursor_style = if *mode == Mode::Insert {
            match focus {
                Focus::Title => (Style::default().reversed(), Style::default()),
                Focus::Description => (Style::default(), Style::default().reversed()),
                Focus::ConfirmPropmt => (Style::default(), Style::default()),
            }
        } else {
            (Style::default(), Style::default())
        };

        self.title.set_cursor_style(cursor_style.0);
        self.description.set_cursor_style(cursor_style.1);
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
        let title = vec![todo.title];
        Self {
            focus: Focus::Title,
            mode: Mode::Normal,
            quit: false,
            completed: false,
            todo: Todo::from(Some(todo.id), None, None, None),
            widgets: Widgets::from(title, description),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);

        let hero_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" New Task ")
            .title_style(Style::default().reset().bold())
            .title_alignment(Alignment::Center)
            .border_style(SECONDARY_STYLE);

        frame.render_widget(&hero_block, area);
        let area = hero_block.inner(area);
        let vertical = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]);
        let [title_area, description_area] = vertical.areas(area);

        self.widgets.set_cursor_style(&self.mode, &self.focus);

        frame.render_widget(&self.widgets.title, title_area);
        frame.render_widget(&self.widgets.description, description_area);

        if self.focus == Focus::ConfirmPropmt {
            crate::app::confirm_prompt(
                frame,
                area,
                " Confirm Save ",
                "Do you want to save this task",
            );
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if self.focus == Focus::ConfirmPropmt {
            match key.code {
                KeyCode::Char('y') => {
                    let title_val = self.widgets.title.lines()[0].to_string();
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
                        self.widgets.title.input(key);
                    }
                    Focus::Description => {
                        self.widgets.description.input(key);
                    }
                    _ => {}
                },
            },
        }
    }

    pub fn footer_text(&self) -> &str {
        match self.mode {
            Mode::Normal => match self.focus {
                Focus::Description | Focus::Title => {
                    "[q] Quit without saving | [i] Insert Mode | [Enter] Save"
                }
                Focus::ConfirmPropmt => "[y] Yes | [n] No",
            },
            Mode::Insert => "[Esc] Normal Mode | [Tab] Switch Fields",
        }
    }
}
