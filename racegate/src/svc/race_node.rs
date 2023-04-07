use crate::app::SystemState;
use crate::hal::gate::GateState::{Active, Inactive};

pub trait RaceNode {
    fn set_system_state(&self, status: &SystemState);
}

pub struct RaceNodeMessage {
    data: [u8; 16],
}

impl RaceNodeMessage {
    pub fn data(&self) -> [u8; 16] {
        self.data
    }
}

impl From<[u8; 16]> for RaceNodeMessage {
    fn from(data: [u8; 16]) -> Self {
        RaceNodeMessage { data }
    }
}

impl From<&SystemState> for RaceNodeMessage {
    fn from(system_state: &SystemState) -> Self {
        let data: [u8; 16] = [
            0, // reserved, id
            system_state.gate_state as u8,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        RaceNodeMessage { data }
    }
}

impl From<&RaceNodeMessage> for SystemState {
    fn from(msg: &RaceNodeMessage) -> Self {
        let gate_state = match msg.data[1] {
            0 => Inactive,
            1 => Active,
            _ => Inactive,
        };

        SystemState { gate_state }
    }
}
