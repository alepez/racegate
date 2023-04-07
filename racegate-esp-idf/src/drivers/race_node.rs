use std::net::{SocketAddr, UdpSocket};

use racegate::app::SystemState;
use racegate::svc::{RaceNode, RaceNodeMessage};

pub struct EspRaceNode {
    socket: UdpSocket,
}

impl EspRaceNode {
    pub fn new() -> anyhow::Result<Self> {
        let bind_addr: SocketAddr = "0.0.0.0:6699".parse()?;
        let dst_addr: SocketAddr = "255.255.255.255:6699".parse()?;
        log::info!("Starting race node at {}", bind_addr);
        let socket = UdpSocket::bind(bind_addr)?;
        socket.connect(dst_addr)?;
        log::info!("race node ready at {}", bind_addr);
        Ok(EspRaceNode { socket })
    }
}

impl RaceNode for EspRaceNode {
    fn set_system_state(&self, status: &SystemState) {
        let msg: RaceNodeMessage = status.into();

        self.socket.send(&msg.data()).ok();
    }
}
