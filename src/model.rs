use etherparse::{LaxPacketHeaders, NetHeaders, TransportHeader};
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    time::Instant,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize)]
pub struct FlowKey {
    pub source: IpAddr,
    pub destination: IpAddr,
    pub source_port: u16,
    pub destination_port: u16,
    pub protocol: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct Flow {
    pub key: FlowKey,
    pub incoming: bool,
    pub bytes: u64,
    pub packets: u64,
    pub service: String,
    pub first: f64,
    pub last: f64,
    pub program: String,
    pub program_path: String,
    pub source_mac: Option<String>,
    pub destination_mac: Option<String>,
    pub protocol_details: String,
}
#[derive(Clone, Debug)]
pub struct PacketInfo {
    pub key: FlowKey,
    pub bytes: u64,
    pub timestamp: f64,
    pub source_mac: Option<String>,
    pub destination_mac: Option<String>,
    pub protocol_details: String,
}
#[derive(Clone, Copy, Default, Debug, Serialize)]
pub struct Totals {
    pub received: u64,
    pub sent: u64,
    pub received_packets: u64,
    pub sent_packets: u64,
}
impl Totals {
    pub fn value(self, packets: bool) -> u64 {
        if packets {
            self.received_packets + self.sent_packets
        } else {
            self.received + self.sent
        }
    }
}
#[derive(Clone, Debug)]
pub struct Sample {
    pub time: f64,
    pub received: f64,
    pub sent: f64,
    pub received_packets: f64,
    pub sent_packets: f64,
}
impl Sample {
    pub fn rate(&self, unit: crate::config::Unit) -> f64 {
        match unit {
            crate::config::Unit::Bytes => self.received + self.sent,
            crate::config::Unit::Bits => (self.received + self.sent) * 8.0,
            crate::config::Unit::Packets => self.received_packets + self.sent_packets,
        }
    }
}
pub struct Session {
    pub totals: Totals,
    pub flows: HashMap<FlowKey, Flow>,
    pub history: VecDeque<Sample>,
    pub dropped: u64,
    pub omitted_flows: u64,
    pub addresses: Vec<IpAddr>,
    pub started: Instant,
    tick: Instant,
    previous: Totals,
    pub elapsed: f64,
    file_origin: Option<f64>,
    file_bucket: f64,
    file_previous: Totals,
}
impl Session {
    pub fn new(addresses: Vec<IpAddr>) -> Self {
        let now = Instant::now();
        Self {
            totals: Totals::default(),
            flows: HashMap::new(),
            history: VecDeque::new(),
            dropped: 0,
            omitted_flows: 0,
            addresses,
            started: now,
            tick: now,
            previous: Totals::default(),
            elapsed: 0.0,
            file_origin: None,
            file_bucket: 0.0,
            file_previous: Totals::default(),
        }
    }
    pub fn ingest(&mut self, packet: PacketInfo) {
        let incoming = !self.addresses.contains(&packet.key.source);
        if incoming {
            self.totals.received += packet.bytes;
            self.totals.received_packets += 1;
        } else {
            self.totals.sent += packet.bytes;
            self.totals.sent_packets += 1;
        }
        if !self.flows.contains_key(&packet.key) && self.flows.len() >= 50_000 {
            self.omitted_flows += 1;
            return;
        }
        let service = service(
            packet.key.source_port,
            packet.key.destination_port,
            &packet.key.protocol,
        )
        .to_owned();
        let program = if matches!(packet.key.protocol.as_str(), "TCP" | "UDP") {
            "?"
        } else {
            "—"
        }
        .to_string();
        let flow = self.flows.entry(packet.key.clone()).or_insert(Flow {
            key: packet.key,
            incoming,
            bytes: 0,
            packets: 0,
            service,
            first: packet.timestamp,
            last: packet.timestamp,
            program,
            program_path: String::new(),
            source_mac: packet.source_mac,
            destination_mac: packet.destination_mac,
            protocol_details: packet.protocol_details,
        });
        flow.bytes += packet.bytes;
        flow.packets += 1;
        flow.first = flow.first.min(packet.timestamp);
        flow.last = flow.last.max(packet.timestamp);
    }
    pub fn ingest_offline(&mut self, packet: PacketInfo) {
        let origin = *self.file_origin.get_or_insert(packet.timestamp);
        let elapsed = (packet.timestamp - origin).max(self.elapsed);
        let bucket = elapsed.floor();
        if bucket > self.file_bucket {
            self.push_file_sample(self.file_bucket + 1.0);
            let mut time = (bucket - 29.0).max(self.file_bucket + 2.0);
            while time <= bucket {
                self.history.push_back(Sample {
                    time,
                    received: 0.0,
                    sent: 0.0,
                    received_packets: 0.0,
                    sent_packets: 0.0,
                });
                time += 1.0;
            }
            self.file_previous = self.totals;
            self.file_bucket = bucket;
        }
        self.elapsed = elapsed;
        self.ingest(packet);
        while self.history.len() > 30 {
            self.history.pop_front();
        }
    }
    fn push_file_sample(&mut self, time: f64) {
        let delta = self.file_previous;
        self.history.push_back(Sample {
            time,
            received: (self.totals.received - delta.received) as f64,
            sent: (self.totals.sent - delta.sent) as f64,
            received_packets: (self.totals.received_packets - delta.received_packets) as f64,
            sent_packets: (self.totals.sent_packets - delta.sent_packets) as f64,
        });
    }
    pub fn finish_offline(&mut self) {
        self.push_file_sample(self.file_bucket + 1.0);
        while self.history.len() > 30 {
            self.history.pop_front();
        }
    }
    pub fn tick(&mut self, paused: bool) {
        let now = Instant::now();
        let duration = now.duration_since(self.tick).as_secs_f64();
        if duration < 1.0 {
            return;
        }
        self.elapsed = now.duration_since(self.started).as_secs_f64();
        let scale = if paused { 0.0 } else { 1.0 / duration };
        self.history.push_back(Sample {
            time: self.elapsed,
            received: (self.totals.received - self.previous.received) as f64 * scale,
            sent: (self.totals.sent - self.previous.sent) as f64 * scale,
            received_packets: (self.totals.received_packets - self.previous.received_packets)
                as f64
                * scale,
            sent_packets: (self.totals.sent_packets - self.previous.sent_packets) as f64 * scale,
        });
        while self
            .history
            .front()
            .is_some_and(|sample| self.elapsed - sample.time > 30.0)
        {
            self.history.pop_front();
        }
        self.previous = self.totals;
        self.tick = now;
    }
    pub fn reset_tick(&mut self) {
        self.tick = Instant::now();
        self.previous = self.totals;
    }
    pub fn rows(&self) -> Vec<&Flow> {
        let mut rows: Vec<_> = self.flows.values().collect();
        rows.sort_by_key(|a| std::cmp::Reverse(a.bytes));
        rows
    }
}

pub fn parse(data: &[u8], link: i32, bytes: u64, timestamp: f64) -> Option<PacketInfo> {
    let mac = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(":")
    };
    let (source_mac, destination_mac) = if link == 1 && data.len() >= 14 {
        (Some(mac(&data[6..12])), Some(mac(&data[..6])))
    } else {
        (None, None)
    };
    if link == 1 && data.len() >= 42 && data[12..14] == [8, 6] && data[18..20] == [6, 4] {
        return Some(PacketInfo {
            key: FlowKey {
                source: IpAddr::from(<[u8; 4]>::try_from(&data[28..32]).ok()?),
                destination: IpAddr::from(<[u8; 4]>::try_from(&data[38..42]).ok()?),
                source_port: 0,
                destination_port: 0,
                protocol: "ARP".into(),
            },
            bytes,
            timestamp,
            source_mac,
            destination_mac,
            protocol_details: format!("ARP operation {}", u16::from_be_bytes([data[20], data[21]])),
        });
    }
    let headers = match link {
        1 => LaxPacketHeaders::from_ethernet(data).ok(),
        113 => LaxPacketHeaders::from_linux_sll(data).ok(),
        276 if data.len() >= 20 => LaxPacketHeaders::from_ip(&data[20..]).ok(),
        0 | 108 if data.len() >= 4 => LaxPacketHeaders::from_ip(&data[4..]).ok(),
        12 | 101 | 228 | 229 => LaxPacketHeaders::from_ip(data).ok(),
        _ => return None,
    }?;
    let (source, destination) = match headers.net? {
        NetHeaders::Ipv4(h, _) => (IpAddr::from(h.source), IpAddr::from(h.destination)),
        NetHeaders::Ipv6(h, _) => (IpAddr::from(h.source), IpAddr::from(h.destination)),
        _ => return None,
    };
    let protocol_details = match &headers.transport {
        Some(TransportHeader::Icmpv4(header)) => format!("{:?}", header.icmp_type),
        Some(TransportHeader::Icmpv6(header)) => format!("{:?}", header.icmp_type),
        _ => String::new(),
    };
    let (source_port, destination_port, protocol) = match headers.transport {
        Some(TransportHeader::Tcp(h)) => (h.source_port, h.destination_port, "TCP"),
        Some(TransportHeader::Udp(h)) => (h.source_port, h.destination_port, "UDP"),
        Some(TransportHeader::Icmpv4(_)) => (0, 0, "ICMP"),
        Some(TransportHeader::Icmpv6(_)) => (0, 0, "ICMPv6"),
        _ => (0, 0, "OTHER"),
    };
    Some(PacketInfo {
        key: FlowKey {
            source,
            destination,
            source_port,
            destination_port,
            protocol: protocol.into(),
        },
        bytes,
        timestamp,
        source_mac,
        destination_mac,
        protocol_details,
    })
}
pub fn service(a: u16, b: u16, protocol: &str) -> &'static str {
    static SERVICES: std::sync::OnceLock<HashMap<(&'static str, u16), &'static str>> =
        std::sync::OnceLock::new();
    let services = SERVICES.get_or_init(|| {
        include_str!("../assets/services.tsv")
            .lines()
            .filter_map(|line| {
                let mut fields = line.trim_start_matches('\u{feff}').split('\t');
                Some((
                    (fields.next()?, fields.next()?.parse().ok()?),
                    fields.next()?,
                ))
            })
            .collect()
    });
    let protocol = if protocol == "TCP" {
        "tcp"
    } else if protocol == "UDP" {
        "udp"
    } else {
        return "—";
    };
    let ports = if a <= 1023 && b > 1023 {
        [a, b]
    } else {
        [b, a]
    };
    ports
        .into_iter()
        .find_map(|port| services.get(&(protocol, port)).copied())
        .unwrap_or("?")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_known_udp_frame_and_accounts_direction() {
        let mut frame = vec![0u8; 42];
        frame[12..14].copy_from_slice(&[8, 0]);
        frame[14] = 0x45;
        frame[16..18].copy_from_slice(&28u16.to_be_bytes());
        frame[23] = 17;
        frame[26..30].copy_from_slice(&[10, 0, 0, 1]);
        frame[30..34].copy_from_slice(&[8, 8, 8, 8]);
        frame[34..36].copy_from_slice(&50000u16.to_be_bytes());
        frame[36..38].copy_from_slice(&53u16.to_be_bytes());
        frame[38..40].copy_from_slice(&8u16.to_be_bytes());
        let p = parse(&frame, 1, 42, 1.0).expect("valid fixture");
        assert_eq!(p.key.destination_port, 53);
        let mut session = Session::new(vec!["10.0.0.1".parse().unwrap()]);
        session.ingest(p.clone());
        session.ingest(p);
        assert_eq!(session.totals.sent, 84);
        assert_eq!(session.flows.len(), 1);
        assert_eq!(session.rows()[0].service, "domain");
    }
    #[test]
    fn rejects_truncated_input() {
        assert!(parse(&[0; 8], 1, 8, 0.0).is_none());
    }
    #[test]
    fn parses_raw_ipv6_udp() {
        let mut packet = vec![0u8; 48];
        packet[0] = 0x60;
        packet[4..6].copy_from_slice(&8u16.to_be_bytes());
        packet[6] = 17;
        packet[7] = 64;
        packet[8..24].copy_from_slice(
            &"2001:db8::1"
                .parse::<std::net::Ipv6Addr>()
                .unwrap()
                .octets(),
        );
        packet[24..40].copy_from_slice(
            &"2001:db8::2"
                .parse::<std::net::Ipv6Addr>()
                .unwrap()
                .octets(),
        );
        packet[40..42].copy_from_slice(&50000u16.to_be_bytes());
        packet[42..44].copy_from_slice(&443u16.to_be_bytes());
        packet[44..46].copy_from_slice(&8u16.to_be_bytes());
        let info = parse(&packet, 101, 48, 1.0).unwrap();
        assert_eq!(info.key.source.to_string(), "2001:db8::1");
        assert_eq!(info.key.protocol, "UDP");
        assert_eq!(info.key.destination_port, 443);
    }
    #[test]
    fn parses_arp_and_ethernet_addresses() {
        let mut packet = vec![0u8; 42];
        packet[..6].copy_from_slice(&[255; 6]);
        packet[6..12].copy_from_slice(&[0, 1, 2, 3, 4, 5]);
        packet[12..14].copy_from_slice(&[8, 6]);
        packet[14..16].copy_from_slice(&[0, 1]);
        packet[16..18].copy_from_slice(&[8, 0]);
        packet[18..20].copy_from_slice(&[6, 4]);
        packet[20..22].copy_from_slice(&[0, 1]);
        packet[28..32].copy_from_slice(&[10, 0, 0, 1]);
        packet[38..42].copy_from_slice(&[10, 0, 0, 2]);
        let info = parse(&packet, 1, 42, 1.0).unwrap();
        assert_eq!(info.key.protocol, "ARP");
        assert_eq!(info.key.destination.to_string(), "10.0.0.2");
        assert_eq!(info.source_mac.as_deref(), Some("00:01:02:03:04:05"));
    }
}
