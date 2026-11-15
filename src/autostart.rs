use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

pub fn set(enabled: bool) -> Result<()> {
    let mut executable = std::env::current_exe().context("Cannot locate LiminalNet executable")?;
    let launcher = executable.with_file_name(if cfg!(windows) {
        "liminalnet.exe"
    } else {
        "liminalnet"
    });
    if launcher.is_file() {
        executable = launcher;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let folder = PathBuf::from(std::env::var_os("APPDATA").context("APPDATA is unavailable")?)
            .join("Microsoft/Windows/Start Menu/Programs/Startup");
        fs::create_dir_all(&folder)?;
        let link = folder.join("LiminalNet.lnk");
        if !enabled {
            if link.exists() {
                fs::remove_file(&link)?;
            }
            return Ok(());
        }
        let output=std::process::Command::new("powershell.exe")
            .args(["-NoProfile","-NonInteractive","-Command","$s=(New-Object -ComObject WScript.Shell).CreateShortcut($env:LIMINALNET_STARTUP_LINK); $s.TargetPath=$env:LIMINALNET_EXECUTABLE; $s.WorkingDirectory=[IO.Path]::GetDirectoryName($env:LIMINALNET_EXECUTABLE); $s.Save()"])
            .env("LIMINALNET_STARTUP_LINK",&link).env("LIMINALNET_EXECUTABLE",&executable).creation_flags(0x08000000).output()?;
        anyhow::ensure!(
            output.status.success(),
            "Cannot create startup shortcut: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    #[cfg(target_os = "linux")]
    {
        let root = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
            .context("Cannot locate autostart directory")?;
        let folder = root.join("autostart");
        let file = folder.join("liminalnet.desktop");
        if !enabled {
            if file.exists() {
                fs::remove_file(file)?;
            }
            return Ok(());
        }
        fs::create_dir_all(folder)?;
        let path = executable
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('`', "\\`")
            .replace('$', "\\$")
            .replace('%', "%%");
        fs::write(
            file,
            format!(
                "[Desktop Entry]\nType=Application\nName=LiminalNet\nExec=\"{path}\"\nTerminal=true\nX-GNOME-Autostart-enabled=true\n"
            ),
        )?;
    }
    Ok(())
}
