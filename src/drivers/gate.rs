use esp_idf_hal::gpio::{AnyInputPin, Input, InputPin, PinDriver, Pins};

use crate::hal::gate::Gate;

pub struct DevkitButton {
    input: PinDriver<'static, AnyInputPin, Input>,
}

impl DevkitButton {
    pub fn new(pins: Pins) -> Self {
        let pin = pins.gpio9.downgrade_input();
        let input = PinDriver::input(pin).unwrap();
        let input = input.into_input().unwrap();
        Self { input }
    }
}

impl Gate for DevkitButton {
    fn is_active(&self) -> bool {
        self.input.is_low()
    }
}

pub struct M5StampC3Button {
    input: PinDriver<'static, AnyInputPin, Input>,
}

impl M5StampC3Button {
    pub fn new(pins: Pins) -> Self {
        let pin = pins.gpio3.downgrade_input();
        let input = PinDriver::input(pin).unwrap();
        let input = input.into_input().unwrap();
        Self { input }
    }
}

impl Gate for M5StampC3Button {
    fn is_active(&self) -> bool {
        self.input.is_low()
    }
}
