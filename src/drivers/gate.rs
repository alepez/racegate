use esp_idf_hal::gpio::{AnyInputPin, Input, InputPin, PinDriver, Pins};

pub struct Gate {
    input: PinDriver<'static, AnyInputPin, Input>,
}

impl Gate {
    pub fn new(pins: Pins) -> Self {
        let pin = pins.gpio9.downgrade_input();
        let input = PinDriver::input(pin).unwrap();
        let input = input.into_input().unwrap();
        Self { input }
    }

    pub fn is_active(&self) -> bool {
        self.input.is_low()
    }
}
