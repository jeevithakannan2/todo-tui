use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, BorderType},
};

use crate::app::SECONDARY_STYLE;

pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

pub fn rounded_block(title: &str) -> Block {
    Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .title_style(Style::reset().bold())
        .border_type(BorderType::Rounded)
        .border_style(SECONDARY_STYLE)
}
