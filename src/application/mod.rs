use crate::drivers::rgb_led::{RgbLed, RgbLedColor};
use crate::platform::Platform;

#[derive(Default)]
pub struct AppState {
    is_wifi_connected: bool,
}

pub struct App<'a> {
    led_controller: LedController<'a>,
    platform: &'a Platform,
    state: AppState,
}

impl<'a> App<'a> {
    pub fn new(platform: &'a mut Platform) -> Self {
        let led_controller = LedController {
            led: &platform.rgb_led,
        };

        Self {
            led_controller,
            platform,
            state: AppState::default(),
        }
    }

    pub fn update(&mut self) {
        self.update_state();
        self.led_controller.update(&self.state);
    }

    pub fn update_state(&mut self) {
        let state = &mut self.state;
        state.is_wifi_connected = self.platform.wifi.is_connected();
    }
}

struct LedController<'a> {
    led: &'a RgbLed,
}

impl<'a> LedController<'a> {
    pub fn update(&mut self, app_state: &AppState) {
        let color = if app_state.is_wifi_connected {
            0x008000
        } else {
            0x800000
        };

        self.led.set_color(RgbLedColor::from(color));
    }
}
