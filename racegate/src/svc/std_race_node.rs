use std::net::{SocketAddr, UdpSocket};
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use anyhow::anyhow;

use crate::app::SystemState;
use crate::svc::race_node::{FrameData, RaceNode, RaceNodeMessage};
use crate::svc::Instant;

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
    tx: mpsc::Sender<RaceNodeMessage>,
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

        let (thread, tx) = spawn_thread(
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
            tx,
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
) -> (JoinHandle<Stats>, mpsc::Sender<RaceNodeMessage>) {
    let (tx_sender, tx_receiver) = mpsc::channel::<RaceNodeMessage>();

    let thread = std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(move || {
            let mut stats = Stats::default();

            loop {
                for tx_msg in tx_receiver.try_iter() {
                    if sender
                        .send_to(tx_msg.data().as_bytes(), broadcast_addr)
                        .is_ok()
                    {
                        stats.tx_count += 1;
                    }
                }

                while let Ok(rx_msg) = receive_message(&mut receiver) {
                    log::debug!("{:?}", rx_msg);
                    stats.rx_count += 1;

                    match rx_msg {
                        RaceNodeMessage::GateBeacon(_) => { /* Ignore */ }
                        RaceNodeMessage::CoordinatorBeacon(beacon) => {
                            state.try_modify(|x| x.coordinator_time = Some(beacon.time))
                        }
                    }
                }

                if !continue_running.load(Ordering::Acquire) {
                    break;
                }
            }
            stats
        })
        .unwrap();

    (thread, tx_sender)
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

    fn coordinator_time(&self) -> Option<Instant> {
        let nodes = self.state.0.lock().ok()?;
        nodes.coordinator_time
    }

    fn publish(&self, msg: RaceNodeMessage) -> anyhow::Result<()> {
        self.tx.send(msg).map_err(|_| anyhow!("Cannot publish"))
    }
}

#[derive(Default)]
struct NodesState {
    this: Option<SystemState>,
    coordinator_time: Option<Instant>,
}

#[derive(Clone)]
struct SharedNodeState(Arc<Mutex<NodesState>>);

impl Default for SharedNodeState {
    fn default() -> Self {
        SharedNodeState(Arc::new(Mutex::new(NodesState::default())))
    }
}

impl SharedNodeState {
    fn try_modify<F>(&self, f: F)
    where
        F: FnOnce(&mut NodesState),
    {
        // Ignore errors
        self.0.try_lock().map(|mut x| f(x.deref_mut())).ok();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::app::SystemState;
    use crate::hal::gate::GateState;
    use crate::svc::race_node::{CoordinatorBeacon, RaceNode};
    use crate::svc::std_race_node::StdRaceNodeConfig;
    use crate::svc::{Instant, StdRaceNode};

    fn make_coordinator_node() -> StdRaceNode {
        // Broadcast does not work on localhost, so we just use different ports

        let cfg = StdRaceNodeConfig {
            sender_addr: "0.0.0.0:0".parse().unwrap(),
            receiver_addr: "127.0.0.10:6699".parse().unwrap(),
            broadcast_addr: "127.0.0.10:6698".parse().unwrap(),
        };

        let node = StdRaceNode::new_with_config(cfg).unwrap();

        node.set_system_state(&SystemState {
            gate_state: GateState::Inactive,
            time: Instant::from_millis(12345),
        });

        node
    }

    fn make_start_node() -> StdRaceNode {
        // Broadcast does not work on localhost, so we just use different ports

        let cfg = StdRaceNodeConfig {
            sender_addr: "0.0.0.0:0".parse().unwrap(),
            receiver_addr: "127.0.0.10:6698".parse().unwrap(),
            broadcast_addr: "127.0.0.10:6699".parse().unwrap(),
        };

        let node = StdRaceNode::new_with_config(cfg).unwrap();

        node.set_system_state(&SystemState {
            gate_state: GateState::Active,
            time: Instant::from_millis(12345),
        });

        node
    }

    #[ignore]
    #[test_log::test]
    fn test_two_nodes_can_talk() {
        log::info!("test_two_nodes_can_talk");
        let mut coordinator_node = make_coordinator_node();
        let mut start_node = make_start_node();

        coordinator_node
            .publish(
                CoordinatorBeacon {
                    time: Instant::from_millis(123),
                }
                .into(),
            )
            .unwrap();

        std::thread::sleep(Duration::from_secs(1));

        let coordinator_stats = coordinator_node.stop().unwrap();
        let start_stats = start_node.stop().unwrap();

        assert_eq!(coordinator_stats.rx_count, 0);
        assert!(coordinator_stats.tx_count > 0);

        assert!(start_stats.rx_count > 0);
        assert_eq!(start_stats.tx_count, 0);
    }
}
