use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    widgets::{Block, Clear, Padding},
};

pub fn center_rect(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

pub fn popup(
    frame: &mut Frame,
    horizontal: Constraint,
    vertical: Constraint,
    background: Color,
    title: &str,
    title_style: Style,
    title_alignment: Alignment,
    border_style: Style,
) -> Rect {
    let area = center_rect(frame.area(), horizontal, vertical);

    let popup = Block::bordered()
        .border_set(border::PROPORTIONAL_WIDE)
        .border_style(border_style)
        .title(title)
        .title_alignment(title_alignment)
        .title_style(title_style);

    let popup_area = popup.inner(area);
    frame.render_widget(popup, area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(Block::new().bg(background), popup_area);

    popup_area
}

pub fn add_padding(area: Rect, padding: Padding) -> Rect {
    Block::new().padding(padding).inner(area)
}
