pub trait Wifi {
    fn setup(&mut self, config: &WifiConfig) -> anyhow::Result<()>;

    fn is_connected(&self) -> bool;
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
