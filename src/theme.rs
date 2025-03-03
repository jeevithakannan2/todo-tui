#[derive(PartialEq)]
pub enum Theme {
    Default,
    Compatible,
}

impl Theme {
    pub fn get_completed(&self) -> &str {
        match self {
            Theme::Default => "󰄴",
            Theme::Compatible => "[x]",
        }
    }

    pub fn get_delete(&self) -> &str {
        match self {
            Theme::Default => "󰄰",
            Theme::Compatible => "[-]",
        }
    }

    pub fn get_uncompleted(&self) -> &str {
        match self {
            Theme::Default => "󰄰",
            Theme::Compatible => "[ ]",
        }
    }

    pub fn change_theme(&self) -> Self {
        match self {
            Theme::Default => Theme::Compatible,
            Theme::Compatible => Theme::Default,
        }
    }
}
