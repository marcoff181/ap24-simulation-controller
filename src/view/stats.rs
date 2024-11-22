pub fn render_stats(model:&Model, area: Rect, buf: &mut Buffer) {
    let collapsed_top_and_left_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.vertical_right,
        top_right: symbols::line::NORMAL.vertical_left,
        bottom_left: symbols::line::NORMAL.horizontal_up,
        ..symbols::border::PLAIN
    };

    // let bottom_right_block =
    Block::new()
        .border_set(collapsed_top_and_left_border_set)
        .borders(Borders::ALL)
        .title("Stats")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR)
        .render(area, buf);
}