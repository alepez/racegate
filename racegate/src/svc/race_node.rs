use crate::app::SystemState;
use crate::hal::gate::GateState::{Active, Inactive};

pub trait RaceNode {
    fn set_system_state(&self, status: &SystemState);
}

pub struct RaceNodeMessage {
    data: FrameData,
}

type FrameData = [u8; RaceNodeMessage::FRAME_SIZE];

impl RaceNodeMessage {
    pub const FRAME_SIZE: usize = 16;

    pub fn data(&self) -> FrameData {
        self.data
    }
}

impl From<FrameData> for RaceNodeMessage {
    fn from(data: FrameData) -> Self {
        RaceNodeMessage { data }
    }
}

impl From<&SystemState> for RaceNodeMessage {
    fn from(system_state: &SystemState) -> Self {
        let data: FrameData = [
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
