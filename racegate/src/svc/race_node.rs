use crate::app::SystemState;
use crate::hal::gate::GateState::{Active, Inactive};

pub enum Error {
    Unknown,
}

pub trait RaceNode {
    fn set_system_state(&self, status: &SystemState);
}

pub struct FrameData([u8; RaceNodeMessage::FRAME_SIZE]);

impl FrameData {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<[u8; 16]> for FrameData {
    fn from(value: [u8; 16]) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub enum RaceNodeMessage {
    SystemState(SystemState),
}

impl RaceNodeMessage {
    pub const FRAME_SIZE: usize = 16;

    pub fn data(&self) -> FrameData {
        match self {
            RaceNodeMessage::SystemState(x) => x.into(),
        }
    }
}

impl TryFrom<FrameData> for RaceNodeMessage {
    type Error = Error;

    fn try_from(data: FrameData) -> Result<RaceNodeMessage, Error> {
        let msg_id = data.0.first().ok_or(Error::Unknown)?;
        match msg_id {
            1 => Ok(RaceNodeMessage::SystemState(SystemState::try_from(data)?)),
            _ => Err(Error::Unknown),
        }
    }
}

impl TryFrom<FrameData> for SystemState {
    type Error = Error;

    fn try_from(data: FrameData) -> Result<SystemState, Error> {
        let gate_state = match data.0.get(1) {
            Some(0) => Inactive,
            Some(1) => Active,
            _ => Inactive,
        };

        Ok(SystemState { gate_state })
    }
}

impl From<&SystemState> for FrameData {
    fn from(value: &SystemState) -> Self {
        FrameData::from(&RaceNodeMessage::SystemState(*value))
    }
}

impl TryFrom<[u8; 16]> for RaceNodeMessage {
    type Error = Error;

    fn try_from(value: [u8; 16]) -> Result<Self, Self::Error> {
        RaceNodeMessage::try_from(FrameData::from(value))
    }
}

impl From<&RaceNodeMessage> for FrameData {
    fn from(msg: &RaceNodeMessage) -> Self {
        let msg_id = match msg {
            RaceNodeMessage::SystemState(_) => 1,
        };

        let mut data = FrameData::from([msg_id, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        match msg {
            RaceNodeMessage::SystemState(x) => serialize_system_state(x, &mut data),
        };

        data
    }
}

fn serialize_system_state(x: &SystemState, data: &mut FrameData) {
    data.0[1] = x.gate_state as u8;
}
