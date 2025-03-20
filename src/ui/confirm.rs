use ratatui::{
    prelude::*,
    widgets::{Clear, Paragraph, Widget, Wrap},
};

use crate::{
    helpers::{PopupSize, create_popup_area},
    ui::{GREEN_STYLE, PRIMARY_STYLE, RED_STYLE},
};

pub struct Confirm {
    title: String,
    body: String,
    popup_size: PopupSize,
}

impl Widget for Confirm {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = create_popup_area(area, &self.popup_size);
        Clear.render(popup_area, buf);
        self.render_prompt(popup_area, buf);
    }
}

impl Confirm {
    pub fn new(title: String, body: String, popup_size: PopupSize) -> Self {
        Self {
            title,
            body,
            popup_size,
        }
    }

    fn render_prompt(&self, area: Rect, buf: &mut Buffer) {
        let block = crate::helpers::rounded_block(self.title.as_str(), PRIMARY_STYLE)
            // Should separate into two title_bottom for a separation line in between text
            .title_bottom(vec![
                Span::styled(" [ ", Style::reset()),
                Span::styled("Y", GREEN_STYLE.bold()),
                Span::styled(" ] ", Style::reset()),
            ])
            .title_bottom(vec![
                Span::styled(" [ ", Style::reset()),
                Span::styled("N", RED_STYLE.bold()),
                Span::styled(" ] ", Style::reset()),
            ])
            .title_alignment(Alignment::Center);

        Paragraph::new(self.body.as_str())
            .centered()
            .block(block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
