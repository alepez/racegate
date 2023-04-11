use crate::app::SystemState;
use crate::hal::gate::GateState::{Active, Inactive};
use crate::svc::clock::Instant;

#[derive(Debug)]
pub enum Error {
    Unknown,
}

pub trait RaceNode {
    fn set_system_state(&self, status: &SystemState);

    fn coordinator(&self) -> Option<SystemState>;

    fn publish(&self, msg: RaceNodeMessage) -> anyhow::Result<()>;
}

#[derive(Debug, Copy, Clone)]
pub struct NodeAddress(u8);

impl NodeAddress {
    pub fn coordinator() -> Self {
        Self(0)
    }

    pub fn start() -> Self {
        Self(1)
    }

    pub fn finish() -> Self {
        Self(32)
    }

    pub fn is_coordinator(&self) -> bool {
        self.0 == 0
    }

    pub fn is_start(&self) -> bool {
        self.0 == 1
    }

    pub fn is_finish(&self) -> bool {
        self.0 == 32
    }
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

#[derive(Debug, Copy, Clone)]
pub struct AddressedSystemState {
    pub addr: NodeAddress,
    pub state: SystemState,
}

#[derive(Debug, Copy, Clone)]
pub struct CoordinatorBeacon {
    pub time: Instant,
}

#[derive(Debug)]
pub enum RaceNodeMessage {
    SystemState(AddressedSystemState),
    CoordinatorBeacon(CoordinatorBeacon),
}

impl RaceNodeMessage {
    pub const FRAME_SIZE: usize = 16;

    pub fn data(&self) -> FrameData {
        FrameData::from(self)
    }
}

impl TryFrom<FrameData> for RaceNodeMessage {
    type Error = Error;

    fn try_from(data: FrameData) -> Result<RaceNodeMessage, Error> {
        let msg_id = data.0.first().ok_or(Error::Unknown)?;
        match msg_id {
            1 => Ok(RaceNodeMessage::SystemState(
                AddressedSystemState::try_from(data)?,
            )),
            2 => Ok(RaceNodeMessage::CoordinatorBeacon(
                CoordinatorBeacon::try_from(data)?,
            )),
            _ => Err(Error::Unknown),
        }
    }
}

impl TryFrom<FrameData> for AddressedSystemState {
    type Error = Error;

    fn try_from(data: FrameData) -> Result<AddressedSystemState, Error> {
        let addr = data.0.get(1).ok_or(Error::Unknown)?;
        let addr = NodeAddress(*addr);

        let gate_state = match data.0.get(2) {
            Some(0) => Inactive,
            Some(1) => Active,
            _ => Inactive,
        };

        let time = Instant::from_millis(deserialize_u32(&data, 3).ok_or(Error::Unknown)? as i32);

        let state = SystemState { gate_state, time };

        Ok(AddressedSystemState { addr, state })
    }
}

impl TryFrom<FrameData> for CoordinatorBeacon {
    type Error = Error;

    fn try_from(data: FrameData) -> Result<CoordinatorBeacon, Error> {
        let time = Instant::from_millis(deserialize_u32(&data, 1).ok_or(Error::Unknown)? as i32);

        Ok(CoordinatorBeacon { time })
    }
}

impl From<CoordinatorBeacon> for RaceNodeMessage {
    fn from(x: CoordinatorBeacon) -> Self {
        RaceNodeMessage::CoordinatorBeacon(x)
    }
}

impl From<AddressedSystemState> for RaceNodeMessage {
    fn from(x: AddressedSystemState) -> Self {
        RaceNodeMessage::SystemState(x)
    }
}

fn serialize_system_state(x: &AddressedSystemState, data: &mut FrameData) {
    data.0[1] = x.addr.0;
    data.0[2] = x.state.gate_state as u8;
    serialize_u32(x.state.time.as_millis() as u32, data, 3);
}

fn serialize_coordinator_beacon(x: &CoordinatorBeacon, data: &mut FrameData) {
    serialize_u32(x.time.as_millis() as u32, data, 1);
}

fn serialize_msg_id(msg: &RaceNodeMessage, data: &mut FrameData) {
    let msg_id = match msg {
        RaceNodeMessage::SystemState(_) => 1,
        RaceNodeMessage::CoordinatorBeacon(_) => 2,
    };

    data.0[0] = msg_id;
}

fn serialize_u32(x: u32, data: &mut FrameData, offset: usize) {
    data.0[offset] = ((x >> 24) & 0xFF) as u8;
    data.0[offset + 1] = ((x >> 16) & 0xFF) as u8;
    data.0[offset + 2] = ((x >> 8) & 0xFF) as u8;
    data.0[offset + 3] = ((x) & 0xFF) as u8;
}

fn deserialize_u32(data: &FrameData, offset: usize) -> Option<u32> {
    let d0 = *data.0.get(offset)?;
    let d1 = *data.0.get(offset + 1)?;
    let d2 = *data.0.get(offset + 2)?;
    let d3 = *data.0.get(offset + 3)?;
    Some(((d0 as u32) << 24) | ((d1 as u32) << 16) | ((d2 as u32) << 8) | (d3 as u32))
}

impl From<&RaceNodeMessage> for FrameData {
    fn from(msg: &RaceNodeMessage) -> Self {
        let mut data = FrameData::from([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        serialize_msg_id(msg, &mut data);

        match msg {
            RaceNodeMessage::SystemState(x) => serialize_system_state(x, &mut data),
            RaceNodeMessage::CoordinatorBeacon(x) => serialize_coordinator_beacon(x, &mut data),
        };

        data
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use crate::hal::gate::GateState;

    use super::*;

    #[test]
    fn test_serialize_system_state() {
        let x = AddressedSystemState {
            addr: NodeAddress::start(),
            state: SystemState {
                gate_state: GateState::Active,
                time: Instant::from_millis(12345),
            },
        };

        let msg = RaceNodeMessage::SystemState(x);
        let data = msg.data();

        assert_debug_snapshot!(data.as_bytes());
        assert_debug_snapshot!(RaceNodeMessage::try_from(data).unwrap());
    }

    #[test]
    fn test_serialize_coordinator_beacon() {
        let x = CoordinatorBeacon {
            time: Instant::from_millis(2_123_456_789),
        };

        let msg = RaceNodeMessage::CoordinatorBeacon(x);
        let data = msg.data();

        assert_debug_snapshot!(data.as_bytes());
        assert_debug_snapshot!(RaceNodeMessage::try_from(data).unwrap());
    }
}
