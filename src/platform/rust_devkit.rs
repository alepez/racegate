use esp_idf_hal::peripherals::Peripherals;

use crate::config::Config;
use crate::drivers::gate::DevkitButton;
use crate::drivers::rgb_led::WS2812RgbLed;
use crate::drivers::wifi::EspWifi;
use crate::hal::gate::Gate;
use crate::hal::rgb_led::RgbLed;
use crate::hal::wifi::Wifi;
use crate::hal::Platform;

pub struct RustDevkit {
    pub wifi: EspWifi,
    pub rgb_led: WS2812RgbLed,
    pub gate: DevkitButton,
}

impl RustDevkit {
    pub fn new(config: &Config) -> Self {
        let peripherals = Peripherals::take().unwrap();

        let wifi = EspWifi::new(peripherals.modem, &config.wifi).expect("Cannot setup wifi");
        let rgb_led = WS2812RgbLed::default();
        let gate = DevkitButton::new(peripherals.pins);
        Self {
            wifi,
            rgb_led,
            gate,
        }
    }
}

impl Platform for RustDevkit {
    fn wifi(&self) -> &(dyn Wifi + '_) {
        &self.wifi
    }

    fn rgb_led(&self) -> &(dyn RgbLed + '_) {
        &self.rgb_led
    }

    fn gate(&self) -> &(dyn Gate + '_) {
        &self.gate
    }
}
