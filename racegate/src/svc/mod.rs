pub use clock::{RaceClock, RaceInstant};
pub use race_node::RaceNode;
pub use race_node::RaceNodeMessage;
pub use std_race_node::StdRaceNode;

use crate::app::SystemState;

mod clock;
mod race_node;
mod std_race_node;

pub trait HttpServer {
    fn set_system_state(&self, status: &SystemState);
}
