use crate::drivers::rgb_led::{RgbLed, RgbLedColor};
use crate::platform::Platform;

pub struct App<'a> {
    led_controller: LedController<'a>,
    platform: &'a Platform,
}

impl<'a> App<'a> {
    pub fn new(platform: &'a mut Platform) -> Self {
        let led_controller = LedController {
            led: &platform.rgb_led,
            is_wifi_connected: false,
        };

        platform.wifi.start();

        Self { led_controller, platform }
    }

    pub fn update(&mut self) {
        self.led_controller.is_wifi_connected = self.platform.wifi.is_connected();
        self.led_controller.update();
    }
}

struct LedController<'a> {
    led: &'a RgbLed,
    is_wifi_connected: bool,
}

impl<'a> LedController<'a> {
    pub fn update(&mut self) {
        let color = if self.is_wifi_connected { 0x008000 } else { 0x800000 };
        self.led.set_color(RgbLedColor::from(color));
    }
}
