#[cfg(feature = "m5stamp_c3")]
pub use m5stamp_c3::PlatformImpl;
#[cfg(feature = "rust_devkit")]
pub use rust_devkit::PlatformImpl;

use crate::hal::wifi::WifiConfig;
use crate::hal::Platform;

#[cfg(feature = "m5stamp_c3")]
mod m5stamp_c3;

#[cfg(feature = "rust_devkit")]
mod rust_devkit;

pub struct Config {
    pub wifi: WifiConfig<'static>,
}

pub fn create(config: &Config) -> Box<dyn Platform> {
    Box::new(PlatformImpl::new(config))
}
