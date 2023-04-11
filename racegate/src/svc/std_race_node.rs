use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::anyhow;

use crate::app::SystemState;
use crate::svc::race_node::FrameData;
use crate::svc::{RaceNode, RaceNodeMessage};

#[derive(Default, Debug)]
struct Stats {
    tx_count: usize,
    rx_count: usize,
}

pub struct StdRaceNode {
    #[allow(dead_code)]
    thread: Option<JoinHandle<Stats>>,
    state: SharedNodeState,
    continue_running: Arc<AtomicBool>,
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

        log::info!("Starting race node");

        let sender = make_sender(sender_addr)?;
        let receiver = make_receiver(receiver_addr)?;

        let continue_running = Arc::new(AtomicBool::new(true));
        let thread = spawn_thread(
            broadcast_addr,
            state.clone(),
            sender,
            receiver,
            continue_running.clone(),
        );

        Ok(StdRaceNode {
            thread: Some(thread),
            state,
            continue_running,
        })
    }

    fn stop(&mut self) -> Option<Stats> {
        self.continue_running.store(false, Ordering::Release);

        if let Some(thread) = self.thread.take() {
            thread.join().ok()
        } else {
            None
        }
    }
}

impl Drop for StdRaceNode {
    fn drop(&mut self) {
        if let Some(stats) = self.stop() {
            log::debug!("stats: {:?}", stats);
        }
    }
}

fn spawn_thread(
    broadcast_addr: SocketAddr,
    state: SharedNodeState,
    sender: UdpSocket,
    mut receiver: UdpSocket,
    continue_running: Arc<AtomicBool>,
) -> JoinHandle<Stats> {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(move || {
            let mut stats = Stats::default();

            loop {
                let tx_msg: Option<RaceNodeMessage> = state.clone().try_into().ok();

                if let Some(tx_msg) = tx_msg {
                    if sender
                        .send_to(&tx_msg.data().as_bytes(), broadcast_addr)
                        .is_ok()
                    {
                        stats.tx_count += 1;
                    }
                }

                while let Ok(rx_msg) = receive_message(&mut receiver) {
                    log::info!("{:?}", rx_msg);
                    stats.rx_count += 1;
                }

                if !continue_running.load(Ordering::Acquire) {
                    break;
                }
            }
            stats
        })
        .unwrap()
}

fn make_receiver(receiver_addr: SocketAddr) -> anyhow::Result<UdpSocket> {
    let receiver = UdpSocket::bind(receiver_addr)?;
    receiver.set_broadcast(true)?;
    receiver.set_read_timeout(Some(Duration::from_millis(40)))?;
    log::info!("receiver {:?}", receiver.local_addr());
    Ok(receiver)
}

fn make_sender(sender_addr: SocketAddr) -> anyhow::Result<UdpSocket> {
    let sender = UdpSocket::bind(sender_addr)?;
    sender.set_broadcast(true)?;
    log::info!("sender {:?}", sender.local_addr());
    Ok(sender)
}

fn receive_message(receiver: &mut UdpSocket) -> anyhow::Result<RaceNodeMessage> {
    let mut buf = [0u8; RaceNodeMessage::FRAME_SIZE];

    if let Ok((number_of_bytes, _src_addr)) = receiver.recv_from(&mut buf) {
        if number_of_bytes == RaceNodeMessage::FRAME_SIZE {
            let data = FrameData::from(buf);
            RaceNodeMessage::try_from(data).map_err(|_| anyhow!("Cannot parse"))
        } else {
            Err(anyhow!("Wrong number of bytes"))
        }
    } else {
        Err(anyhow!("No messages"))
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
            .map(RaceNodeMessage::SystemState)
            .ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::app::SystemState;
    use crate::hal::gate::GateState;
    use crate::svc::std_race_node::StdRaceNodeConfig;
    use crate::svc::{RaceNode, StdRaceNode};

    fn start_config() -> StdRaceNodeConfig {
        StdRaceNodeConfig {
            sender_addr: "0.0.0.0:0".parse().unwrap(),
            receiver_addr: "127.0.0.10:6699".parse().unwrap(),
            broadcast_addr: "127.0.0.10:6698".parse().unwrap(),
        }
    }

    fn finish_config() -> StdRaceNodeConfig {
        StdRaceNodeConfig {
            sender_addr: "0.0.0.0:0".parse().unwrap(),
            receiver_addr: "127.0.0.10:6698".parse().unwrap(),
            broadcast_addr: "127.0.0.10:6699".parse().unwrap(),
        }
    }

    #[test_log::test]
    fn test_two_nodes_can_talk() {
        log::info!("test_two_nodes_can_talk");
        let mut start_node = StdRaceNode::new_with_config(start_config()).unwrap();

        start_node.set_system_state(&SystemState {
            gate_state: GateState::Active,
        });

        let mut finish_node = StdRaceNode::new_with_config(finish_config()).unwrap();

        finish_node.set_system_state(&SystemState {
            gate_state: GateState::Inactive,
        });

        std::thread::sleep(Duration::from_secs(1));

        let start_stats = start_node.stop().unwrap();
        let finish_stats = finish_node.stop().unwrap();

        assert!(start_stats.rx_count > 0);
        assert!(start_stats.tx_count > 0);

        assert!(finish_stats.rx_count > 0);
        assert!(finish_stats.tx_count > 0);
    }
}
