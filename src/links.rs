use anyhow::{Context, Result};

pub const REPOSITORY: Option<&str> = Some("https://github.com/RZU-IT/LiminalNet");

pub fn open(url: Option<&str>) -> Result<()> {
    let url = url.context("This project link has not been configured yet")?;
    anyhow::ensure!(url.starts_with("https://"), "Project links must use HTTPS");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("explorer.exe")
            .arg(url)
            .creation_flags(0x08000000)
            .spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
    }
    Ok(())
}
