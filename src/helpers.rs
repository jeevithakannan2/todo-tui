use directories::ProjectDirs;
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, BorderType},
};

pub enum PopupSize {
    Percentage { x: u16, y: u16 },
    Fixed { width: u16, height: u16 },
}

impl PopupSize {
    fn constraints(&self) -> ([Constraint; 1], [Constraint; 1]) {
        match *self {
            PopupSize::Percentage { x, y } => {
                ([Constraint::Percentage(x)], [Constraint::Percentage(y)])
            }
            PopupSize::Fixed { width, height } => {
                ([Constraint::Length(width)], [Constraint::Length(height)])
            }
        }
    }
}

pub fn create_popup_area(area: Rect, size: &PopupSize) -> Rect {
    let (horizontal_constraints, vertical_constraints) = size.constraints();

    let vertical = Layout::vertical(vertical_constraints).flex(Flex::Center);
    let horizontal = Layout::horizontal(horizontal_constraints).flex(Flex::Center);

    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);

    area
}

pub fn rounded_block(title: Line, border_style: Style) -> Block {
    Block::bordered()
        .title(title.reset().bold())
        .border_type(BorderType::Rounded)
        .border_style(border_style)
}

pub fn get_project_dir() -> ProjectDirs {
    ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap()
}
