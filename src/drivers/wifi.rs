use std::net::Ipv4Addr;
use std::time::Duration;

use anyhow::bail;
use embedded_svc::wifi::{AccessPointConfiguration, AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::netif::{EspNetif, EspNetifWait};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{EspWifi, WifiWait};

pub struct Wifi {
    esp_wifi: EspWifi<'static>,
    sys_loop: EspSystemEventLoop,
}

pub struct WifiConfig<'a> {
    pub(crate) ap: bool,
    pub(crate) ssid: &'a str,
    pub(crate) password: &'a str,
}

impl WifiConfig<'_> {
    pub const fn default() -> Self {
        Self {
            ap: true,
            ssid: "racegate",
            password: "racegate",
        }
    }
}

impl TryInto<Configuration> for &WifiConfig<'_> {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<Configuration> {
        let &WifiConfig { ap, ssid, password } = self;

        if ssid.is_empty() {
            bail!("Wi-Fi SSID must be non-empty")
        }

        let auth_method = if password.is_empty() {
            log::info!("Wi-Fi password is empty. Authentication is disabled.");
            AuthMethod::None
        } else {
            AuthMethod::WPA2Personal
        };

        if ap {
            let config = AccessPointConfiguration {
                ssid: ssid.into(),
                password: password.into(),
                auth_method,
                ..Default::default()
            };

            Ok(Configuration::AccessPoint(config))
        } else {
            let config = ClientConfiguration {
                ssid: ssid.into(),
                password: password.into(),
                channel: Default::default(),
                auth_method,
                ..Default::default()
            };

            Ok(Configuration::Client(config))
        }
    }
}

impl Wifi {
    pub fn new(config: &WifiConfig) -> anyhow::Result<Wifi> {
        let peripherals = Peripherals::take().unwrap();
        let sys_loop = EspSystemEventLoop::take().unwrap();
        let nvs = EspDefaultNvsPartition::take().unwrap();

        let mut wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
        let config = config.try_into()?;
        wifi.set_configuration(&config)?;

        Ok(Wifi { esp_wifi: wifi, sys_loop })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        self.esp_wifi.start()?;
        let sys_loop = self.sys_loop.clone();

        let started = {
            let timeout = Duration::from_secs(20);
            let matcher = || self.esp_wifi.is_started().unwrap_or(false);
            WifiWait::new(&sys_loop)?.wait_with_timeout(timeout, matcher)
        };

        if !started {
            log::error!("Wi-Fi did not start");
        }

        self.esp_wifi.connect()?;

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.esp_wifi.driver().is_connected().unwrap_or(false)
    }
}
