// pub use rust_devkit::PlatformImpl;
pub use m5stamp_c3::PlatformImpl;

use crate::config::Config;
use crate::hal::Platform;

mod m5stamp_c3;
mod rust_devkit;

pub fn create(config: &Config) -> Box<dyn Platform> {
    Box::new(PlatformImpl::new(config))
}
