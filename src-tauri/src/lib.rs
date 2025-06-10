// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use regex::Regex;
use serde::{Deserialize, Serialize};
use reqwest;
use std::process::Command;

/// Simple greeting command used by the template.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Representation of a network device discovered on the local network.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct Device {
    /// IPv4 address of the device.
    pub ip: String,
    /// MAC address reported by the ARP table.
    pub mac: String,
}

/// Basic statistics returned by the Pi-hole FTL API.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PiholeStats {
    pub domains_being_blocked: u64,
    pub dns_queries_today: u64,
    pub ads_blocked_today: u64,
    pub ads_percentage_today: f64,
    pub status: String,
}

/// Parse the output of the `arp -a` command into a list of [`Device`].
fn parse_arp_output(output: &str) -> Vec<Device> {
    // Regex matches an IPv4 address and a MAC address consisting of hex
    // digits separated by either ':' or '-'. The order can vary between
    // platforms, so we keep it flexible.
    let re = Regex::new(
        r"(?i)([0-9]{1,3}(?:\.[0-9]{1,3}){3}).*?([0-9a-f]{2}([-:][0-9a-f]{2}){5})",
    )
    .expect("invalid regex");
    output
        .lines()
        .filter_map(|line| {
            re.captures(line).map(|caps| Device {
                ip: caps.get(1).unwrap().as_str().to_string(),
                mac: caps.get(2).unwrap().as_str().replace('-', ":"),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_linux_style() {
        let sample = "? (192.168.1.1) at aa:bb:cc:dd:ee:ff [ether] on eth0\n? (192.168.1.2) at 11:22:33:44:55:66 [ether] on eth0";
        let devices = parse_arp_output(sample);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0], Device { ip: "192.168.1.1".into(), mac: "aa:bb:cc:dd:ee:ff".into() });
        assert_eq!(devices[1].mac, "11:22:33:44:55:66");
    }

    #[test]
    fn parse_macos_style() {
        let sample = "? (192.168.1.3) at 77-88-99-aa-bb-cc on en0 ifscope [ethernet]";
        let devices = parse_arp_output(sample);
        assert_eq!(devices, vec![Device { ip: "192.168.1.3".into(), mac: "77:88:99:aa:bb:cc".into() }]);
    }
}

/// Scan the local network using the system `arp` command.
///
/// The command output is parsed and returned to the caller. Errors from the
/// command execution are converted into strings.
#[tauri::command]
async fn scan_network() -> Result<Vec<Device>, String> {
    let output = Command::new("arp")
        .arg("-a")
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_arp_output(&stdout))
}

/// Fetch statistics from the Pi-hole instance at the given host.
#[tauri::command]
async fn get_pihole_stats(host: &str) -> Result<PiholeStats, String> {
    let url = format!("http://{}/admin/api.php?summaryRaw", host);
    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let stats = resp.json::<PiholeStats>().await.map_err(|e| e.to_string())?;
    Ok(stats)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, scan_network, get_pihole_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
