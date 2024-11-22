use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use crate::{
    model::{node_kind::NodeKind, node_representation::NodeRepresentation, screen::Screen},
    MySimulationController,
};

pub fn handle_crossterm_events(cntrl: &mut MySimulationController) -> Result<(), std::io::Error> {
    match event::read()? {
        // check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_keypress(cntrl, key),
        Event::Mouse(_) => {}
        Event::Resize(_, _) => {}
        _ => {}
    }
    Ok(())
}

/// Handles the key events and updates the state of [`App`].
fn handle_keypress(cntrl: &mut MySimulationController, key: KeyEvent) {
    match cntrl.model.screen {
        Screen::Start => {} //handle_keypress_start(model,key),
        Screen::Main => handle_keypress_main(cntrl, key),
        Screen::Move => handle_keypress_move(cntrl, key),
        Screen::AddNode => handle_keypress_add_node(cntrl, key),
        Screen::AddConnection { origin: from } => handle_keypress_add_connection(cntrl, key, from),
    }
}

// fn handle_keypress_start(model:&mut Model, key: KeyEvent) {
//     match (key.modifiers, key.code) {
//         (_, KeyCode::Char('q')) => model.running = false,
//         // (_, KeyCode::Up) => model.node_list_state.scroll_up_by(1),
//         // (_, KeyCode::Down) => model.node_list_state.scroll_down_by(1),
//         (_, KeyCode::Char('+')) => model.screen = Screen::Main,
//         _ => {}
//     }
// }

fn handle_keypress_main(cntrl: &mut MySimulationController, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => cntrl.model.node_list_state.scroll_up_by(1),
        (_, KeyCode::Down) => cntrl.model.node_list_state.scroll_down_by(1),
        (_, KeyCode::Char('q')) => cntrl.model.running = false,
        (_, KeyCode::Char('m')) => cntrl.model.screen = Screen::Move,
        (_, KeyCode::Char('c')) => {
            cntrl.model.screen = Screen::AddConnection {
                origin: cntrl.model.node_list_state.selected().unwrap() as u32,
            }
        }
        (_, KeyCode::Char('+')) => {
            cntrl.model.nodes.push(NodeRepresentation::default());
            cntrl.model.node_list_state.select_last();
            cntrl.model.screen = Screen::AddNode
        }
        other => match cntrl.model.get_selected_kind() {
            Some(NodeKind::Drone { pdr: _, crashed: _ }) => match other {
                (_, KeyCode::Char('p')) => todo!(),
                (_, KeyCode::Char('k')) => todo!(),
                _ => {}
            },
            _ => {}
        },
    }
}

fn handle_keypress_move(cntrl: &mut MySimulationController, key: KeyEvent) {
    let n = match cntrl.model.node_list_state.selected() {
        None => {
            cntrl.model.screen = Screen::Main;
            return;
        }
        Some(x) => &mut cntrl.model.nodes[x],
    };

    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q')) => cntrl.model.running = false,
        (_, KeyCode::Up) => n.shiftu(1),
        (_, KeyCode::Down) => n.shiftd(1),
        (_, KeyCode::Left) => n.shiftl(1),
        (_, KeyCode::Right) => n.shiftr(1),
        (_, KeyCode::Enter) => cntrl.model.screen = Screen::Main,
        // | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C'))
        // (_,KeyCode::Char('c')) => controller.model.running = false,
        _ => {}
    }
}

fn handle_keypress_add_connection(cntrl: &mut MySimulationController, key: KeyEvent, from: u32) {
    let x = match cntrl.model.node_list_state.selected() {
        None => {
            cntrl.model.screen = Screen::Main;
            return;
        }
        Some(x) => x,
    };

    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q')) => cntrl.model.running = false,
        (_, KeyCode::Up) => cntrl.model.node_list_state.scroll_up_by(1),
        (_, KeyCode::Down) => cntrl.model.node_list_state.scroll_down_by(1),
        (_, KeyCode::Enter) => {
            cntrl.add_connection(from as usize, x);
            cntrl.model.node_list_state.select(Some(from as usize));
            cntrl.model.screen = Screen::Main
        }
        _ => {}
    }
}

fn handle_keypress_add_node(cntrl: &mut MySimulationController, key: KeyEvent) {
    // when you enter add_node screen, the new node gets selected
    let n = match cntrl.model.node_list_state.selected() {
        None => {
            cntrl.model.screen = Screen::Main;
            return;
        }
        Some(x) => &mut cntrl.model.nodes[x],
    };

    match (key.modifiers, key.code) {
        (_, KeyCode::Up) => n.shiftu(1),
        (_, KeyCode::Down) => n.shiftd(1),
        (_, KeyCode::Left) => n.shiftl(1),
        (_, KeyCode::Right) => n.shiftr(1),
        (_, KeyCode::Char('c')) => n.kind = NodeKind::Client,
        (_, KeyCode::Char('s')) => n.kind = NodeKind::Server,
        (_, KeyCode::Char('d')) => {
            n.kind = NodeKind::Drone {
                pdr: 0.5,
                crashed: false,
            }
        } //todo: add way to edit
        (_, KeyCode::Enter) => cntrl.model.screen = Screen::Main,
        (_, KeyCode::Char('q')) => cntrl.model.running = false,
        _ => {}
    }
}
