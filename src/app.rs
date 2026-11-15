mod input;

pub const SETTINGS_KEYS: [char; 19] = [
    'l', 't', 'k', 'r', 'c', 'w', 'v', 'o', 'z', 'd', 'n', 'p', 'm', 'h', 'x', 'g', 'j', 'b', 'u',
];

use crate::{
    capture::{self, Adapter, Message, Worker},
    config::{Config, Unit},
    model::{Flow, FlowKey, Session},
};
use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEventKind,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
    sync::atomic::Ordering,
    time::Duration,
};

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Welcome,
    Sources,
    Overview,
    Inspect,
    Alerts,
    Settings,
    About,
    Firewall,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Input {
    None,
    Filter,
    File,
    Export,
    Search,
    Threshold,
    Country,
    Asn,
    Blacklist,
    Webhook,
    Palette,
}
pub struct App {
    pub config: Config,
    pub screen: Screen,
    pub devices: Vec<Adapter>,
    pub selected: usize,
    pub session: Option<Session>,
    pub worker: Option<Worker>,
    pub status: String,
    pub paused: bool,
    pub opening: bool,
    pub finished: bool,
    pub confirm: bool,
    pub help: bool,
    pub quit: bool,
    pub filter: String,
    pub file: String,
    pub export: String,
    pub search: String,
    pub input: Input,
    pub buffer: String,
    pub focus: usize,
    pub flow_index: usize,
    pub details: bool,
    pub details_key: Option<FlowKey>,
    pub inspect_favorites: bool,
    pub inspect_blacklist: bool,
    pub events: VecDeque<String>,
    pub only_favorites: [bool; 3],
    pub sorting: [usize; 3],
    pub monitor: crate::platform::Monitor,
    pub previews: HashMap<String, VecDeque<u64>>,
    pub processes: Vec<listeners::Listener>,
    pub enrichment: Option<crate::enrichment::Enrichment>,
    pub blacklist: Vec<ipnet::IpNet>,
    seen_blacklist: HashSet<std::net::IpAddr>,
    delivery: crate::alerts::Delivery,
    threshold_active: bool,
    favorite_alerts: HashMap<String, std::time::Instant>,
    pub compact: bool,
    pub host_group: usize,
    pub settings_choice: usize,
    pub probe_message: String,
    pub probe_result: Option<std::sync::mpsc::Receiver<String>>,
    pub pending_rule: Option<(bool, crate::firewall::Rule)>,
    rule_result: Option<std::sync::mpsc::Receiver<Result<(bool, crate::firewall::Rule), String>>>,
}
impl App {
    pub fn new(config: Config) -> Self {
        let screen = if !config.welcomed || config.show_welcome {
            Screen::Welcome
        } else {
            Screen::Sources
        };
        let mut app = Self {
            config,
            screen,
            devices: vec![],
            selected: 0,
            session: None,
            worker: None,
            status: String::new(),
            paused: false,
            opening: false,
            finished: false,
            confirm: false,
            help: false,
            quit: false,
            filter: String::new(),
            file: String::new(),
            export: String::new(),
            search: String::new(),
            input: Input::None,
            buffer: String::new(),
            focus: 0,
            flow_index: 0,
            details: false,
            details_key: None,
            inspect_favorites: false,
            inspect_blacklist: false,
            events: VecDeque::new(),
            only_favorites: [false; 3],
            sorting: [0; 3],
            monitor: crate::platform::Monitor::start(),
            previews: HashMap::new(),
            processes: vec![],
            enrichment: None,
            blacklist: vec![],
            seen_blacklist: HashSet::new(),
            delivery: crate::alerts::Delivery::new(),
            threshold_active: false,
            favorite_alerts: HashMap::new(),
            compact: false,
            host_group: 0,
            settings_choice: 0,
            probe_message: String::new(),
            probe_result: None,
            pending_rule: None,
            rule_result: None,
        };
        app.reload_enrichment();
        app.refresh();
        app
    }
    pub fn tr<'a>(&self, key: &'a str) -> &'a str {
        crate::i18n::text(key, self.config.french)
    }
    pub fn refresh(&mut self) {
        match capture::adapters() {
            Ok(devices) => {
                self.devices = devices;
                self.selected = self
                    .devices
                    .iter()
                    .position(|d| d.name == self.config.adapter)
                    .unwrap_or(0);
            }
            Err(e) => self.status = format!("{e:#}"),
        }
    }
    pub fn notify(&mut self, message: String) {
        if self.config.alert_sound {
            use std::io::Write;
            let _ = std::io::stdout().write_all(b"\x07");
        }
        if !self.config.webhook.is_empty()
            && self
                .delivery
                .requests
                .try_send((self.config.webhook.clone(), message.clone()))
                .is_err()
        {
            self.status = "Webhook queue full; event retained locally".into();
        }
        let seconds = self
            .session
            .as_ref()
            .map(|s| s.started.elapsed().as_secs())
            .unwrap_or(0);
        self.events.push_front(format!(
            "{:02}:{:02} · {message}",
            seconds / 60,
            seconds % 60
        ));
        self.events.truncate(200);
    }
    fn reload_enrichment(&mut self) {
        match crate::enrichment::Enrichment::new(
            self.config.country_database.clone(),
            self.config.asn_database.clone(),
        ) {
            Ok(value) => self.enrichment = Some(value),
            Err(error) => self.status = format!("{error:#}"),
        };
        match crate::enrichment::load_blacklist(&self.config.blacklist) {
            Ok(list) => self.blacklist = list,
            Err(error) => self.status = format!("{error:#}"),
        }
    }
    pub fn start(&mut self) {
        let file = (!self.file.trim().is_empty()).then(|| PathBuf::from(self.file.trim()));
        let device = self.devices.get(self.selected);
        if file.is_none() && device.is_none() {
            self.status = self.tr("no_interface_available").into();
            return;
        }
        let addresses = device.map(|d| d.addresses.clone()).unwrap_or_default();
        let name = device.map(|d| d.name.clone());
        match Worker::start(
            name.clone(),
            file,
            self.filter.clone(),
            (!self.export.is_empty()).then(|| PathBuf::from(&self.export)),
        ) {
            Ok(worker) => {
                self.worker = Some(worker);
                self.session = Some(Session::new(addresses));
                self.paused = false;
                self.opening = true;
                self.finished = false;
                self.screen = Screen::Overview;
                self.flow_index = 0;
                self.events.clear();
                self.status = self.tr("opening_capture").into();
                if let Some(name) = name {
                    self.config.adapter = name;
                }
                self.save();
            }
            Err(error) => self.status = format!("{error:#}"),
        }
    }
    pub fn save(&mut self) {
        if let Err(error) = self.config.save() {
            self.status = format!("{error:#}");
            crate::diagnostics::record("configuration", &self.status);
            self.events
                .push_front(format!("Configuration not saved: {error:#}"));
            self.events.truncate(200);
        }
    }
    pub fn poll(&mut self) {
        if let Some(message) = self.probe_result.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.probe_message = message;
            self.probe_result = None;
        }
        if let Some(result) = self.rule_result.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.rule_result = None;
            match result {
                Ok((remove, rule)) => {
                    self.config.rules.retain(|r| r.id != rule.id);
                    if !remove {
                        self.config.rules.push(rule);
                    }
                    self.status = match self.config.save() {
                        Ok(()) => "Firewall operation completed".into(),
                        Err(error) => format!(
                            "OS rule changed, but its local record could not be saved: {error:#}"
                        ),
                    };
                }
                Err(error) => self.status = error,
            }
        }
        while let Ok(error) = self.delivery.errors.try_recv() {
            self.status = error;
        }
        let mut notifications = Vec::new();
        while let Ok(update) = self.monitor.messages.try_recv() {
            match update {
                crate::platform::Update::Processes(list) => self.processes = list,
                crate::platform::Update::Networks(values) => {
                    for (name, (rx, tx)) in values {
                        crate::platform::preview_push(
                            self.previews.entry(name).or_default(),
                            rx + tx,
                        );
                    }
                }
                crate::platform::Update::Error(error) => {
                    if !self.events.contains(&error) {
                        self.events.push_front(error);
                        self.events.truncate(200);
                    }
                }
            }
        }
        if let Some(enrichment) = &mut self.enrichment {
            enrichment.poll();
        }
        let mut pending = vec![];
        if let Some(worker) = &self.worker {
            for _ in 0..32 {
                if let Ok(message) = worker.messages.try_recv() {
                    pending.push(message);
                } else {
                    break;
                }
            }
        }
        for message in pending {
            match message {
                Message::Ready => {
                    self.opening = false;
                    self.status.clear();
                }
                Message::Warning(warning) => {
                    crate::diagnostics::record("warning", &warning);
                    self.notify(warning);
                }
                Message::Packets(packets) => {
                    if let Some(session) = &mut self.session {
                        let mut touched = HashSet::new();
                        for packet in packets {
                            touched.insert(packet.key.clone());
                            let remote = if session.addresses.contains(&packet.key.source) {
                                packet.key.destination
                            } else {
                                packet.key.source
                            };
                            if let Some(enrichment) = &mut self.enrichment {
                                enrichment.request(remote);
                            }
                            if self.config.alert_blacklist
                                && self.blacklist.iter().any(|net| net.contains(&remote))
                                && self.seen_blacklist.insert(remote)
                            {
                                notifications.push(format!("BLACKLIST · {remote}"));
                            }
                            if self.file.is_empty() {
                                session.ingest(packet);
                            } else {
                                session.ingest_offline(packet);
                            }
                        }
                        if self.config.alert_favorites {
                            for flow in touched.iter().filter_map(|key| session.flows.get(key)) {
                                let remote = if flow.incoming {
                                    flow.key.source
                                } else {
                                    flow.key.destination
                                };
                                for key in [
                                    format!("0:{remote}"),
                                    format!("1:{}", flow.service),
                                    format!("2:{}", flow.program),
                                ] {
                                    if self.config.favorites.contains(&key)
                                        && self.favorite_alerts.get(&key).is_none_or(|last| {
                                            last.elapsed() >= Duration::from_secs(30)
                                        })
                                    {
                                        self.favorite_alerts
                                            .insert(key.clone(), std::time::Instant::now());
                                        notifications.push(format!("FAVORITE · {key}"));
                                    }
                                }
                            }
                        }
                        for key in touched {
                            let Some(flow) = session.flows.get_mut(&key) else {
                                continue;
                            };
                            if flow.program != "?" {
                                continue;
                            }
                            if self.file.is_empty() {
                                let (ip, port) = if flow.incoming {
                                    (flow.key.destination, flow.key.destination_port)
                                } else {
                                    (flow.key.source, flow.key.source_port)
                                };
                                if let Some(process) = crate::platform::process_for(
                                    &self.processes,
                                    ip,
                                    port,
                                    &flow.key.protocol,
                                ) {
                                    flow.program = process.name;
                                    flow.program_path = process.path;
                                }
                            }
                        }
                    }
                    if !self.finished {
                        self.status.clear();
                    }
                }
                Message::Stats(dropped) => {
                    if let Some(session) = &mut self.session {
                        session.dropped = dropped;
                    }
                }
                Message::Paused(value) => {
                    self.paused = value;
                    if let Some(session) = &mut self.session {
                        session.reset_tick();
                    }
                }
                Message::Finished => {
                    if let Some(session) = &mut self.session {
                        session.finish_offline();
                    }
                    self.finished = true;
                    self.status = self.tr("capture_file_complete").into();
                }
                Message::Error(error) => {
                    self.opening = false;
                    crate::diagnostics::record("capture", &error);
                    self.finished = true;
                    self.status = error.clone();
                    self.notify(error);
                }
            }
        }
        let mut alert = None;
        if let Some(session) = &mut self.session {
            let before = session.history.back().map(|s| s.time);
            if self.file.is_empty() {
                session.tick(self.paused || self.finished);
            }
            if session.history.back().map(|s| s.time) != before
                && self.config.threshold > 0
                && let Some(sample) = session.history.back()
                && sample.rate(self.config.threshold_unit) > self.config.threshold as f64
                && !self.threshold_active
            {
                alert = Some(format!(
                    "{:.0}s · threshold exceeded: {:.0} {:?}/s",
                    sample.time,
                    sample.rate(self.config.threshold_unit),
                    self.config.threshold_unit
                ));
                self.threshold_active = true;
            }
            if session
                .history
                .back()
                .is_some_and(|s| s.rate(self.config.threshold_unit) <= self.config.threshold as f64)
            {
                self.threshold_active = false;
            }
        }
        if let Some(alert) = alert {
            self.notify(alert);
        }
        for notification in notifications {
            self.notify(notification);
        }
    }
    pub fn flows(&self) -> Vec<&Flow> {
        self.session
            .as_ref()
            .map(|s| {
                s.rows()
                    .into_iter()
                    .filter(|f| {
                        let remote = if f.incoming {
                            f.key.source
                        } else {
                            f.key.destination
                        };
                        if self.inspect_blacklist
                            && !self.blacklist.iter().any(|net| net.contains(&remote))
                        {
                            return false;
                        }
                        if self.inspect_favorites {
                            let mut names = vec![
                                format!("0:{remote}"),
                                format!("1:{}", f.service),
                                format!("2:{}", f.program),
                            ];
                            if let Some(host) =
                                self.enrichment.as_ref().and_then(|e| e.hosts.get(&remote))
                            {
                                names.extend([
                                    format!("0:{}", host.country),
                                    format!("0:{}", host.organization),
                                    format!("0:{}", host.domain),
                                ]);
                            }
                            if !names
                                .iter()
                                .any(|name| self.config.favorites.contains(name))
                            {
                                return false;
                            }
                        }
                        if self.search.is_empty() {
                            return true;
                        }
                        let remote = if f.incoming {
                            f.key.source
                        } else {
                            f.key.destination
                        };
                        let mut fields = vec![
                            f.key.source.to_string(),
                            f.key.source_port.to_string(),
                            f.key.destination.to_string(),
                            f.key.destination_port.to_string(),
                            f.key.protocol.clone(),
                            f.service.clone(),
                            f.program.clone(),
                            f.program_path.clone(),
                        ];
                        if let Some(host) =
                            self.enrichment.as_ref().and_then(|e| e.hosts.get(&remote))
                        {
                            fields.extend([
                                host.domain.clone(),
                                host.country.clone(),
                                host.organization.clone(),
                            ]);
                        }
                        let fields: Vec<_> = fields.into_iter().map(|s| s.to_lowercase()).collect();
                        let text = fields.join(" ");
                        let query = self.search.to_lowercase();
                        if let Some(q) = query.strip_prefix("!=") {
                            !fields.iter().any(|v| v == q)
                        } else if let Some(q) = query.strip_prefix('!') {
                            !text.contains(q)
                        } else if let Some(q) = query.strip_prefix('=') {
                            fields.iter().any(|v| v == q)
                        } else {
                            text.contains(&query)
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
    pub fn panel_rows(&self, index: usize) -> Vec<(String, u64, u64, f64)> {
        let Some(session) = &self.session else {
            return vec![];
        };
        let packets = self.config.unit == Unit::Packets;
        let mut grouped: HashMap<String, (u64, u64, f64)> = HashMap::new();
        for flow in session.flows.values() {
            let name = match index {
                0 => {
                    let ip = if flow.incoming {
                        flow.key.source
                    } else {
                        flow.key.destination
                    };
                    if let Some(host) = self.enrichment.as_ref().and_then(|e| e.hosts.get(&ip)) {
                        match self.host_group {
                            1 if !host.country.is_empty() => host.country.clone(),
                            2 if !host.organization.is_empty() => host.organization.clone(),
                            3 => host.domain.clone(),
                            _ => ip.to_string(),
                        }
                    } else {
                        ip.to_string()
                    }
                }
                1 => flow.service.clone(),
                _ => flow.program.clone(),
            };
            let value = if packets { flow.packets } else { flow.bytes };
            let entry = grouped.entry(name).or_default();
            if flow.incoming {
                entry.0 += value;
            } else {
                entry.1 += value;
            }
            entry.2 = entry.2.max(flow.last);
        }
        let mut rows: Vec<_> = grouped
            .into_iter()
            .filter(|(name, _)| {
                !self.only_favorites[index]
                    || self.config.favorites.contains(&format!("{index}:{name}"))
            })
            .map(|(name, (rx, tx, time))| (name, rx, tx, time))
            .collect();
        rows.sort_by(|a, b| {
            let order = match self.sorting[index] {
                0 => b.3.total_cmp(&a.3),
                1 => (b.1 + b.2).cmp(&(a.1 + a.2)),
                _ => (a.1 + a.2).cmp(&(b.1 + b.2)),
            };
            order.then_with(|| a.0.cmp(&b.0))
        });
        rows
    }
    pub fn details_flow(&self) -> Option<&Flow> {
        self.details_key
            .as_ref()
            .and_then(|key| self.session.as_ref()?.flows.get(key))
    }
}
pub fn run(mut app: App) -> Result<()> {
    ratatui::run(|terminal| -> Result<()> {
        struct MouseGuard;
        impl Drop for MouseGuard {
            fn drop(&mut self) {
                let _ = crossterm::execute!(std::io::stdout(), DisableMouseCapture);
            }
        }
        crossterm::execute!(std::io::stdout(), EnableMouseCapture)?;
        let _mouse = MouseGuard;
        while !app.quit {
            app.poll();
            terminal.draw(|frame| crate::ui::draw(frame, &app))?;
            if event::poll(Duration::from_millis(50))? {
                app.handle(event::read()?);
            }
        }
        app.worker = None;
        app.save();
        Ok(())
    })
}
