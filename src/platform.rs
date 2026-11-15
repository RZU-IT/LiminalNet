use listeners::{Listener, Protocol};
use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct Process {
    pub name: String,
    pub path: String,
}
pub enum Update {
    Processes(Vec<Listener>),
    Networks(HashMap<String, (u64, u64)>),
    Error(String),
}
pub struct Monitor {
    pub messages: Receiver<Update>,
    stop: Arc<AtomicBool>,
}
impl Monitor {
    pub fn start() -> Self {
        let (tx, messages) = mpsc::sync_channel(4);
        let stop = Arc::new(AtomicBool::new(false));
        let control = stop.clone();
        thread::spawn(move || {
            let mut networks = sysinfo::Networks::new_with_refreshed_list();
            let mut tick = Instant::now();
            while !control.load(Ordering::Relaxed) {
                if tick.elapsed() >= Duration::from_secs(1) {
                    match listeners::get_all() {
                        Ok(list) => {
                            let _ = tx.try_send(Update::Processes(list.into_iter().collect()));
                        }
                        Err(e) => {
                            let _ = tx.try_send(Update::Error(format!("Process enumeration: {e}")));
                        }
                    }
                    networks.refresh(true);
                    let mut values = HashMap::new();
                    for (name, data) in &networks {
                        let counters = (data.received(), data.transmitted());
                        values.insert(name.clone(), counters);
                        for network in data.ip_networks() {
                            values.insert(network.addr.to_string(), counters);
                        }
                    }
                    let _ = tx.try_send(Update::Networks(values));
                    tick = Instant::now();
                }
                thread::sleep(Duration::from_millis(50));
            }
        });
        Self { messages, stop }
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
pub fn process_for(list: &[Listener], ip: IpAddr, port: u16, protocol: &str) -> Option<Process> {
    let matches: Vec<_> = list
        .iter()
        .filter(|l| {
            l.socket.port() == port
                && matches!(
                    (&l.protocol, protocol),
                    (Protocol::TCP, "TCP") | (Protocol::UDP, "UDP")
                )
                && (l.socket.ip() == ip
                    || (l.socket.ip().is_unspecified() && l.socket.ip().is_ipv4() == ip.is_ipv4()))
        })
        .collect();
    let exact: Vec<_> = matches.iter().filter(|l| l.socket.ip() == ip).collect();
    let candidate = if exact.len() == 1 {
        Some(exact[0])
    } else if matches.len() == 1 {
        Some(&matches[0])
    } else {
        None
    }?;
    Some(Process {
        name: candidate.process.name.clone(),
        path: candidate.process.path.clone(),
    })
}
pub fn preview_push(history: &mut VecDeque<u64>, value: u64) {
    history.push_back(value);
    while history.len() > 30 {
        history.pop_front();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn attributes_a_real_local_listener() {
        let socket = std::net::TcpListener::bind("127.0.0.1:0").expect("local test socket");
        let address = socket.local_addr().unwrap();
        let listeners: Vec<_> = listeners::get_all()
            .expect("enumerate own socket")
            .into_iter()
            .collect();
        let process = process_for(&listeners, address.ip(), address.port(), "TCP")
            .expect("identify own listener");
        assert!(!process.name.is_empty());
        assert!(!process.path.is_empty());
    }
}
