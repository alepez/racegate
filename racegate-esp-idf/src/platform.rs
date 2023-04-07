use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::peripherals::Peripherals;

use crate::drivers::gate::EspGate;
use crate::drivers::http::HttpServer as EspHttpServer;
use crate::drivers::rgb_led::WS2812RgbLed;
use crate::drivers::wifi::EspWifi;
use racegate::hal::gate::Gate;
use racegate::hal::rgb_led::RgbLed;
use racegate::hal::wifi::{Wifi, WifiConfig};
use racegate::hal::Platform;
use racegate::svc::HttpServer;

pub enum BoardType {
    M5StampC3,
    RustDevKit,
}

pub struct PlatformImpl {
    wifi: EspWifi,
    rgb_led: WS2812RgbLed,
    gate: EspGate,
    http_server: EspHttpServer,
}

pub struct Config {
    pub wifi: WifiConfig<'static>,
    pub board_type: BoardType,
}

impl PlatformImpl {
    pub fn new(config: &Config) -> Self {
        let peripherals = Peripherals::take().unwrap();

        let wifi = EspWifi::new(peripherals.modem).expect("Cannot create Wi-Fi");
        wifi.setup(&config.wifi).expect("Cannot setup Wi-Fi");

        let rgb_led = WS2812RgbLed::default();

        let gate_pin = match config.board_type {
            BoardType::M5StampC3 => peripherals.pins.gpio3.downgrade_input(),
            BoardType::RustDevKit => peripherals.pins.gpio9.downgrade_input(),
        };

        let gate = EspGate::new(gate_pin).expect("Cannot setup gate");
        let http_server = EspHttpServer::new().expect("Cannot setup http server");

        Self {
            wifi,
            rgb_led,
            gate,
            http_server,
        }
    }
}

impl Platform for PlatformImpl {
    fn wifi(&self) -> &(dyn Wifi + '_) {
        &self.wifi
    }

    fn rgb_led(&self) -> &(dyn RgbLed + '_) {
        &self.rgb_led
    }

    fn gate(&self) -> &(dyn Gate + '_) {
        &self.gate
    }

    fn http_server(&self) -> &(dyn HttpServer + '_) {
        &self.http_server
    }
}
