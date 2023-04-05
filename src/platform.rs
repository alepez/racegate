use crate::config::Config;
use crate::drivers::gate::Gate;
use crate::drivers::rgb_led::RgbLed;
use crate::drivers::wifi::Wifi;
use esp_idf_hal::peripherals::Peripherals;

pub struct Platform {
    pub wifi: Wifi,
    pub rgb_led: RgbLed,
    pub gate: Gate,
}

impl Platform {
    pub fn new(config: &Config) -> Self {
        let peripherals = Peripherals::take().unwrap();

        let wifi = Wifi::new(peripherals.modem, &config.wifi).expect("Cannot setup wifi");
        let rgb_led = RgbLed::new();
        let gate = Gate::new(peripherals.pins);
        Self {
            wifi,
            rgb_led,
            gate,
        }
    }
}
