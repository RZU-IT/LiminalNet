use crate::model::{PacketInfo, parse};
use anyhow::{Context, Result};
use pcap::{Capture, Device};
use std::{
    net::IpAddr,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct Adapter {
    pub name: String,
    pub description: String,
    pub addresses: Vec<IpAddr>,
}
pub fn adapters() -> Result<Vec<Adapter>> {
    Device::list()
        .context("Cannot list capture interfaces")
        .map(|devices| {
            devices
                .into_iter()
                .map(|d| Adapter {
                    description: d.desc.unwrap_or_else(|| d.name.clone()),
                    name: d.name,
                    addresses: d.addresses.into_iter().map(|a| a.addr).collect(),
                })
                .collect()
        })
}
pub enum Message {
    Ready,
    Warning(String),
    Packets(Vec<PacketInfo>),
    Stats(u64),
    Paused(bool),
    Finished,
    Error(String),
}
pub struct Worker {
    pub messages: Receiver<Message>,
    pub paused: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start(
        adapter: Option<String>,
        file: Option<PathBuf>,
        bpf: String,
        export: Option<PathBuf>,
    ) -> Result<Self> {
        if let Some(path) = &export {
            anyhow::ensure!(
                !path.exists(),
                "Recording already exists: {}",
                path.display()
            );
        }
        let (tx, messages) = mpsc::sync_channel(32);
        let stop = Arc::new(AtomicBool::new(false));
        let paused = Arc::new(AtomicBool::new(false));
        let control = (stop.clone(), paused.clone());
        let thread = thread::Builder::new().name("packet-capture".into()).spawn(move || {
            let result = if let Some(file) = file {
                Capture::from_file(&file).with_context(||format!("Cannot open {}",file.display())).and_then(|cap|run(cap,&bpf,export,&tx,control,false))
            } else {
                (|| { let name=adapter.context("No capture interface selected")?;
                    let cap=Capture::from_device(name.as_str())?.promisc(false).snaplen(65535).timeout(100).open().context("Cannot open capture interface; check capture permissions and driver")?.setnonblock()?;
                    run(cap,&bpf,export,&tx,control,true) })()
            };
            if let Err(error)=result { let _=tx.send(Message::Error(format!("{error:#}"))); }
        })?;
        Ok(Self {
            messages,
            paused,
            stop,
            thread: Some(thread),
        })
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let (_, replacement) = mpsc::channel();
        drop(std::mem::replace(&mut self.messages, replacement));
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
fn run<T: pcap::Activated>(
    mut cap: Capture<T>,
    bpf: &str,
    export: Option<PathBuf>,
    tx: &SyncSender<Message>,
    control: (Arc<AtomicBool>, Arc<AtomicBool>),
    live: bool,
) -> Result<()> {
    if !bpf.trim().is_empty() {
        cap.filter(bpf, true).context("Invalid capture filter")?;
    }
    let link = cap.get_datalink().0;
    anyhow::ensure!(
        [0, 1, 12, 101, 108, 113, 228, 229, 276].contains(&link),
        "Unsupported link type {link}"
    );
    let mut save = match export {
        Some(path) => Some(
            cap.savefile(path)
                .context("Cannot create capture recording")?,
        ),
        None => None,
    };
    let mut batch = Vec::with_capacity(256);
    let mut last = Instant::now();
    let mut stats = Instant::now();
    let mut is_paused = false;
    if tx.send(Message::Ready).is_err() {
        return Ok(());
    }
    loop {
        if control.0.load(Ordering::Relaxed) {
            break;
        }
        let requested = control.1.load(Ordering::Relaxed);
        if requested != is_paused {
            if requested
                && !batch.is_empty()
                && tx
                    .send(Message::Packets(std::mem::take(&mut batch)))
                    .is_err()
            {
                return Ok(());
            }
            if live {
                cap.filter(
                    if requested {
                        "less 1"
                    } else if bpf.trim().is_empty() {
                        "greater 0"
                    } else {
                        bpf
                    },
                    true,
                )?;
                if requested {
                    while cap.next_packet().is_ok() {}
                }
            }
            is_paused = requested;
            if tx.send(Message::Paused(is_paused)).is_err() {
                return Ok(());
            }
        }
        if is_paused {
            thread::sleep(Duration::from_millis(25));
            continue;
        }
        match cap.next_packet() {
            Ok(packet) => {
                if let Some(writer) = save.as_mut() {
                    writer.write(&packet);
                }
                let timestamp =
                    packet.header.ts.tv_sec as f64 + packet.header.ts.tv_usec as f64 / 1_000_000.0;
                if let Some(info) =
                    parse(packet.data, link, u64::from(packet.header.len), timestamp)
                {
                    batch.push(info);
                }
            }
            Err(pcap::Error::TimeoutExpired | pcap::Error::NoMorePackets) if live => {
                thread::sleep(Duration::from_millis(5))
            }
            Err(pcap::Error::NoMorePackets) => {
                if !batch.is_empty() {
                    let _ = tx.send(Message::Packets(std::mem::take(&mut batch)));
                }
                let _ = tx.send(Message::Finished);
                break;
            }
            Err(e) => return Err(e).context("Packet capture failed"),
        }
        if batch.len() >= 256 || last.elapsed() >= Duration::from_millis(100) {
            if !batch.is_empty() {
                let mut message = Message::Packets(std::mem::take(&mut batch));
                loop {
                    match tx.try_send(message) {
                        Ok(()) => break,
                        Err(TrySendError::Disconnected(_)) => return Ok(()),
                        Err(TrySendError::Full(pending)) => {
                            if control.0.load(Ordering::Relaxed) {
                                return Ok(());
                            }
                            message = pending;
                            thread::sleep(Duration::from_millis(5));
                        }
                    }
                }
            }
            last = Instant::now();
        }
        if live && stats.elapsed() >= Duration::from_secs(1) {
            if let Some(writer) = save.as_mut()
                && let Err(error) = writer.flush()
            {
                let _ = tx.try_send(Message::Warning(format!(
                    "Recording failed; capture continues: {error}"
                )));
                save = None;
            }
            if let Ok(s) = cap.stats() {
                let _ = tx.try_send(Message::Stats(u64::from(s.dropped)));
            }
            stats = Instant::now();
        }
    }
    if let Some(writer) = save.as_mut() {
        writer.flush().context("Cannot flush capture recording")?;
    }
    Ok(())
}

#[cfg(all(test, any(target_os = "linux", target_os = "windows")))]
mod tests {
    use super::*;
    fn receive(worker: &Worker, until: Instant) -> u64 {
        let mut packets = 0;
        while Instant::now() < until {
            match worker.messages.recv_timeout(Duration::from_millis(25)) {
                Ok(Message::Packets(batch)) => packets += batch.len() as u64,
                Ok(Message::Error(error)) => panic!("{error}"),
                _ => {}
            }
        }
        packets
    }
    #[test]
    #[ignore = "requires capture privileges"]
    fn pause_excludes_actual_packets_and_resume_collects_again() {
        let sink = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let sender = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = sink.local_addr().unwrap();
        let adapter = if cfg!(windows) {
            "\\Device\\NPF_Loopback"
        } else {
            "lo"
        };
        let worker = Worker::start(
            Some(adapter.into()),
            None,
            format!("udp dst port {}", address.port()),
            None,
        )
        .unwrap();
        assert!(matches!(
            worker
                .messages
                .recv_timeout(Duration::from_secs(3))
                .unwrap(),
            Message::Ready
        ));
        sender.send_to(b"before", address).unwrap();
        assert!(receive(&worker, Instant::now() + Duration::from_millis(400)) > 0);
        worker.paused.store(true, Ordering::Relaxed);
        loop {
            if matches!(
                worker
                    .messages
                    .recv_timeout(Duration::from_secs(2))
                    .unwrap(),
                Message::Paused(true)
            ) {
                break;
            }
        }
        sender.send_to(b"during", address).unwrap();
        assert_eq!(
            receive(&worker, Instant::now() + Duration::from_millis(300)),
            0
        );
        worker.paused.store(false, Ordering::Relaxed);
        loop {
            if matches!(
                worker
                    .messages
                    .recv_timeout(Duration::from_secs(2))
                    .unwrap(),
                Message::Paused(false)
            ) {
                break;
            }
        }
        sender.send_to(b"after", address).unwrap();
        assert!(receive(&worker, Instant::now() + Duration::from_millis(400)) > 0);
    }
}
