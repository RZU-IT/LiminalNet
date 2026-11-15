use std::{
    io::{self, IsTerminal, Write},
    process::Command,
};

fn main() {
    if let Err(error) = launch() {
        eprintln!("LiminalNet: {error}");
        if io::stdin().is_terminal() && std::env::args_os().len() == 1 {
            eprintln!("Press Enter to close.");
            let _ = io::stdin().read_line(&mut String::new());
        }
        std::process::exit(1);
    }
}
fn launch() -> io::Result<()> {
    let executable = std::env::current_exe()?;
    let core = executable.with_file_name(if cfg!(windows) {
        "liminalnet-core.exe"
    } else {
        "liminalnet-core"
    });
    if !core.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "liminalnet-core is missing. Extract the complete LiminalNet package before launching.",
        ));
    }
    let mut command = Command::new(core);
    command.args(std::env::args_os().skip(1));
    #[cfg(windows)]
    {
        let windows = std::path::PathBuf::from(
            std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()),
        );
        let npcap = windows.join("System32/Npcap");
        if !npcap.join("wpcap.dll").is_file() && !windows.join("System32/wpcap.dll").is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Npcap is required for network capture. Install it from https://npcap.com/ and launch LiminalNet again. Npcap is not bundled.",
            ));
        }
        let mut paths = vec![npcap];
        if let Some(existing) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&existing));
        }
        command.env(
            "PATH",
            std::env::join_paths(paths).map_err(io::Error::other)?,
        );
    }
    io::stdout().flush()?;
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec())
    }
    #[cfg(not(unix))]
    {
        let status = command.status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
}
