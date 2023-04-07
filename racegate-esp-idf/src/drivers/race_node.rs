use racegate::app::SystemState;
use racegate::svc::RaceNode;

pub struct EspRaceNode;

impl EspRaceNode {
    pub fn new() -> anyhow::Result<Self> {
        Ok(EspRaceNode)
    }
}

impl RaceNode for EspRaceNode {
    fn set_system_state(&self, _status: &SystemState) {
        // TODO
    }
}
