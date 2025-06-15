// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use log::{debug, error, info};
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;
use thiserror::Error;
use url::Url;

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

/// Pi-hole authentication response
#[derive(Debug, Serialize, Deserialize)]
struct PiholeAuthResponse {
    session: PiholeSession,
}

#[derive(Debug, Serialize, Deserialize)]
struct PiholeSession {
    valid: bool,
    sid: String,
    csrf: String,
    validity: u64,
}

/// Pi-hole authentication request
#[derive(Debug, Serialize)]
struct PiholeAuthRequest {
    password: String,
}

/// Custom error types for better error handling
#[derive(Error, Debug)]
pub enum PiholeError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("JSON parsing failed: {0}")]
    JsonError(String),
    #[error("Invalid host format: {0}")]
    InvalidHost(String),
    #[error("Server returned non-success status: {status}")]
    ServerError { status: u16 },
    #[error("Response validation failed: {reason}")]
    ValidationError { reason: String },
}

impl From<PiholeError> for String {
    fn from(err: PiholeError) -> Self {
        err.to_string()
    }
}

/// Network scanning error types
#[derive(Error, Debug)]
pub enum NetworkScanError {
    #[error("Command execution failed: {0}")]
    CommandError(#[from] std::io::Error),
    #[error("Command output was not valid UTF-8")]
    InvalidOutput,
}

impl From<NetworkScanError> for String {
    fn from(err: NetworkScanError) -> Self {
        err.to_string()
    }
}

/// Parse the output of the `arp -a` command into a list of [`Device`].
pub fn parse_arp_output(output: &str) -> Vec<Device> {
    debug!("Parsing ARP table output");
    // Regex matches an IPv4 address and a MAC address consisting of hex
    // digits separated by either ':' or '-'. The order can vary between
    // platforms, so we keep it flexible.
    let re = Regex::new(r"(?i)([0-9]{1,3}(?:\.[0-9]{1,3}){3}).*?([0-9a-f]{2}([-:][0-9a-f]{2}){5})")
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
        .map_err(NetworkScanError::from)?;

    let stdout = String::from_utf8(output.stdout).map_err(|_| NetworkScanError::InvalidOutput)?;

    let devices = parse_arp_output(&stdout);
    debug!("scan_network found {} devices", devices.len());
    Ok(devices)
}

/// Validate that the response contains expected Pi-hole fields
fn validate_pihole_response(stats: &PiholeStats) -> Result<(), PiholeError> {
    // Check if status is a reasonable value
    if stats.status.is_empty() {
        return Err(PiholeError::ValidationError {
            reason: "Status field is empty".to_string(),
        });
    }

    // Check for reasonable ranges (Pi-hole specific validation)
    if stats.ads_percentage_today > 100.0 {
        return Err(PiholeError::ValidationError {
            reason: "Ads percentage cannot exceed 100%".to_string(),
        });
    }

    debug!("Pi-hole response validation passed");
    Ok(())
}

/// Parse and validate a host string, attempting both legacy and new API endpoints
fn parse_pihole_urls(host: &str) -> Result<(Url, Url), PiholeError> {
    let trimmed_host = host.trim();

    if trimmed_host.is_empty() {
        return Err(PiholeError::InvalidHost("Host cannot be empty".to_string()));
    }

    // Add http:// if no protocol is specified
    let url_string = if trimmed_host.starts_with("http://") || trimmed_host.starts_with("https://")
    {
        trimmed_host.to_string()
    } else {
        format!("http://{}", trimmed_host)
    };

    let base_url = Url::parse(&url_string)?;

    // Legacy API endpoint
    let mut legacy_url = base_url.clone();
    legacy_url.set_path("/admin/api.php");
    legacy_url.set_query(Some("summaryRaw"));

    // New API endpoint
    let mut new_url = base_url.clone();
    new_url.set_path("/api/stats/summary");

    debug!("Legacy Pi-hole URL: {}", legacy_url);
    debug!("New Pi-hole URL: {}", new_url);
    
    Ok((legacy_url, new_url))
}

/// Authenticate with Pi-hole to get session ID
async fn authenticate_pihole(host: &str, password: Option<&str>) -> Result<Option<String>, PiholeError> {
    if password.is_none() {
        return Ok(None);
    }

    let password = password.unwrap();
    let trimmed_host = host.trim();
    
    let url_string = if trimmed_host.starts_with("http://") || trimmed_host.starts_with("https://") {
        trimmed_host.to_string()
    } else {
        format!("http://{}", trimmed_host)
    };

    let mut auth_url = Url::parse(&url_string)?;
    auth_url.set_path("/api/auth");

    let client = create_http_client();
    let auth_request = PiholeAuthRequest {
        password: password.to_string(),
    };

    debug!("Attempting authentication with: {}", auth_url);
    
    let response = client
        .post(auth_url)
        .json(&auth_request)
        .send()
        .await?;

    if response.status().is_success() {
        let auth_response: PiholeAuthResponse = response.json().await?;
        debug!("Authentication successful, SID obtained");
        Ok(Some(auth_response.session.sid))
    } else {
        debug!("Authentication failed with status: {}", response.status());
        Ok(None)
    }
}

/// Create a configured HTTP client for Pi-hole requests
fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("NetScene/1.0")
        .build()
        .expect("Failed to create HTTP client")
}

/// Fetch statistics from the Pi-hole instance at the given host.
/// Supports both legacy and new API formats with optional authentication.
#[tauri::command]
async fn get_pihole_stats(host: &str, password: Option<String>) -> Result<PiholeStats, String> {
    get_pihole_stats_internal(host, password.as_deref()).await.map_err(|e| e.into())
}

/// Internal function for testing - parses host URLs for both legacy and new API
#[cfg(test)]
pub fn parse_pihole_urls_internal(host: &str) -> Result<(Url, Url), PiholeError> {
    parse_pihole_urls(host)
}

/// Internal function for testing - validates pihole response
#[cfg(test)]
pub fn validate_pihole_response_internal(stats: &PiholeStats) -> Result<(), PiholeError> {
    validate_pihole_response(stats)
}

/// Internal function for testing - parses host (for legacy compatibility)
#[cfg(test)]
pub fn parse_host_internal(host: &str) -> Result<Url, PiholeError> {
    let (legacy_url, _) = parse_pihole_urls(host)?;
    Ok(legacy_url)
}

/// Internal function for testing - gets pihole stats without Tauri command wrapper
pub async fn get_pihole_stats_internal(host: &str, password: Option<&str>) -> Result<PiholeStats, PiholeError> {
    info!("Requesting Pi-hole stats from host: {}", host);

    let (legacy_url, new_url) = parse_pihole_urls(host)?;
    let client = create_http_client();

    // Try to authenticate if password is provided
    let sid = if password.is_some() {
        match authenticate_pihole(host, password).await {
            Ok(sid) => sid,
            Err(e) => {
                debug!("Authentication failed, continuing without auth: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Try new API first, then fall back to legacy API
    let endpoints = [
        ("new API", new_url),
        ("legacy API", legacy_url),
    ];

    for (api_type, url) in endpoints.iter() {
        debug!("Trying {} endpoint: {}", api_type, url);
        
        let mut request = client.get(url.clone());
        
        // Add authentication if we have a session ID
        if let Some(ref session_id) = sid {
            request = request.header("X-FTL-SID", session_id);
        }

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                debug!("{} response status: {}", api_type, status);

                if !status.is_success() {
                    debug!("{} returned non-success status: {}, trying next endpoint", api_type, status);
                    continue;
                }

                let response_text = response.text().await?;
                debug!("{} response body length: {} bytes", api_type, response_text.len());
                
                if response_text.is_empty() {
                    debug!("{} returned empty response, trying next endpoint", api_type);
                    continue;
                }

                // Log first 200 characters of response for debugging
                let preview = if response_text.len() > 200 {
                    format!("{}...", &response_text[..200])
                } else {
                    response_text.clone()
                };
                debug!("{} response preview: {}", api_type, preview);

                // Check if response is HTML (likely a login page)
                if response_text.trim_start().starts_with("<!DOCTYPE") || response_text.trim_start().starts_with("<html") {
                    debug!("{} returned HTML response (likely login page), trying next endpoint", api_type);
                    continue;
                }

                // Try to parse as JSON
                match serde_json::from_str::<PiholeStats>(&response_text) {
                    Ok(stats) => {
                        validate_pihole_response(&stats)?;
                        info!(
                            "Successfully retrieved Pi-hole stats using {}: status={}, blocked_today={}",
                            api_type, stats.status, stats.ads_blocked_today
                        );
                        debug!("Full stats: {:?}", stats);
                        return Ok(stats);
                    }
                    Err(e) => {
                        debug!("{} JSON parsing failed: {}, trying next endpoint", api_type, e);
                        continue;
                    }
                }
            }
            Err(e) => {
                debug!("{} request failed: {}, trying next endpoint", api_type, e);
                continue;
            }
        }
    }

    // If we get here, all endpoints failed
    Err(PiholeError::JsonError(
        "Failed to get valid response from any Pi-hole API endpoint. Check if Pi-hole is running and accessible, or if authentication is required.".to_string()
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    info!("Starting NetScene application");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![scan_network, get_pihole_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
