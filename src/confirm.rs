use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph, Widget, Wrap},
};

use crate::{
    app::{GREEN_STYLE, PRIMARY_STYLE, RED_STYLE},
    helpers::{create_popup_area, PopupSize},
};

pub struct Confirm {
    pub title: String,
    pub body: String,
}

impl Widget for &mut Confirm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = create_popup_area(area, PopupSize::Percentage { x: 30, y: 25 });
        Clear.render(popup_area, buf);
        self.render_prompt(popup_area, buf);
    }
}

impl Confirm {
    pub fn new(title: String, body: String) -> Self {
        Self { title, body }
    }

    fn render_prompt(&self, area: Rect, buf: &mut Buffer) {
        let block = crate::helpers::rounded_block(self.title.as_str(), PRIMARY_STYLE)
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
