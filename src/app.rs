use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::input;

pub struct App {
    messages: Vec<String>,
    mode: Mode,
    title: String,
    inp: input::Input,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
}

const TITLE: &str = " TODO List ";

impl App {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            mode: Mode::Normal,
            title: TITLE.to_string(),
            inp: input::Input::new(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]);
        let [input_area, messages_area] = vertical.areas(frame.area());

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

        self.inp.draw(frame, input_area);

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(
            Block::bordered()
                .title("Messages")
                .title_bottom(text.centered()),
        );
        frame.render_widget(messages, messages_area);
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
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
                _ => false,
            },
            Mode::Insert => {
                match key.code {
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.title = TITLE.to_string();
                    }
                    KeyCode::Enter => {
                        if let Some(inp) = self.inp.submit_message() {
                            self.messages.push(inp);
                        }
                    }
                    _ => {
                        self.inp.handle_key(key);
                    }
                }

                false
            }
        }
    }
}
