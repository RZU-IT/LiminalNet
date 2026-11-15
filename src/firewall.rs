use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Target {
    Ip(String),
    Program(String),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub target: Target,
    pub scope: String,
}
fn identifier(target: &Target) -> String {
    let text = match target {
        Target::Ip(ip) => format!("ip:{ip}"),
        Target::Program(path) => format!("program:{path}"),
    };
    let hash = text.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    format!("LiminalNet-{hash:016x}")
}
pub fn install(target: Target) -> Result<Rule> {
    match &target {
        Target::Ip(text) => {
            text.parse::<ipnet::IpNet>()
                .or_else(|_| text.parse::<std::net::IpAddr>().map(ipnet::IpNet::from))
                .context("Invalid IP address or CIDR range")?;
        }
        Target::Program(path) => {
            anyhow::ensure!(
                Path::new(path).is_absolute() && Path::new(path).is_file(),
                "Select an existing absolute executable path"
            );
        }
    }
    let id = identifier(&target);
    let scope = install_platform(&id, &target)?;
    Ok(Rule { id, target, scope })
}
pub fn remove(rule: &Rule) -> Result<()> {
    anyhow::ensure!(
        rule.id == identifier(&rule.target),
        "Rule identifier does not match its target"
    );
    remove_platform(rule)
}
fn checked(command: &mut Command, action: &str) -> Result<String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command
        .output()
        .with_context(|| format!("Cannot run {action}"))?;
    anyhow::ensure!(
        output.status.success(),
        "{action} failed (administrative permissions may be required): {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
#[cfg(windows)]
fn install_platform(id: &str, target: &Target) -> Result<String> {
    let (script, value) = match target {
        Target::Ip(value) => (
            "$ErrorActionPreference='Stop'; foreach($d in @('Inbound','Outbound')) { $name=$env:LIMINALNET_RULE+'-'+$d; if(-not(Get-NetFirewallRule -Name $name -ErrorAction SilentlyContinue)){ New-NetFirewallRule -Name $name -DisplayName $name -Group 'LiminalNet' -Direction $d -Action Block -RemoteAddress $env:LIMINALNET_TARGET | Out-Null }; if((Get-NetFirewallRule -Name $name).Enabled -ne 'True'){throw 'Rule is disabled'} }",
            value,
        ),
        Target::Program(value) => (
            "$ErrorActionPreference='Stop'; foreach($d in @('Inbound','Outbound')) { $name=$env:LIMINALNET_RULE+'-'+$d; if(-not(Get-NetFirewallRule -Name $name -ErrorAction SilentlyContinue)){ New-NetFirewallRule -Name $name -DisplayName $name -Group 'LiminalNet' -Direction $d -Action Block -Program $env:LIMINALNET_TARGET | Out-Null }; if((Get-NetFirewallRule -Name $name).Enabled -ne 'True'){throw 'Rule is disabled'} }",
            value,
        ),
    };
    checked(
        Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .env("LIMINALNET_RULE", id)
            .env("LIMINALNET_TARGET", value),
        "Windows Firewall rule installation",
    )?;
    Ok("Windows Firewall · inbound and outbound · persistent".into())
}
#[cfg(windows)]
fn remove_platform(rule: &Rule) -> Result<()> {
    checked(Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command","$ErrorActionPreference='Stop'; foreach($d in @('Inbound','Outbound')){Get-NetFirewallRule -Name ($env:LIMINALNET_RULE+'-'+$d) -ErrorAction SilentlyContinue | Where-Object Group -eq 'LiminalNet' | Remove-NetFirewallRule}"]).env("LIMINALNET_RULE",&rule.id),"Windows Firewall rule removal")?;
    Ok(())
}
#[cfg(target_os = "linux")]
fn linux_table(id: &str) -> String {
    format!("liminalnet_{}", id.trim_start_matches("LiminalNet-"))
}
#[cfg(target_os = "linux")]
fn install_platform(id: &str, target: &Target) -> Result<String> {
    use std::io::Write;
    let table = linux_table(id);
    let mut script = format!(
        "add table inet {table}\nadd chain inet {table} output {{ type filter hook output priority -10; policy accept; }}\nadd chain inet {table} input {{ type filter hook input priority -10; policy accept; }}\n"
    );
    let scope = match target {
        Target::Ip(text) => {
            let address: ipnet::IpNet = text
                .parse()
                .or_else(|_| text.parse::<std::net::IpAddr>().map(ipnet::IpNet::from))?;
            let family = if address.addr().is_ipv4() {
                "ip"
            } else {
                "ip6"
            };
            script.push_str(&format!("add rule inet {table} output {family} daddr {address} counter drop\nadd rule inet {table} input {family} saddr {address} counter drop\n"));
            "nftables · IP/CIDR · runtime (until reboot)".to_string()
        }
        Target::Program(path) => {
            let group = program_group(path)?;
            let depth = group.split('/').filter(|s| !s.is_empty()).count();
            anyhow::ensure!(
                group
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "/._-:@\\".contains(c)),
                "Unsupported cgroup path characters"
            );
            let quoted = group.trim_start_matches('/').replace('\\', "\\\\");
            script.push_str(&format!("add rule inet {table} output socket cgroupv2 level {depth} \"{quoted}\" counter drop\n"));
            format!(
                "nftables · current dedicated cgroup {group} · outbound · reapply after app restart"
            )
        }
    };
    // Check the full transaction before applying.
    let validate = |check: bool| -> Result<()> {
        let mut command = Command::new("nft");
        if check {
            command.arg("--check");
        }
        command
            .args(["-f", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = command
            .spawn()
            .context("nftables is unavailable; install nftables")?;
        child
            .stdin
            .take()
            .context("Cannot open nftables input")?
            .write_all(script.as_bytes())?;
        let output = child.wait_with_output()?;
        anyhow::ensure!(
            output.status.success(),
            "nftables rejected the rule; root permissions and kernel support are required: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    };
    validate(true)?;
    validate(false)?;
    Ok(scope)
}
#[cfg(target_os = "linux")]
fn remove_platform(rule: &Rule) -> Result<()> {
    checked(
        Command::new("nft").args(["delete", "table", "inet", &linux_table(&rule.id)]),
        "nftables rule removal",
    )?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn program_group(path: &str) -> Result<String> {
    let target = std::fs::canonicalize(path)?;
    let mut groups = std::collections::HashSet::new();
    let mut processes = vec![];
    for entry in std::fs::read_dir("/proc")?.flatten() {
        if entry.file_name().to_string_lossy().parse::<u32>().is_err() {
            continue;
        }
        let Ok(executable) = std::fs::read_link(entry.path().join("exe")) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(entry.path().join("cgroup")) else {
            continue;
        };
        let Some(group) = text.lines().find_map(|line| line.strip_prefix("0::")) else {
            continue;
        };
        if executable == target {
            groups.insert(group.to_string());
        }
        processes.push((executable, group.to_string()));
    }
    anyhow::ensure!(
        groups.len() == 1,
        "The application must be running in one dedicated cgroup v2 scope"
    );
    let group = groups
        .into_iter()
        .next()
        .context("No application cgroup found")?;
    anyhow::ensure!(
        group != "/"
            && !processes
                .iter()
                .any(|(exe, g)| g == &group && exe != &target),
        "Refusing to block a cgroup shared with other applications"
    );
    Ok(group)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identifiers_separate_targets() {
        assert_ne!(
            identifier(&Target::Ip("127.0.0.1".into())),
            identifier(&Target::Program("127.0.0.1".into()))
        );
    }
    #[test]
    fn rejects_shell_text_before_execution() {
        assert!(install(Target::Ip("127.0.0.1; echo unsafe".into())).is_err());
    }
}
