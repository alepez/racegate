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
        let bind_addr: SocketAddr = "0.0.0.0:6699".parse()?;
        let dst_addr: SocketAddr = "255.255.255.255:6699".parse()?;
        log::info!("Starting race node at {}", bind_addr);
        let socket = UdpSocket::bind(bind_addr)?;
        socket.connect(dst_addr)?;
        log::info!("race node ready at {}", bind_addr);

        let thread = std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(40));
            let msg: Option<RaceNodeMessage> = state_copy.clone().try_into().ok();
            if let Some(msg) = msg {
                socket.send(&msg.data()).ok();
            }
        });

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
