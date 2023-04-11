use anyhow::anyhow;
use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver};
use racegate::hal::dip_switch::DipSwitch;
use racegate::svc::race_node::NodeAddress;

pub struct EspDipSwitch {
    pins: [PinDriver<'static, AnyInputPin, Input>; 3],
}

impl EspDipSwitch {
    pub fn new(pins: [AnyInputPin; 3]) -> anyhow::Result<EspDipSwitch> {
        let mut pins = pins
            .into_iter()
            .map(|pin| PinDriver::input(pin).unwrap().into_input().unwrap());
        Ok(Self {
            pins: [
                pins.next().ok_or(anyhow!("Invalid pin"))?,
                pins.next().ok_or(anyhow!("Invalid pin"))?,
                pins.next().ok_or(anyhow!("Invalid pin"))?,
            ],
        })
    }
}

impl DipSwitch for EspDipSwitch {
    fn address(&self) -> NodeAddress {
        let p0 = self.pins[0].is_low() as u8;
        let p1 = self.pins[1].is_low() as u8;
        let p2 = self.pins[2].is_low() as u8;

        log::info!("{p0} {p1} {p2}");

        NodeAddress::from((p0 << 2) | (p1 << 1) | p2)
    }
}
