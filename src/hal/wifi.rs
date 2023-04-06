pub trait Wifi {
    fn setup(&self, config: &WifiConfig) -> anyhow::Result<()>;

    fn is_connected(&self) -> bool;
}

#[derive(Eq, PartialEq)]
pub struct WifiConfig<'a> {
    pub ap: bool,
    pub ssid: &'a str,
    pub password: &'a str,
}

impl WifiConfig<'_> {
    fn try_from_str(s: &'static str) -> Result<Self, ()> {
        let mut iter = s.split_terminator(":");
        let ap: bool = iter.next().ok_or_else(|| ())?.parse().or(Err(()))?;
        let ssid: &str = iter.next().ok_or_else(|| ())?;
        let password: &str = iter.next().ok_or_else(|| ())?;
        Ok(WifiConfig { ap, ssid, password })
    }

    pub fn from_env_var() -> Result<Self, ()> {
        if let Some(s) = option_env!("RACEGATE_WIFI_CONFIG") {
            WifiConfig::try_from_str(s)
        } else {
            Err(())
        }
    }
}

impl Default for WifiConfig<'_> {
    fn default() -> Self {
        WifiConfig {
            ap: true,
            ssid: "racegate",
            password: "racegate",
        }
    }
}
