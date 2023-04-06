use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver};

use crate::hal::gate::Gate;

pub struct EspGate {
    input: PinDriver<'static, AnyInputPin, Input>,
}

impl EspGate {
    pub fn new(pin: AnyInputPin) -> Self {
        let input = PinDriver::input(pin).unwrap();
        let input = input.into_input().unwrap();
        Self { input }
    }
}

impl Gate for EspGate {
    fn is_active(&self) -> bool {
        self.input.is_low()
    }
}
