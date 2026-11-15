use anyhow::{Context, Result};
use ipnet::IpNet;
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::Duration,
};
#[derive(Clone, Debug, Default)]
pub struct Host {
    pub domain: String,
    pub country: String,
    pub organization: String,
}
pub struct Resolver {
    pub requests: SyncSender<IpAddr>,
    pub results: Receiver<(IpAddr, Host)>,
    stop: Arc<AtomicBool>,
}
impl Resolver {
    pub fn start(country: String, asn: String) -> Result<Self> {
        let country = if country.is_empty() {
            Some(maxminddb::Reader::from_source(
                include_bytes!("../assets/databases/country.mmdb").to_vec(),
            )?)
        } else {
            Some(
                maxminddb::Reader::open_readfile(&country)
                    .context("Cannot open country database")?,
            )
        };
        let asn = if asn.is_empty() {
            Some(maxminddb::Reader::from_source(
                include_bytes!("../assets/databases/asn.mmdb").to_vec(),
            )?)
        } else {
            Some(maxminddb::Reader::open_readfile(&asn).context("Cannot open ASN database")?)
        };
        let (requests, rx) = mpsc::sync_channel::<IpAddr>(128);
        let (tx, results) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let control = stop.clone();
        thread::spawn(move || {
            while !control.load(Ordering::Relaxed) {
                let ip = match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(ip) => ip,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(_) => break,
                };
                let mut host = Host {
                    domain: dns_lookup::lookup_addr(&ip).unwrap_or_else(|_| ip.to_string()),
                    ..Host::default()
                };
                if let Some(db) = &country
                    && let Ok(record) = db.lookup(ip)
                {
                    host.country = record
                        .decode_path::<String>(&maxminddb::path!["country", "iso_code"])
                        .ok()
                        .flatten()
                        .unwrap_or_default();
                }
                if let Some(db) = &asn
                    && let Ok(record) = db.lookup(ip)
                {
                    host.organization = record
                        .decode_path::<String>(&maxminddb::path!["autonomous_system_organization"])
                        .ok()
                        .flatten()
                        .unwrap_or_default();
                }
                if tx.send((ip, host)).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            requests,
            results,
            stop,
        })
    }
}
impl Drop for Resolver {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
pub struct Enrichment {
    pub hosts: HashMap<IpAddr, Host>,
    pending: HashSet<IpAddr>,
    pub resolver: Resolver,
}
impl Enrichment {
    pub fn new(country: String, asn: String) -> Result<Self> {
        Ok(Self {
            hosts: HashMap::new(),
            pending: HashSet::new(),
            resolver: Resolver::start(country, asn)?,
        })
    }
    pub fn request(&mut self, ip: IpAddr) {
        if self.hosts.len() + self.pending.len() >= 50_000 {
            return;
        }
        if !self.hosts.contains_key(&ip)
            && !self.pending.contains(&ip)
            && self.resolver.requests.try_send(ip).is_ok()
        {
            self.pending.insert(ip);
        }
    }
    pub fn poll(&mut self) {
        while let Ok((ip, host)) = self.resolver.results.try_recv() {
            self.pending.remove(&ip);
            self.hosts.insert(ip, host);
        }
    }
    pub fn label(&self, ip: IpAddr) -> String {
        let local = match ip {
            IpAddr::V4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
            IpAddr::V6(ip) => {
                ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local()
            }
        };
        let fallback = if ip.is_multicast() {
            "MC"
        } else if local {
            "LAN"
        } else {
            "??"
        };
        if let Some(host) = self.hosts.get(&ip) {
            format!(
                "{} {}{}",
                if host.country.is_empty() {
                    fallback
                } else {
                    &host.country
                },
                host.domain,
                if host.organization.is_empty() {
                    String::new()
                } else {
                    format!(" · {}", host.organization)
                }
            )
        } else {
            ip.to_string()
        }
    }
}
pub fn load_blacklist(path: &str) -> Result<Vec<IpNet>> {
    if path.is_empty() {
        return Ok(vec![]);
    }
    let text = std::fs::read_to_string(path).context("Cannot read IP blacklist")?;
    Ok(text
        .lines()
        .filter_map(|line| {
            let token = line.split_whitespace().next()?;
            token
                .parse::<IpNet>()
                .ok()
                .or_else(|| token.parse::<IpAddr>().ok().map(IpNet::from))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_databases_resolve_public_address() {
        let ip = "8.8.8.8".parse().unwrap();
        let country = maxminddb::Reader::from_source(
            include_bytes!("../assets/databases/country.mmdb").as_slice(),
        )
        .unwrap();
        let code: Option<String> = country
            .lookup(ip)
            .unwrap()
            .decode_path(&maxminddb::path!["country", "iso_code"])
            .unwrap();
        assert_eq!(code.as_deref(), Some("US"));
        let asn = maxminddb::Reader::from_source(
            include_bytes!("../assets/databases/asn.mmdb").as_slice(),
        )
        .unwrap();
        let name: Option<String> = asn
            .lookup(ip)
            .unwrap()
            .decode_path(&maxminddb::path!["autonomous_system_organization"])
            .unwrap();
        assert!(name.is_some_and(|value| !value.is_empty()));
    }
}
