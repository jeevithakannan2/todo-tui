use crate::{
    app::{popup_area, SECONDARY_STYLE},
    handle_json::Todo,
};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    style::Styled,
    widgets::{Block, BorderType, Clear, Paragraph, Wrap},
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
            todo: Todo {
                title: String::new(),
                description: String::new(),
                completed: false,
            },
            widgets: Widgets::new(),
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
                "Y".set_style(Style::default().green().bold()),
                " ] ".into(),
            ])
            .title_bottom(vec![
                " [ ".into(),
                "N".set_style(Style::default().red().bold()),
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
}
