use std::collections::HashSet;

use ratatui::{
    buffer::Buffer, layout::Rect, style::{Color, Stylize}, symbols::{self, Marker}, text::Line, widgets::{canvas::Canvas, Block, Borders, Padding, Widget}
};

use crate::{model::{node_kind::NodeKind, node_representation::NodeRepresentation, screen::Screen}, utilities::theme::*, Model};

pub fn render_simulation(model:&Model, area: Rect, buf: &mut Buffer) {
    let nodes = model.nodes;

    let top_right_border_set = symbols::border::Set {
        top_left: symbols::line::NORMAL.horizontal_down,
        ..symbols::border::PLAIN
    };

    let block = Block::new()
        .border_set(top_right_border_set)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title("Simulation")
        .bg(BG_COLOR)
        .fg(TEXT_COLOR)
        .padding(Padding::proportional(1));

    let inner_area = block.inner(area);

    // redo to avoid panic
    let max_x = nodes.iter().map(|n| n.x).max().unwrap();
    let max_y = nodes.iter().map(|n| n.y).max().unwrap();
    let min_x = nodes.iter().map(|n| n.x).min().unwrap();
    let min_y = nodes.iter().map(|n| n.y).min().unwrap();

    let scale_x = inner_area.width as f64 / max_x as f64;
    let scale_y = inner_area.height as f64 / max_y as f64;

    let canvas_border_offset: f64 = 1.0;

    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .paint(|ctx| {
            let mut checked: HashSet<&NodeRepresentation> = HashSet::new();

            for (p1, n1) in model.nodes.iter().enumerate() {
                checked.insert(&n1);
                for (p2, n2) in nodes.iter().enumerate() {
                    if !checked.contains(&n2) && n1.adj.contains(&(p2 as u32)) {
                        let mut c: Color = Color::DarkGray;
                        // if let Some(selected_index) = node_list_state.selected() {
                        //     if (selected_index == p1 || selected_index == p2) {
                        //         c = HIGHLIGHT_COLOR;
                        //     }
                        // }

                        let line = ratatui::widgets::canvas::Line {
                            x1: (n1.x as f64),
                            y1: (n1.y as f64),
                            x2: (n2.x as f64),
                            y2: (n2.y as f64),
                            color: c,
                        };
                        ctx.draw(&line);
                    }
                }
            }

            ctx.layer();

            match screen {
                Screen::Start => {
                    todo!()
                }
                // Highlight edges that connect selected node
                Screen::Main | Screen::Move | Screen::AddNode => {
                    if let Some(id1) = node_list_state.selected() {
                        let n1 = &nodes[id1];
                        for (p2, n2) in nodes.iter().enumerate() {
                            if n1.adj.contains(&(p2 as u32)) {
                                let line = ratatui::widgets::canvas::Line {
                                    x1: (n1.x as f64),
                                    y1: (n1.y as f64),
                                    x2: (n2.x as f64),
                                    y2: (n2.y as f64),
                                    color: HIGHLIGHT_COLOR,
                                };
                                ctx.draw(&line);
                            }
                        }
                    }
                }
                Screen::AddConnection { origin: o } => {
                    if let Some(id1) = node_list_state.selected() {
                        if (o as usize) < nodes.len() {
                            let n1 = &nodes[id1];
                            let n2 = &nodes[o as usize];

                            let line = ratatui::widgets::canvas::Line {
                                x1: (n1.x as f64),
                                y1: (n1.y as f64),
                                x2: (n2.x as f64),
                                y2: (n2.y as f64),
                                color: Color::Green,
                            };
                            ctx.draw(&line);
                        }
                    }
                }
            }

            for (pos, n) in nodes.iter().enumerate() {
                let tx = (n.x as f64);
                let ty = (n.y as f64);

                let mut s = Style::new().fg(TEXT_COLOR);
                let mut c: char;
                let mut bl: char;
                let mut br: char;
                match n.kind {
                    NodeKind::Drone => {
                        s = s.bg(DRONE_COLOR);
                        c = 'D';
                        bl = '(';
                        br = ')';
                    }
                    NodeKind::Client => {
                        s = s.bg(CLIENT_COLOR);
                        c = 'C';
                        bl = '[';
                        br = ']';
                    }
                    NodeKind::Server => {
                        s = s.bg(SERVER_COLOR);
                        c = 'S';
                        bl = '[';
                        br = ']';
                    }
                }

                if let Some(selected_index) = node_list_state.selected() {
                    match screen {
                        Screen::Start => todo!(),
                        // highlight selected node
                        Screen::Main | Screen::Move => {
                            if (selected_index == pos) {
                                s = s.bg(HIGHLIGHT_COLOR);
                                s = s.fg(BG_COLOR);
                                s = s.bold();
                            }
                        }
                        // highlight node from which connection starts
                        // and highlight green selected ndoe for destination
                        Screen::AddConnection { origin: o } => {
                            if (selected_index == pos) {
                                s = s.bg(Color::Green);
                                //s = s.fg(BG_COLOR);
                                s = s.bold();
                            }
                            if (pos == o as usize) {
                                s = s.bg(HIGHLIGHT_COLOR);
                                s = s.fg(BG_COLOR);
                                s = s.bold();
                            }
                        }
                        // highlight green the new node
                        Screen::AddNode => {
                            if (selected_index == pos) {
                                s = s.bg(Color::Green);
                                s = s.bold();
                            }
                        }
                    }
                }

                ctx.print(tx, ty, Line::styled(format!("{}{}{}", bl, c, br), s));
            }
        })
        .background_color(BG_COLOR)
        .x_bounds([min_x as f64, max_x as f64 + (3.0) / (max_x as f64)])
        .y_bounds([min_y as f64, max_y as f64]);

    block.render(area, buf);
    canvas.render(inner_area, buf);
}