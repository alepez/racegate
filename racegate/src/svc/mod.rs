pub use clock::{
    calculate_clock_offset, CoordinatedClock, CoordinatedInstant, LocalClock, LocalOffset, LocalInstant,
};
pub use race_node::RaceNode;
pub use std_race_node::StdRaceNode;

use crate::app::SystemState;

mod clock;
pub mod race_node;
mod std_race_node;

pub trait HttpServer {
    fn set_system_state(&self, status: &SystemState);
}
