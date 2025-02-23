use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph, Widget, Wrap},
};

use crate::app::{GREEN_STYLE, RED_STYLE};

pub struct Confirm {
    pub title: String,
    pub body: String,
}

impl Widget for &mut Confirm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = crate::helpers::popup_area(area, 30, 25);
        Clear.render(area, buf);
        self.render_prompt(popup_area, buf);
    }
}

impl Confirm {
    pub fn new(title: String, body: String) -> Self {
        Self { title, body }
    }

    fn render_prompt(&self, area: Rect, buf: &mut Buffer) {
        let block = crate::helpers::rounded_block(self.title.as_str())
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
            ]);
        Paragraph::new(self.body.as_str())
            .centered()
            .block(block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
