use crate::hal::gate::Gate;
use crate::hal::rgb_led::RgbLed;
use crate::hal::wifi::Wifi;

pub mod gate;
pub mod rgb_led;
pub mod wifi;

pub trait Platform {
    fn wifi(&self) -> &(dyn Wifi + '_);
    fn rgb_led(&self) -> &(dyn RgbLed + '_);
    fn gate(&self) -> &(dyn Gate + '_);
}
