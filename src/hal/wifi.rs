pub trait Wifi {
    fn setup(&self, config: &WifiConfig) -> anyhow::Result<()>;

    fn is_connected(&self) -> bool;
}

pub struct WifiConfig<'a> {
    pub ap: bool,
    pub ssid: &'a str,
    pub password: &'a str,
}
