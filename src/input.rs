use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, Paragraph},
};

use unicode_width::UnicodeWidthChar;

pub struct Input {
    input_position: usize,
    input: Vec<char>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            input_position: 0,
            input: Vec::new(),
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let cursor_position: u16 = self.input[..self.input_position]
            .iter()
            .map(|c| c.width().unwrap_or(1) as u16)
            .sum();

        let x = area.x + cursor_position + 1;
        let y = area.y + 1;
        frame.set_cursor_position(Position::new(x, y));
        let input = Paragraph::new(self.input.iter().collect::<String>())
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Backspace => self.remove_previous(),
            KeyCode::Delete => self.remove_next(),
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Left => self.cursor_left(),
            KeyCode::Right => self.cursor_right(),
            _ => (),
        };
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

    pub fn submit_message(&mut self) -> Option<String> {
        let input: String = self.input.clone().into_iter().collect();
        self.input.clear();
        self.reset_cursor();
        if input.is_empty() {
            None
        } else {
            Some(input)
        }
    }
}
