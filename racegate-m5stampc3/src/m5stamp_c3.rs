use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::peripherals::Peripherals;

use crate::drivers::gate::EspGate;
use crate::drivers::http::HttpServer as EspHttpServer;
use crate::drivers::rgb_led::WS2812RgbLed;
use crate::drivers::wifi::EspWifi;
use crate::hal::gate::Gate;
use crate::hal::rgb_led::RgbLed;
use crate::hal::wifi::Wifi;
use crate::hal::Platform;
use crate::platform::Config;
use crate::svc::HttpServer;

pub struct PlatformImpl {
    wifi: EspWifi,
    rgb_led: WS2812RgbLed,
    gate: EspGate,
    http_server: EspHttpServer,
}

impl PlatformImpl {
    pub fn new(config: &Config) -> Self {
        let peripherals = Peripherals::take().unwrap();

        let wifi = EspWifi::new(peripherals.modem).expect("Cannot create Wi-Fi");
        wifi.setup(&config.wifi).expect("Cannot setup Wi-Fi");

        let rgb_led = WS2812RgbLed::default();
        let gate =
            EspGate::new(peripherals.pins.gpio3.downgrade_input()).expect("Cannot setup gate");
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
