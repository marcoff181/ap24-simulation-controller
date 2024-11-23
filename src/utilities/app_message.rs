use wg_2024::network::NodeId;


pub enum AppMessage{
    AddConnection{from:NodeId,to:NodeId},
    Crash{drone:NodeId},
    Quit
}