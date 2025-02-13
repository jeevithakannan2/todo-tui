use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, List, ListItem},
};
use tui_textarea::TextArea;

pub struct App<'a> {
    focus: Focus,
    messages: Vec<(String, String)>,
    mode: Mode,
    title: String,
    entry: Entry<'a>,
}

struct Entry<'a> {
    title: TextArea<'a>,
    title_style: Style,
    body: TextArea<'a>,
    body_style: Style,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}

#[derive(PartialEq)]
enum Focus {
    Title,
    Body,
}

const TITLE: &str = " TODO List ";

impl App<'_> {
    pub fn new() -> Self {
        Self {
            focus: Focus::Title,
            messages: Vec::new(),
            mode: Mode::Normal,
            title: TITLE.to_string(),
            entry: Entry {
                title: TextArea::default(),
                title_style: Style::default(),
                body: TextArea::default(),
                body_style: Style::default(),
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
                Style::default().add_modifier(Modifier::SLOW_BLINK),
            ),
            Mode::Insert => (
                vec![
                    " Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message. ".into(),
                ],
                Style::default(),
            ),
        };

        let text = Line::from(msg).patch_style(style);

        if self.mode == Mode::Insert {
            match self.focus {
                Focus::Title => {
                    self.entry.title_style = Style::default().reversed();
                }
                Focus::Body => {
                    self.entry.body_style = Style::default().reversed();
                }
            }
        }

        self.entry.title.set_cursor_style(self.entry.title_style);
        self.entry.body.set_cursor_style(self.entry.body_style);

        self.entry.title.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Title "),
        );

        self.entry.body.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" Body ")
                .title_bottom(text.centered()),
        );

        frame.render_widget(&self.entry.title, title_area);
        frame.render_widget(&self.entry.body, body_area);

        // let messages: Vec<ListItem> = self
        //     .messages
        //     .iter()
        //     .enumerate()
        //     .map(|(i, m)| {
        //         let content = Line::from(Span::raw(format!("{i}: {0} - {1}", m.0, m.1)));
        //         ListItem::new(content)
        //     })
        //     .collect();
        // let messages = List::new(messages).block(
        //     Block::bordered()
        //         .border_type(BorderType::Rounded)
        //         .title(" Messages ")
        //         .title_bottom(text.centered()),
        // );
        // frame.render_widget(messages, messages_area);
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Tab {
            self.focus = match self.focus {
                Focus::Title => {
                    self.entry.title_style = Style::default();
                    Focus::Body
                }

                Focus::Body => {
                    self.entry.body_style = Style::default();
                    Focus::Title
                }
            };
            return false;
        }

        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('q') => {
                    if self.mode == Mode::Insert {
                        false
                    } else {
                        true
                    }
                }
                KeyCode::Char('i') => {
                    if self.mode == Mode::Normal {
                        self.mode = Mode::Insert;
                        self.title = format!("{}(Insert Mode) ", TITLE);
                    }
                    false
                }
                KeyCode::Enter => {
                    let title_val = self.entry.title.yank_text();
                    // let val = self.entry.body.submit_message().unwrap_or_default();
                    let body_val = self.entry.body.yank_text();
                    self.entry.body.select_all();
                    self.entry.body.cut();
                    self.entry.title.select_all();
                    self.entry.body.cut();
                    self.messages.push((title_val, body_val));
                    self.mode = Mode::Normal;
                    self.focus = Focus::Title;
                    false
                }
                _ => false,
            },
            Mode::Insert => {
                match key.code {
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.title = TITLE.to_string();
                        self.entry.title_style = Style::default();
                        self.entry.body_style = Style::default();
                    }
                    _ => match self.focus {
                        Focus::Title => {
                            if key.code == KeyCode::Enter {
                                return false;
                            }
                            self.entry.title.input(key);
                            return false;
                        }
                        Focus::Body => {
                            self.entry.body.input(key);
                            return false;
                        }
                    },
                }
                false
            }
        }
    }
}
