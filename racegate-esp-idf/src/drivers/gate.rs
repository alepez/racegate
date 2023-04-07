use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver};

use racegate::hal::gate::{Gate, GateState};

pub struct EspGate {
    input: PinDriver<'static, AnyInputPin, Input>,
}

impl EspGate {
    pub fn new(pin: AnyInputPin) -> anyhow::Result<EspGate> {
        let input = PinDriver::input(pin)?;
        let input = input.into_input()?;
        Ok(Self { input })
    }
}

impl Gate for EspGate {
    fn is_active(&self) -> bool {
        self.input.is_low()
    }

    fn state(&self) -> GateState {
        if self.is_active() {
            GateState::Active
        } else {
            GateState::Inactive
        }
    }
}
