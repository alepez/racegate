use crate::app::gates::Gates;
use crate::svc::CoordinatedInstant;

#[derive(Debug, Default, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Race {
    start_gate_is_active: bool,
    start_gate_activation_time: Option<CoordinatedInstant>,

    finish_gate_is_active: bool,
    finish_gate_activation_time: Option<CoordinatedInstant>,
}

impl Race {
    pub fn set_gates(&mut self, gates: &Gates, now: CoordinatedInstant) {
        if !self.start_gate_is_active && gates.start_gate().active {
            log::info!("start gate activated");
            self.finish_gate_activation_time = Some(now);
        }

        if !self.finish_gate_is_active && gates.finish_gate().active {
            log::info!("finish gate activated");
            self.finish_gate_activation_time = Some(now);
        }
    }
}
