use std::{
    collections::{HashMap, HashSet},
    fmt::format,
    hash::Hash,
};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Styled, Stylize},
    symbols::{self, Marker},
    text::Text,
    widgets::{
        canvas::{Canvas, Context, Line},
        Block, Borders, Padding, Widget,
    },
};
use wg_2024::network::NodeId;

use crate::{
    network::{node_kind::NodeKind, node_representation::NodeRepresentation},
    screen::{Screen, Window},
    utilities::theme::*,
    Network,
};

use super::draw_options::DrawGraphOptions;

pub fn render_simulation(opt: DrawGraphOptions, area: Rect, buf: &mut Buffer) {
    if opt.nodes.is_empty() {
        Text::from("No nodes in graph").render(area, buf);
        return;
    }
    //TODO redo to avoid panic
    let max_x = opt.nodes.values().map(|n| n.x).reduce(f64::max).unwrap();
    let max_y = opt.nodes.values().map(|n| n.y).reduce(f64::max).unwrap();
    let min_x = opt.nodes.values().map(|n| n.x).reduce(f64::min).unwrap();
    let min_y = opt.nodes.values().map(|n| n.y).reduce(f64::min).unwrap();

    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .paint(|ctx| {
            paint_edges(ctx, &opt);
            print_labels(ctx, &opt);
        })
        .background_color(BG_COLOR)
        .x_bounds([
            min_x as f64 - opt.padding,
            max_x as f64 + (0.01) * (max_x as f64) + opt.padding,
        ])
        .y_bounds([min_y as f64 - opt.padding, max_y as f64 + opt.padding]);

    canvas.render(area, buf);
}

pub fn paint_edges(ctx: &mut Context, opt: &DrawGraphOptions) {
    for (from, to) in opt.lines_back.iter() {
        let nfrom = opt.nodes.get(&from).unwrap();
        let nto = opt.nodes.get(&to).unwrap();
        let line = ratatui::widgets::canvas::Line {
            x1: nfrom.x,
            y1: nfrom.y,
            x2: nto.x,
            y2: nto.y,
            color: opt.back_color,
        };
        ctx.draw(&line);
    }
    ctx.layer();
    for (from, to) in opt.lines_front.iter() {
        let nfrom = opt.nodes.get(&from).unwrap();
        let nto = opt.nodes.get(&to).unwrap();
        let line = ratatui::widgets::canvas::Line {
            x1: nfrom.x,
            y1: nfrom.y,
            x2: nto.x,
            y2: nto.y,
            color: opt.front_color,
        };
        ctx.draw(&line);
    }
}

// TODO this could consume instead of take ref
pub fn print_labels(ctx: &mut Context, opt: &DrawGraphOptions) {
    for (id, n) in opt.nodes.iter() {
        let tx = n.x;
        let ty = n.y;

        let mut s = Style::new().fg(TEXT_COLOR);
        //let c: char;
        //let bl: char;
        //let br: char;
        let label = n.label.clone();

        let line = ratatui::text::Line::styled(label, n.style);
        ctx.print(tx, ty, line);
    }
}
