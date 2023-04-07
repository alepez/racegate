use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use racegate::app::SystemState;
use racegate::svc::{RaceNode, RaceNodeMessage};

pub struct EspRaceNode {
    #[allow(dead_code)]
    thread: JoinHandle<()>,
    state: SharedNodeState,
}

impl EspRaceNode {
    pub fn new() -> anyhow::Result<Self> {
        let state = SharedNodeState::default();
        let state_copy = state.clone();

        log::info!("Starting race node");

        let sender_addr: SocketAddr = "0.0.0.0:0".parse()?;
        let sender = UdpSocket::bind(sender_addr)?;
        sender.set_broadcast(true)?;

        let receiver_addr: SocketAddr = "0.0.0.0:6699".parse()?;
        let receiver = UdpSocket::bind(receiver_addr)?;
        receiver.set_broadcast(true)?;
        receiver.set_read_timeout(Some(Duration::from_millis(40)))?;

        let broadcast_addr: SocketAddr = "255.255.255.255:6699".parse()?;

        let thread = std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(move || loop {
                let msg: Option<RaceNodeMessage> = state_copy.clone().try_into().ok();
                if let Some(msg) = msg {
                    // log::info!("send");
                    sender.send_to(&msg.data(), broadcast_addr).ok();
                }

                // log::info!("receive");
                loop {
                    let mut buf: [u8; 16] = [0; 16];
                    if let Ok((number_of_bytes, src_addr)) = receiver.recv_from(&mut buf) {
                        if number_of_bytes == 16 {
                            let msg = RaceNodeMessage::from(buf);
                            let s = SystemState::from(&msg);
                            log::info!("{src_addr} : {:?}", s);
                        }
                    } else {
                        break;
                    }
                }
            })
            .unwrap();

        Ok(EspRaceNode { thread, state })
    }
}

impl RaceNode for EspRaceNode {
    fn set_system_state(&self, state: &SystemState) {
        self.state
            .0
            .try_lock()
            .as_mut()
            .map(|x| {
                // TODO Change start/finish depending on this node type
                x.this = Some(*state)
            })
            .ok();
    }
}

#[derive(Clone, Default)]
struct NodeState {
    start: Option<SystemState>,
    finish: Option<SystemState>,
    this: Option<SystemState>,
}

#[derive(Clone)]
struct SharedNodeState(Arc<Mutex<NodeState>>);

impl Default for SharedNodeState {
    fn default() -> Self {
        SharedNodeState(Arc::new(Mutex::new(NodeState::default())))
    }
}

impl TryFrom<SharedNodeState> for RaceNodeMessage {
    type Error = ();

    fn try_from(x: SharedNodeState) -> Result<Self, Self::Error> {
        x.0.try_lock()
            .ok()
            .and_then(|x| x.this)
            .map(|x| RaceNodeMessage::from(&x))
            .ok_or(())
    }
}
