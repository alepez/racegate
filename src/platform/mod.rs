pub use rust_devkit::RustDevkit as PlatformImpl;

use crate::config::Config;
use crate::hal::Platform;

mod rust_devkit;

pub fn create(config: &Config) -> Box<dyn Platform> {
    Box::new(PlatformImpl::new(config))
}
