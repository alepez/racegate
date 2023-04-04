use crate::drivers::rgb_led::{RgbLed, RgbLedColor};
use crate::platform::Platform;

pub struct App<'a> {
    led_controller: LedController<'a>,
}

impl<'a> App<'a> {
    pub fn new(platform: &'a Platform) -> Self {
        let led_controller = LedController {
            led: &platform.rgb_led,
        };
        let app = Self { led_controller };

        app
    }

    pub fn update(&mut self) {
        self.led_controller.update();
    }
}

struct LedController<'a> {
    led: &'a RgbLed,
}

impl<'a> LedController<'a> {
    pub fn update(&mut self) {
        self.led.set_color(RgbLedColor::from(0x00FF00));
    }
}
