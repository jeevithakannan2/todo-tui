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

// TODO: Still not perfect center
pub fn popup_fixed_height(area: Rect, percent_x: u16, length_y: u16) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage(50),   // Centering the popup vertically
        Constraint::Length(length_y), // Fixed height of 8
        Constraint::Percentage(50),   // Remaining space
    ])
    .flex(Flex::Center);

    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);

    let [_, area, _] = vertical.areas(area);
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
