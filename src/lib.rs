use std::collections::HashMap;

use app::App;

mod app;

use crossbeam_channel::{Receiver, Sender};
use wg_2024::{config::Config, controller::Command, network::NodeId};

use network_initializer::unofficial_wg_implementations::{
    SimControllerOptions, SimulationController,
};

pub struct MySimulationController {
    command_send: HashMap<NodeId, Sender<Command>>,
    command_recv: Receiver<Command>,
    config: Config, // the simulation controller should know more than NodeKind: PDR for drone
}

impl SimulationController for MySimulationController {
    fn new(opt: SimControllerOptions) -> Self {
        MySimulationController {
            command_send: opt.command_send,
            command_recv: opt.command_recv,
            config: opt.config,
        }
    }

    // could return Result and then thread handler in network intializer handles the Error
    fn run(&mut self) {
        let terminal = ratatui::init();
        let _result = App::new(self).run(terminal);
        ratatui::restore();
    }
}

impl MySimulationController {
    // handle commands from drone

    // functions that get called from App and send commands to corresponding drone
}
