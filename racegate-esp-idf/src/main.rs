use std::time::Duration;

use esp_idf_sys as _;

use racegate::app::App;
use racegate::hal::wifi::WifiConfig;
use racegate_esp_idf::platform::{BoardType, Config, PlatformImpl};

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = Config {
        wifi: WifiConfig::from_env_var().unwrap_or_default(),
        #[cfg(feature = "m5stampc3")]
        board_type: BoardType::M5StampC3,
        #[cfg(feature = "rustdevkit")]
        board_type: BoardType::RustDevKit,
    };

    log::info!("Create platform");
    let mut p = PlatformImpl::new(&config);

    log::info!("Create app");
    let mut app = App::new(&mut p);

    let period = Duration::from_millis(10);

    log::info!("Start loop");
    loop {
        std::thread::sleep(period);
        app.update();
    }
}
