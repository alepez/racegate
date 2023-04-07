pub use race_node::RaceNode;
pub use race_node::RaceNodeMessage;

use crate::app::SystemState;

mod race_node;

pub trait HttpServer {
    fn set_system_state(&self, status: &SystemState);
}
