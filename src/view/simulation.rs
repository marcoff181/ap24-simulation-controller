use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::{self, Marker},
    widgets::{
        canvas::{Canvas, Context, Line},
        Block, Borders, Padding, Widget,
    },
};

use crate::{
    network::{node_kind::NodeKind, node_representation::NodeRepresentation},
    screen::{Screen, Window},
    utilities::theme::*,
    Network,
};

pub fn render_simulation(network: &Network, screen: &Screen, area: Rect, buf: &mut Buffer) {
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
    let max_x = network.nodes.iter().map(|n| n.x).max().unwrap();
    let max_y = network.nodes.iter().map(|n| n.y).max().unwrap();
    let min_x = network.nodes.iter().map(|n| n.x).min().unwrap();
    let min_y = network.nodes.iter().map(|n| n.y).min().unwrap();

    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .paint(|ctx| simulation_painter(ctx, network, screen))
        .background_color(BG_COLOR)
        .x_bounds([min_x as f64, max_x as f64 + (0.01) * (max_x as f64)])
        .y_bounds([min_y as f64, max_y as f64]);

    block.render(area, buf);
    canvas.render(inner_area, buf);
}

fn simulation_painter(ctx: &mut Context, network: &Network, screen: &Screen) {
    paint_edges(ctx, network, screen);
    print_labels(ctx, network, screen);
}

fn paint_edges(ctx: &mut Context, network: &Network, screen: &Screen) {
    //let checked: HashSet<&NodeRepresentation> = HashSet::new();

    let id = screen.focus;

    // keep them in a buffer so that we can draw the highlights on top of the others
    let mut lines_back: Vec<Line> = Vec::new();
    let mut lines_front: Vec<Line> = Vec::new();

    for (from, to) in network.edges.iter() {
        let mut c: Color = TEXT_COLOR;

        let mut is_line_front = false;

        if *from == id || *to == id {
            match screen.window {
                Window::Main
                | Window::Move
                | Window::AddNode { toadd: _ }
                | Window::ChangePdr { pdr: _ } => {
                    c = HIGHLIGHT_COLOR;
                    is_line_front = true
                }
                Window::AddConnection { origin: _ } => {
                    c = TEXT_COLOR;
                    is_line_front = false;
                }
                Window::Detail => unreachable!("this should have been filtered out earlier"),
            }
        };

        let opt1 = network.get_node_from_id(*from);
        let opt2 = network.get_node_from_id(*to);

        if let (Some(n1), Some(n2)) = (opt1, opt2) {
            let line = ratatui::widgets::canvas::Line {
                x1: n1.x as f64,
                y1: n1.y as f64,
                x2: n2.x as f64,
                y2: n2.y as f64,
                color: c,
            };

            if is_line_front {
                lines_front.push(line);
            } else {
                lines_back.push(line);
            }
        }
    }

    for line in lines_back {
        ctx.draw(&line);
    }

    ctx.layer();

    for line in lines_front {
        ctx.draw(&line);
    }

    if let Window::AddConnection { origin } = screen.window {
        if origin != id {
            let line = ratatui::widgets::canvas::Line {
                x1: network.get_node_from_id(id).unwrap().x as f64,
                y1: network.get_node_from_id(id).unwrap().y as f64,
                x2: network.get_node_from_id(origin).unwrap().x as f64,
                y2: network.get_node_from_id(origin).unwrap().y as f64,
                color: ADD_EDGE_COLOR,
            };
            ctx.layer();
            ctx.draw(&line);
        }
    }
}

fn print_labels(ctx: &mut Context, model: &Network, screen: &Screen) {
    for n in model.nodes.iter() {
        let tx = n.x as f64;
        let ty = n.y as f64;

        let mut s = Style::new().fg(TEXT_COLOR);
        let c: char;
        let bl: char;
        let br: char;
        match n.kind {
            NodeKind::Drone { pdr: _, crashed } => {
                if crashed {
                    s = s.bg(CRASH_COLOR);
                    c = 'X';
                } else {
                    s = s.bg(DRONE_COLOR);
                    c = 'D';
                }

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

        // special coloring
        let selected_index = screen.focus;
        match screen.window {
            // highlight selected node
            Window::Main | Window::Move | Window::ChangePdr { pdr: _ } => {
                if selected_index == n.id {
                    s = s.bg(HIGHLIGHT_COLOR);
                    s = s.fg(BG_COLOR);
                    s = s.bold();
                }
            }
            // highlight node from which connection starts
            // and highlight green selected node for destination
            Window::AddConnection { origin } => {
                if n.id == origin {
                    s = s.bg(HIGHLIGHT_COLOR);
                    s = s.fg(BG_COLOR);
                    s = s.bold();
                } else if selected_index == n.id {
                    s = s.bg(Color::Green);
                    //s = s.fg(BG_COLOR);
                    s = s.bold();
                }
            }
            // highlight green the new node
            Window::AddNode { toadd: _ } => {
                if selected_index == n.id {
                    s = s.bg(Color::Green);
                    s = s.bold();
                }
            }
            Window::Detail => unreachable!(),
        }

        ctx.print(
            tx,
            ty,
            ratatui::text::Line::styled(format!("{}{}{}", bl, c, br), s),
        );
    }
}
