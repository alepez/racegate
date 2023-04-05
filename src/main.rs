use std::time::Duration;

use esp_idf_sys as _;

use racegate::app::App;
use racegate::config::Config;
use racegate::platform::Platform;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = Config::default();

    log::info!("Create platform");
    let mut platform = Platform::new(&config);

    log::info!("Create app");
    let mut app = App::new(&mut platform);

    let period = Duration::from_millis(20);

    log::info!("Start loop");
    loop {
        std::thread::sleep(period);
        app.update();
    }
}
