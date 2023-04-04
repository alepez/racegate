use crate::drivers::wifi::WifiConfig;

#[toml_cfg::toml_config]
pub struct TomlConfig {
    #[default(true)]
    pub ap: bool,
    #[default("")]
    pub wifi_ssid: &'static str,
    #[default("")]
    pub wifi_password: &'static str,
}

pub struct Config {
    pub wifi: WifiConfig<'static>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            wifi: WifiConfig {
                ap: TOML_CONFIG.ap,
                ssid: TOML_CONFIG.wifi_ssid,
                password: TOML_CONFIG.wifi_password,
            },
        }
    }
}
