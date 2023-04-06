use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver};

use crate::hal::gate::Gate;

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
}
