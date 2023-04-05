use esp_idf_hal::peripherals::Peripherals;

use crate::config::Config;
use crate::drivers::gate::DevkitButton as Gate;
use crate::drivers::rgb_led::RgbLed;
use crate::drivers::wifi::Wifi;

pub struct Platform {
    pub wifi: Wifi,
    pub rgb_led: RgbLed,
    pub gate: Gate,
}

impl Platform {
    pub fn new(config: &Config) -> Self {
        let peripherals = Peripherals::take().unwrap();

        let wifi = Wifi::new(peripherals.modem, &config.wifi).expect("Cannot setup wifi");
        let rgb_led = RgbLed::default();
        let gate = Gate::new(peripherals.pins);
        Self {
            wifi,
            rgb_led,
            gate,
        }
    }
}
