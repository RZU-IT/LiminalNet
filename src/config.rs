use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    Bits,
    #[default]
    Bytes,
    Packets,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub welcomed: bool,
    pub show_welcome: bool,
    pub french: bool,
    pub unit: Unit,
    pub theme: usize,
    pub adapter: String,
    pub favorites: BTreeSet<String>,
    pub threshold: u64,
    pub threshold_unit: Unit,
    pub country_database: String,
    pub asn_database: String,
    pub blacklist: String,
    pub autostart: bool,
    pub auto_capture: bool,
    pub alert_blacklist: bool,
    pub alert_favorites: bool,
    pub alert_sound: bool,
    pub webhook: String,
    pub custom_colors: Vec<String>,
    pub gradients: bool,
    pub rules: Vec<crate::firewall::Rule>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            welcomed: false,
            show_welcome: false,
            french: true,
            unit: Unit::Bytes,
            theme: 0,
            adapter: String::new(),
            favorites: BTreeSet::new(),
            threshold: 0,
            threshold_unit: Unit::Bytes,
            country_database: String::new(),
            asn_database: String::new(),
            blacklist: String::new(),
            autostart: false,
            auto_capture: false,
            alert_blacklist: false,
            alert_favorites: false,
            alert_sound: false,
            webhook: String::new(),
            custom_colors: vec![],
            gradients: false,
            rules: vec![],
        }
    }
}
impl Config {
    pub fn path() -> Result<PathBuf> {
        if let Some(folder) = std::env::var_os("LIMINALNET_CONFIG_DIR") {
            let folder = PathBuf::from(folder);
            anyhow::ensure!(
                folder.is_absolute(),
                "LIMINALNET_CONFIG_DIR must be absolute"
            );
            return Ok(folder.join("settings.toml"));
        }
        let dirs = ProjectDirs::from("fr", "RZU Informatique", "LiminalNet")
            .context("Cannot locate the user configuration directory")?;
        Ok(dirs.config_dir().join("settings.toml"))
    }
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        match fs::read_to_string(&path) {
            Ok(text) => {
                let mut config: Self = toml::from_str(&text)
                    .with_context(|| format!("Invalid configuration: {}", path.display()))?;
                anyhow::ensure!(
                    config.version == 1,
                    "Unsupported configuration version {}",
                    config.version
                );
                config.theme = config.theme.min(3);
                Ok(config)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("Cannot read {}", path.display())),
        }
    }
    pub fn save(&self) -> Result<()> {
        let path = Self::path()?;
        fs::create_dir_all(path.parent().context("Invalid configuration path")?)?;
        use std::io::Write;
        let mut temporary =
            tempfile::NamedTempFile::new_in(path.parent().context("Invalid configuration path")?)?;
        temporary.write_all(toml::to_string_pretty(self)?.as_bytes())?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(path)
            .context("Cannot commit configuration")?;
        Ok(())
    }
}

pub fn clean(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).take(256).collect()
}

pub fn amount(value: u64, unit: Unit, rate: bool) -> String {
    let suffix = if rate { "/s" } else { "" };
    if unit == Unit::Packets {
        return format!("{value} pkt{suffix}");
    }
    let mut number = value as f64 * if unit == Unit::Bits { 8.0 } else { 1.0 };
    let labels = if unit == Unit::Bits {
        ["bit", "Kbit", "Mbit", "Gbit", "Tbit"]
    } else {
        ["B", "KB", "MB", "GB", "TB"]
    };
    let mut index = 0;
    while number >= 1000.0 && index < labels.len() - 1 {
        number /= 1000.0;
        index += 1;
    }
    format!("{number:.1} {}{suffix}", labels[index])
}
