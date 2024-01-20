use ratatui::style::Color;

pub struct Theme {
    pub name_color: Color,
    pub string_color: Color,
    pub number_color: Color,
    pub bool_color: Color,
    pub null_color: Color,
    pub selection_level_indicator_color: Color,
    pub selection_indicator_color: Color,
    pub selection_background_color: Color,
    pub indent_color: Color,
    pub search_indicator_color: Color,
    pub breadcrumbs_color: Color,
    pub status_text_color: Color,
}

pub const THEME: Theme = Theme {
    name_color: Color::White,
    string_color: Color::Yellow,
    number_color: Color::LightBlue,
    bool_color: Color::Cyan,
    null_color: Color::Red,
    selection_level_indicator_color: Color::Cyan,
    selection_indicator_color: Color::Magenta,
    selection_background_color: Color::DarkGray,
    indent_color: Color::DarkGray,
    search_indicator_color: Color::LightYellow,
    breadcrumbs_color: Color::Gray,
    status_text_color: Color::Gray,
};
