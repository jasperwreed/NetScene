use mockito::mock;
use netscene_lib::{PiholeError, PiholeStats};
use serde_json::json;

#[tokio::test]
async fn test_get_pihole_stats_success() {
    let _m = mock("GET", "/admin/api.php?summaryRaw")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "domains_being_blocked": 100000,
                "dns_queries_today": 5000,
                "ads_blocked_today": 1500,
                "ads_percentage_today": 30.0,
                "status": "enabled"
            })
            .to_string(),
        )
        .create();

    let host = &mockito::server_url()[7..]; // Remove "http://" prefix
    let result = netscene_lib::get_pihole_stats_internal(host, None).await;

    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.domains_being_blocked, 100000);
    assert_eq!(stats.dns_queries_today, 5000);
    assert_eq!(stats.ads_blocked_today, 1500);
    assert_eq!(stats.ads_percentage_today, 30.0);
    assert_eq!(stats.status, "enabled");
}

#[tokio::test]
async fn test_get_pihole_stats_server_error() {
    let _m = mock("GET", "/admin/api.php?summaryRaw")
        .with_status(500)
        .create();

    let host = &mockito::server_url()[7..]; // Remove "http://" prefix
    let result = netscene_lib::get_pihole_stats_internal(host, None).await;

    assert!(result.is_err());
    // Note: The new implementation tries multiple endpoints, so we might get a different error
}

#[tokio::test]
async fn test_get_pihole_stats_invalid_json() {
    let _m = mock("GET", "/admin/api.php?summaryRaw")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("invalid json")
        .create();

    let host = &mockito::server_url()[7..]; // Remove "http://" prefix
    let result = netscene_lib::get_pihole_stats_internal(host, None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PiholeError::JsonError(_) => {}
        _ => panic!("Expected JsonError"),
    }
}

#[tokio::test]
async fn test_get_pihole_stats_validation_error() {
    let _m = mock("GET", "/admin/api.php?summaryRaw")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "domains_being_blocked": 100000,
                "dns_queries_today": 5000,
                "ads_blocked_today": 1500,
                "ads_percentage_today": 150.0, // Invalid: >100%
                "status": "enabled"
            })
            .to_string(),
        )
        .create();

    let host = &mockito::server_url()[7..]; // Remove "http://" prefix
    let result = netscene_lib::get_pihole_stats_internal(host, None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PiholeError::ValidationError { .. } => {}
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_parse_host_with_protocol() {
    let url = netscene_lib::parse_host_internal("https://192.168.1.100").unwrap();
    assert_eq!(url.scheme(), "https");
    assert_eq!(url.host_str().unwrap(), "192.168.1.100");
    assert_eq!(url.path(), "/admin/api.php");
    assert_eq!(url.query(), Some("summaryRaw"));
}

#[test]
fn test_parse_host_without_protocol() {
    let url = netscene_lib::parse_host_internal("192.168.1.100").unwrap();
    assert_eq!(url.scheme(), "http");
    assert_eq!(url.host_str().unwrap(), "192.168.1.100");
    assert_eq!(url.path(), "/admin/api.php");
    assert_eq!(url.query(), Some("summaryRaw"));
}

#[test]
fn test_parse_host_with_port() {
    let url = netscene_lib::parse_host_internal("192.168.1.100:8080").unwrap();
    assert_eq!(url.scheme(), "http");
    assert_eq!(url.host_str().unwrap(), "192.168.1.100");
    assert_eq!(url.port(), Some(8080));
}

#[test]
fn test_parse_host_empty() {
    let result = netscene_lib::parse_host_internal("");
    assert!(result.is_err());
    match result.unwrap_err() {
        PiholeError::InvalidHost(_) => {}
        _ => panic!("Expected InvalidHost error"),
    }
}

#[test]
fn test_validate_pihole_response_success() {
    let stats = PiholeStats {
        domains_being_blocked: 100000,
        dns_queries_today: 5000,
        ads_blocked_today: 1500,
        ads_percentage_today: 30.0,
        status: "enabled".to_string(),
    };

    let result = netscene_lib::validate_pihole_response_internal(&stats);
    assert!(result.is_ok());
}

#[test]
fn test_validate_pihole_response_empty_status() {
    let stats = PiholeStats {
        domains_being_blocked: 100000,
        dns_queries_today: 5000,
        ads_blocked_today: 1500,
        ads_percentage_today: 30.0,
        status: "".to_string(),
    };

    let result = netscene_lib::validate_pihole_response_internal(&stats);
    assert!(result.is_err());
    match result.unwrap_err() {
        PiholeError::ValidationError { .. } => {}
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_validate_pihole_response_invalid_percentage() {
    let stats = PiholeStats {
        domains_being_blocked: 100000,
        dns_queries_today: 5000,
        ads_blocked_today: 1500,
        ads_percentage_today: 150.0,
        status: "enabled".to_string(),
    };

    let result = netscene_lib::validate_pihole_response_internal(&stats);
    assert!(result.is_err());
    match result.unwrap_err() {
        PiholeError::ValidationError { .. } => {}
        _ => panic!("Expected ValidationError"),
    }
}
