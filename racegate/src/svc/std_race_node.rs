use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::app::SystemState;
use crate::svc::{RaceNode, RaceNodeMessage};

pub struct StdRaceNode {
    #[allow(dead_code)]
    thread: JoinHandle<()>,
    state: SharedNodeState,
}

struct StdRaceNodeConfig {
    sender_addr: SocketAddr,
    receiver_addr: SocketAddr,
    broadcast_addr: SocketAddr,
}

impl Default for StdRaceNodeConfig {
    fn default() -> Self {
        Self {
            sender_addr: "0.0.0.0:0".parse().unwrap(),
            receiver_addr: "0.0.0.0:6699".parse().unwrap(),
            broadcast_addr: "255.255.255.255:6699".parse().unwrap(),
        }
    }
}

impl StdRaceNode {
    pub fn new() -> anyhow::Result<Self> {
        Self::new_with_config(StdRaceNodeConfig::default())
    }

    fn new_with_config(config: StdRaceNodeConfig) -> anyhow::Result<Self> {
        let StdRaceNodeConfig {
            sender_addr,
            receiver_addr,
            broadcast_addr,
        } = config;

        let state = SharedNodeState::default();
        let state_copy = state.clone();

        log::info!("Starting race node");

        let sender = UdpSocket::bind(sender_addr)?;
        sender.set_broadcast(true)?;

        let receiver = UdpSocket::bind(receiver_addr)?;
        receiver.set_broadcast(true)?;
        receiver.set_read_timeout(Some(Duration::from_millis(40)))?;

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

        Ok(StdRaceNode { thread, state })
    }
}

impl RaceNode for StdRaceNode {
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

#[derive(Default)]
struct NodesState {
    start: Option<SystemState>,
    finish: Option<SystemState>,
    this: Option<SystemState>,
}

#[derive(Clone)]
struct SharedNodeState(Arc<Mutex<NodesState>>);

impl Default for SharedNodeState {
    fn default() -> Self {
        SharedNodeState(Arc::new(Mutex::new(NodesState::default())))
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

#[cfg(test)]
mod tests {
    use crate::svc::std_race_node::StdRaceNodeConfig;
    use crate::svc::StdRaceNode;
    use std::time::Duration;

    fn start_config() -> StdRaceNodeConfig {
        StdRaceNodeConfig {
            sender_addr: "127.0.0.1:0".parse().unwrap(),
            receiver_addr: "127.0.0.1:6699".parse().unwrap(),
            broadcast_addr: "127.0.0.255:6699".parse().unwrap(),
        }
    }

    fn finish_config() -> StdRaceNodeConfig {
        StdRaceNodeConfig {
            sender_addr: "127.0.0.2:0".parse().unwrap(),
            receiver_addr: "127.0.0.2:6699".parse().unwrap(),
            broadcast_addr: "127.0.0.255:6699".parse().unwrap(),
        }
    }

    #[test]
    fn test_node_can_be_created_with_default_config() {
        let node = StdRaceNode::new();
        assert!(node.is_ok());
    }

    #[test]
    fn test_two_nodes_can_talk() {
        let start_node = StdRaceNode::new_with_config(start_config());
        let finish_node = StdRaceNode::new_with_config(finish_config());
        std::thread::sleep(Duration::from_secs(5));
    }
}
