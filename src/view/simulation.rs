use std::collections::HashSet;

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
    model::{node_kind::NodeKind, node_representation::NodeRepresentation, screen::Screen},
    utilities::theme::*,
    Model,
};

pub fn render_simulation(model: &Model, area: Rect, buf: &mut Buffer) {
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
    let max_x = model.nodes.iter().map(|n| n.x).max().unwrap();
    let max_y = model.nodes.iter().map(|n| n.y).max().unwrap();
    let min_x = model.nodes.iter().map(|n| n.x).min().unwrap();
    let min_y = model.nodes.iter().map(|n| n.y).min().unwrap();

    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .paint(|ctx| simulation_painter(ctx, model))
        .background_color(BG_COLOR)
        .x_bounds([min_x as f64, max_x as f64 + (0.01)*(max_x as f64)])
        .y_bounds([min_y as f64, max_y as f64]);

    block.render(area, buf);
    canvas.render(inner_area, buf);
}

fn simulation_painter(ctx: &mut Context, model: &Model) {
    paint_edges(ctx, model);
    print_labels(ctx, model);
}

fn paint_edges(ctx: &mut Context, model: &Model) {
    let checked: HashSet<&NodeRepresentation> = HashSet::new();

    let selected = model.selected_node_id();

    // keep them in a buffer so that we can draw the highlights on top of the others
    let mut lines_back: Vec<Line> = Vec::new();
    let mut lines_front: Vec<Line> = Vec::new();

    for (from, to) in model.edges.iter() {
        let mut c: Color = TEXT_COLOR;

        let mut is_line_front = false;

        
        if let Some(id) = selected { 
        if  *from == id || *to == id {
            match model.screen {
                Screen::Start => todo!(),
                Screen::Main | Screen::Move | Screen::AddNode => {c =HIGHLIGHT_COLOR;is_line_front=true},
                Screen::AddConnection { origin } => {
                    c = TEXT_COLOR; is_line_front = false;
                }
            }
        };
        }

        let opt1 = model.get_node_from_id(*from);
        let opt2 = model.get_node_from_id(*to);

        if let (Some(n1),Some(n2))= (opt1,opt2){
            let line = ratatui::widgets::canvas::Line {
                x1: n1.x as f64,
                y1: n1.y as f64,
                x2: n2.x as f64,
                y2: n2.y as f64,
                color: c,
            };

            if is_line_front{
                lines_front.push(line);
            }
            else{
                lines_back.push(line);
            }
        }             
    }

    for line in lines_back{
        ctx.draw(&line);
    }

    ctx.layer();

    for line in lines_front{
        ctx.draw(&line);
    }

    if let Screen::AddConnection{origin} = model.screen {
        if let Some(id) = selected{
            if  origin != id {
                let line = ratatui::widgets::canvas::Line {
                    x1: model.get_node_from_id(id).unwrap().x as f64,
                    y1: model.get_node_from_id(id).unwrap().y as f64,
                    x2: model.get_node_from_id(origin).unwrap().x as f64,
                    y2: model.get_node_from_id(origin).unwrap().y as f64,
                    color: ADD_EDGE_COLOR,
                };
                ctx.layer();
                ctx.draw(&line);
            } 
        }
        
    }
}

fn print_labels(ctx: &mut Context, model: &Model) {
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
        if let Some(selected_index) = model.selected_node_id() {
            match model.screen {
                Screen::Start => todo!(),
                // highlight selected node
                Screen::Main | Screen::Move => {
                    if selected_index == n.id {
                        s = s.bg(HIGHLIGHT_COLOR);
                        s = s.fg(BG_COLOR);
                        s = s.bold();
                    }
                }
                // highlight node from which connection starts
                // and highlight green selected node for destination
                Screen::AddConnection { origin } => {
                    if n.id  == origin {
                        s = s.bg(HIGHLIGHT_COLOR);
                        s = s.fg(BG_COLOR);
                        s = s.bold();
                    }
                    else if selected_index == n.id {
                        s = s.bg(Color::Green);
                        //s = s.fg(BG_COLOR);
                        s = s.bold();
                    }
                }
                // highlight green the new node
                Screen::AddNode => {
                    if selected_index == n.id {
                        s = s.bg(Color::Green);
                        s = s.bold();
                    }
                }
            }
        }

        ctx.print(tx, ty, ratatui::text::Line::styled(format!("{}{}{}", bl, c, br), s));
    }
}
