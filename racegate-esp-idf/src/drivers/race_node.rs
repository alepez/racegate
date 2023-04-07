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
        socket.set_read_timeout(Some(Duration::from_millis(40)))?;

        let thread = std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(move || loop {
                let msg: Option<RaceNodeMessage> = state_copy.clone().try_into().ok();
                if let Some(msg) = msg {
                    socket.send(&msg.data()).ok();
                }

                // log::info!("waiting messages from other nodes...");
                let mut count = 0;
                loop {
                    let mut buf: [u8; 16] = [0; 16];
                    if let Ok((number_of_bytes, src_addr)) = socket.recv_from(&mut buf) {
                        count += 1;
                        if number_of_bytes == 16 {
                            let msg = RaceNodeMessage::from(buf);
                            let s = SystemState::from(&msg);
                            log::info!("{:?}", s);
                        }
                    } else {
                        break;
                    }
                }
                if count > 0 {
                    log::info!("received count: {}", count);
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
