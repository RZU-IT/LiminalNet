use std::{
    fs,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn record(category: &str, message: &str) {
    let Ok(config) = crate::config::Config::path() else {
        return;
    };
    let Some(folder) = config.parent() else {
        return;
    };
    if fs::create_dir_all(folder).is_err() {
        return;
    }
    let path = folder.join("liminalnet.log");
    if fs::metadata(&path).is_ok_and(|m| m.len() > 1_048_576) {
        let old = folder.join("liminalnet.log.old");
        let _ = fs::remove_file(&old);
        let _ = fs::rename(&path, old);
    }
    let mut options = fs::OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    if let Ok(mut file) = options.open(path) {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(file, "{time} {category}: {}", crate::config::clean(message));
    }
}
