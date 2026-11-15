mod alerts;
mod app;
mod autostart;
mod capture;
mod config;
mod diagnostics;
mod enrichment;
mod firewall;
mod i18n;
mod links;
mod model;
mod platform;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(
    name = "liminalnet",
    version,
    about = "LiminalNet — network visibility by RZU Informatique"
)]
struct Args {
    #[arg(long)]
    autostart: Option<bool>,
    #[arg(long)]
    diagnose: bool,
    #[arg(long)]
    reset_settings: bool,
    #[arg(long)]
    block_ip: Option<String>,
    #[arg(long)]
    block_program: Option<String>,
    #[arg(long)]
    unblock: Option<String>,
    #[arg(long)]
    list_interfaces: bool,
    #[arg(long)]
    config_path: bool,
    #[arg(long)]
    interface: Option<String>,
    #[arg(long)]
    read: Option<PathBuf>,
    #[arg(long, default_value = "")]
    filter: String,
    #[arg(long)]
    record: Option<PathBuf>,
    /// Run a bounded capture and print a JSON summary without opening the TUI.
    #[arg(long)]
    capture_seconds: Option<u64>,
    /// Render a screen to plain text for layout verification.
    #[arg(long)]
    render: Option<String>,
    #[arg(long, default_value_t = 120)]
    render_width: u16,
    #[arg(long, default_value_t = 38)]
    render_height: u16,
    #[arg(long)]
    render_cells: Option<PathBuf>,
}
fn main() {
    if let Err(error) = start() {
        diagnostics::record("fatal", &format!("{error:#}"));
        eprintln!("LiminalNet: {error:#}");
        use std::io::IsTerminal;
        if std::io::stdin().is_terminal() && std::env::args_os().len() == 1 {
            eprintln!(
                "Press Enter to close. Use --diagnose for diagnostics, or --reset-settings to recover an invalid configuration."
            );
            let _ = std::io::stdin().read_line(&mut String::new());
        }
        std::process::exit(1);
    }
}
fn start() -> Result<()> {
    let args = Args::parse();
    if let Some(enabled) = args.autostart {
        let mut config = config::Config::load()?;
        autostart::set(enabled)?;
        config.autostart = enabled;
        config.save()?;
        println!("Session-login startup: {enabled}");
        return Ok(());
    }
    if args.diagnose {
        println!(
            "LiminalNet {}\nOS: {}\nArchitecture: {}\nCapture: pcap 2.5 (system Npcap/libpcap)\nConfiguration: {}",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            config::Config::path()?.display()
        );
        match capture::adapters() {
            Ok(devices) => println!("Capture interfaces: {}", devices.len()),
            Err(error) => println!("Capture interfaces unavailable: {error:#}"),
        };
        return Ok(());
    }
    if args.reset_settings {
        let path = config::Config::path()?;
        if path.exists() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos();
            std::fs::copy(
                &path,
                path.with_extension(format!("toml.backup-{timestamp}")),
            )?;
        }
        config::Config::default().save()?;
        println!("Settings reset; original file backed up if present.");
        return Ok(());
    }
    if args.block_ip.is_some() || args.block_program.is_some() || args.unblock.is_some() {
        let mut config = config::Config::load()?;
        if let Some(id) = args.unblock {
            let rule = config
                .rules
                .iter()
                .find(|r| r.id == id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Unknown LiminalNet rule"))?;
            firewall::remove(&rule)?;
            config.rules.retain(|r| r.id != id);
            config.save()?;
            println!("Removed {id}");
        } else {
            let target = if let Some(ip) = args.block_ip {
                firewall::Target::Ip(ip)
            } else {
                firewall::Target::Program(args.block_program.unwrap_or_default())
            };
            let rule = firewall::install(target)?;
            config.rules.retain(|r| r.id != rule.id);
            config.rules.push(rule.clone());
            config.save()?;
            println!("{}", serde_json::to_string(&rule)?);
        }
        return Ok(());
    }
    if args.config_path {
        println!("{}", config::Config::path()?.display());
        return Ok(());
    }
    if args.list_interfaces {
        for d in capture::adapters()? {
            println!(
                "{}\t{}\t{:?}",
                d.name,
                config::clean(&d.description),
                d.addresses
            );
        }
        return Ok(());
    }
    if let Some(seconds) = args.capture_seconds {
        let addresses = capture::adapters()?
            .into_iter()
            .find(|d| Some(&d.name) == args.interface.as_ref())
            .map(|d| d.addresses)
            .unwrap_or_default();
        let mut session = model::Session::new(addresses);
        let worker = capture::Worker::start(args.interface, args.read, args.filter, args.record)?;
        let deadline = Instant::now() + Duration::from_secs(seconds);
        while Instant::now() < deadline {
            match worker.messages.recv_timeout(Duration::from_millis(100)) {
                Ok(capture::Message::Packets(packets)) => {
                    for packet in packets {
                        session.ingest(packet);
                    }
                }
                Ok(capture::Message::Error(error)) => anyhow::bail!(error),
                Ok(capture::Message::Finished) => break,
                _ => {}
            }
        }
        println!(
            "{}",
            serde_json::json!({"totals":session.totals,"flows":session.rows(),"dropped":session.dropped})
        );
        return Ok(());
    }
    let config = config::Config::load()?;
    let mut application = app::App::new(config);
    application.filter = args.filter;
    if let Some(file) = args.read {
        application.file = file.display().to_string();
    }
    if let Some(interface) = args.interface {
        if let Some(index) = application.devices.iter().position(|d| d.name == interface) {
            application.selected = index;
        } else {
            anyhow::bail!("Unknown interface: {interface}");
        }
    }
    if let Some(path) = args.record {
        application.export = path.display().to_string();
    }
    if let Some(screen) = args.render {
        if matches!(screen.as_str(), "overview" | "inspect") {
            let mut session = model::Session::new(
                application
                    .devices
                    .get(application.selected)
                    .map(|d| d.addresses.clone())
                    .unwrap_or_default(),
            );
            if !application.file.is_empty() {
                let mut capture = pcap::Capture::from_file(&application.file)?;
                let link = capture.get_datalink().0;
                loop {
                    match capture.next_packet() {
                        Ok(packet) => {
                            if let Some(info) = model::parse(
                                packet.data,
                                link,
                                u64::from(packet.header.len),
                                packet.header.ts.tv_sec as f64
                                    + packet.header.ts.tv_usec as f64 / 1e6,
                            ) {
                                session.ingest_offline(info);
                            }
                        }
                        Err(pcap::Error::NoMorePackets) => break,
                        Err(error) => return Err(error.into()),
                    }
                }
                session.finish_offline();
            }
            application.session = Some(session);
            application.finished = true;
            application.config.adapter = application
                .devices
                .get(application.selected)
                .map(|d| d.name.clone())
                .unwrap_or_default();
        }
        application.screen = match screen.as_str() {
            "sources" => app::Screen::Sources,
            "settings" => app::Screen::Settings,
            "about" => app::Screen::About,
            "overview" => app::Screen::Overview,
            "inspect" => app::Screen::Inspect,
            _ => app::Screen::Welcome,
        };
        anyhow::ensure!(
            args.render_width > 0
                && args.render_height > 0
                && args.render_width <= 300
                && args.render_height <= 120,
            "Render size must be between 1x1 and 300x120"
        );
        let backend = ratatui::backend::TestBackend::new(args.render_width, args.render_height);
        let mut terminal = ratatui::Terminal::new(backend)?;
        terminal.draw(|f| ui::draw(f, &application))?;
        let buffer = terminal.backend().buffer();
        if let Some(path) = args.render_cells {
            let color = |value: ratatui::style::Color| match value {
                ratatui::style::Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
                ratatui::style::Color::Black => "#000000".into(),
                ratatui::style::Color::White => "#ffffff".into(),
                ratatui::style::Color::LightRed => "#ff7777".into(),
                _ => "#dddddd".into(),
            };
            let mut cells = vec![];
            for y in 0..args.render_height {
                for x in 0..args.render_width {
                    let cell = &buffer[(x, y)];
                    cells.push(serde_json::json!({"x":x,"y":y,"text":cell.symbol(),"fg":color(cell.fg),"bg":color(cell.bg)}));
                }
            }
            std::fs::write(
                path,
                serde_json::to_vec(
                    &serde_json::json!({"width":args.render_width,"height":args.render_height,"cells":cells}),
                )?,
            )?;
        }
        for y in 0..args.render_height {
            let line: String = (0..args.render_width)
                .map(|x| buffer[(x, y)].symbol())
                .collect();
            println!("{}", line.trim_end());
        }
        return Ok(());
    }
    if application.config.auto_capture
        && application.screen == app::Screen::Sources
        && !application.config.adapter.is_empty()
    {
        application.start();
    }
    app::run(application)
}
