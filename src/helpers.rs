use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, BorderType},
};

#[allow(dead_code)]
pub enum PopupSize {
    Percentage { x: u16, y: u16 },
    FixedHeight { x: u16, height: u16 },
    FixedWidth { width: u16, y: u16 },
    FixedBoth { width: u16, height: u16 },
}

impl PopupSize {
    fn constraints(&self) -> ([Constraint; 1], [Constraint; 1]) {
        match *self {
            PopupSize::Percentage { x, y } => {
                ([Constraint::Percentage(x)], [Constraint::Percentage(y)])
            }
            PopupSize::FixedHeight { x, height } => {
                ([Constraint::Percentage(x)], [Constraint::Length(height)])
            }
            PopupSize::FixedWidth { width, y } => {
                ([Constraint::Length(width)], [Constraint::Percentage(y)])
            }
            PopupSize::FixedBoth { width, height } => {
                ([Constraint::Length(width)], [Constraint::Length(height)])
            }
        }
    }
}

pub fn create_popup_area(area: Rect, size: PopupSize) -> Rect {
    let (horizontal_constraints, vertical_constraints) = size.constraints();

    let vertical = Layout::vertical(vertical_constraints).flex(Flex::Center);
    let horizontal = Layout::horizontal(horizontal_constraints).flex(Flex::Center);

    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);

    area
}

pub fn rounded_block(title: &str, border_style: Style) -> Block {
    Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .title_style(Style::reset().bold())
        .border_type(BorderType::Rounded)
        .border_style(border_style)
}
