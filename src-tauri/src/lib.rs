// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use regex::Regex;
use serde::{Deserialize, Serialize};
use reqwest;
use std::process::Command;
use log::{debug, info};

/// Simple greeting command used by the template.
#[tauri::command]
pub fn greet(name: &str) -> String {
    debug!("Greeting {name}");
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
pub fn parse_arp_output(output: &str) -> Vec<Device> {
    debug!("Parsing ARP table output");
    // Regex matches an IPv4 address and a MAC address consisting of hex
    // digits separated by either ':' or '-'. The order can vary between
    // platforms, so we keep it flexible.
    let re = Regex::new(
        r"(?i)([0-9]{1,3}(?:\.[0-9]{1,3}){3}).*?([0-9a-f]{2}([-:][0-9a-f]{2}){5})",
    )
    .expect("invalid regex");
    let devices: Vec<Device> = output
        .lines()
        .filter_map(|line| {
            re.captures(line).map(|caps| Device {
                ip: caps.get(1).unwrap().as_str().to_string(),
                mac: caps.get(2).unwrap().as_str().replace('-', ":"),
            })
        })
        .collect();
    debug!("Discovered {} devices", devices.len());
    devices
}


/// Scan the local network using the system `arp` command.
///
/// The command output is parsed and returned to the caller. Errors from the
/// command execution are converted into strings.
#[tauri::command]
async fn scan_network() -> Result<Vec<Device>, String> {
    info!("Running arp -a to scan network");
    let output = Command::new("arp")
        .arg("-a")
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let devices = parse_arp_output(&stdout);
    debug!("scan_network found {} devices", devices.len());
    Ok(devices)
}

/// Fetch statistics from the Pi-hole instance at the given host.
#[tauri::command]
async fn get_pihole_stats(host: &str) -> Result<PiholeStats, String> {
    let url = format!("http://{}/admin/api.php?summaryRaw", host);
    info!("Requesting Pi-hole stats from {url}");
    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let stats = resp.json::<PiholeStats>().await.map_err(|e| e.to_string())?;
    debug!("Received stats: {:?}", stats);
    Ok(stats)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    info!("Starting NetScene application");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, scan_network, get_pihole_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
