use std::time::Duration;

use esp_idf_sys as _;

use racegate::app::App;
use racegate::hal::wifi::WifiConfig;
use racegate::platform::Config;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = Config {
        wifi: WifiConfig::from_env_var().unwrap_or_default(),
    };

    log::info!("Create platform");
    let mut p = racegate::platform::create(&config);
    let p = p.as_mut();

    log::info!("Create app");
    let mut app = App::new(p);

    let period = Duration::from_millis(10);

    log::info!("Start loop");
    loop {
        std::thread::sleep(period);
        app.update();
    }
}
