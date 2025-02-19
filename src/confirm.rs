use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap},
};

use crate::app::{GREEN_STYLE, RED_STYLE, SECONDARY_STYLE};

pub struct Confirm {
    pub title: String,
    pub body: String,
}

impl Widget for &mut Confirm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = popup_area(area, 30, 25);
        Confirm::render_clear(popup_area, buf);
        self.render_prompt(popup_area, buf);
    }
}

impl Confirm {
    pub fn new(title: String, body: String) -> Self {
        Self { title, body }
    }

    fn render_clear(area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
    }

    fn render_prompt(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(self.title.as_str().bold())
            .title_style(Style::reset())
            .border_type(BorderType::Rounded)
            .border_style(SECONDARY_STYLE)
            // Should separate into two title_bottom for a separation line in between text
            .title_bottom(vec![
                " [ ".into(),
                Span::styled("Y", GREEN_STYLE.bold()),
                " ] ".into(),
            ])
            .title_bottom(vec![
                " [ ".into(),
                Span::styled("N", RED_STYLE.bold()),
                " ] ".into(),
            ])
            .title_alignment(Alignment::Center);
        Paragraph::new(self.body.as_str())
            .centered()
            .block(block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
