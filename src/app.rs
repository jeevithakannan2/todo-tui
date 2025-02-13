use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph},
};

use unicode_width::UnicodeWidthChar;

pub struct App {
    input_position: usize,
    input: Vec<char>,
    messages: Vec<String>,
    mode: Mode,
    title: String,
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
            input_position: 0,
            input: Vec::new(),
            messages: Vec::new(),
            mode: Mode::Normal,
            title: TITLE.to_string(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = vertical.areas(frame.area());

        let (msg, style) = match self.mode {
            Mode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "i".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::SLOW_BLINK),
            ),
            Mode::Insert => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let cursor_position: u16 = self.input[..self.input_position]
            .iter()
            .map(|c| c.width().unwrap_or(1) as u16)
            .sum();

        let x = input_area.x + cursor_position + 1;
        let y = input_area.y + 1;
        frame.set_cursor_position(Position::new(x, y));
        let input = Paragraph::new(self.input.iter().collect::<String>())
            .style(match self.mode {
                Mode::Normal => Style::default(),
                Mode::Insert => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Messages"));
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
                    KeyCode::Backspace => self.remove_previous(),
                    KeyCode::Delete => self.remove_next(),
                    KeyCode::Enter => self.submit_message(),
                    KeyCode::Char(c) => self.insert_char(c),
                    KeyCode::Left => self.cursor_left(),
                    KeyCode::Right => self.cursor_right(),
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.title = TITLE.to_string();
                    }
                    _ => (),
                };
                false
            }
        }
    }

    fn cursor_left(&mut self) {
        self.input_position = self.input_position.saturating_sub(1);
    }

    fn cursor_right(&mut self) {
        if self.input_position < self.input.len() {
            self.input_position += 1;
        }
    }

    fn insert_char(&mut self, input: char) {
        self.input.insert(self.input_position, input);
        self.cursor_right();
    }

    fn remove_previous(&mut self) {
        let current = self.input_position;
        if current > 0 {
            self.input.remove(current - 1);
            self.cursor_left();
        }
    }

    fn remove_next(&mut self) {
        let current = self.input_position;
        if current < self.input.len() {
            self.input.remove(current);
        }
    }

    fn reset_cursor(&mut self) {
        self.input_position = 0;
    }

    fn submit_message(&mut self) {
        self.messages.push(self.input.clone().into_iter().collect());
        self.input.clear();
        self.reset_cursor();
    }
}
