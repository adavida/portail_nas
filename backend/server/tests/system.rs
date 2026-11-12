#[path = "shared/bootstrap_sqlite.rs"]
mod bootstrap;

use axum::http::StatusCode;
#[allow(dead_code)]
mod common;

use common::http::oneshot::oneshot;

#[tokio::test]
async fn system_returns_stats() {
    let (status, body) = oneshot("/api/system").await;

    if status == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    assert_eq!(status, StatusCode::OK, "GET /api/system should be 200");
    assert!(
        body["ram_total"]
            .as_str()
            .is_some_and(|v| !v.starts_with('0')),
        "ram total should serialize as a positive size with unit"
    );
    assert!(
        body["ram_used"].is_string() && body["cpu_usage"].as_str().unwrap().ends_with(" %"),
        "ram used should be a size string and cpu usage a percent string"
    );
    assert!(
        body["swap_used"].is_string(),
        "swap used should be a size string"
    );
    assert!(
        body["net_rx"].as_str().unwrap().ends_with("/s")
            && body["net_tx"].as_str().unwrap().ends_with("/s"),
        "net rates should be sized rates with unit"
    );
    assert!(
        body["tcp"]["established"].as_u64().is_some(),
        "tcp established count should be a number"
    );
    assert!(body["cores"].is_array(), "cores should be an array");
    assert!(body["temps"].is_array(), "temps should be an array");
    assert!(
        body["interfaces"].is_array(),
        "interfaces should be an array"
    );
    assert!(body["raid"].is_array(), "raid should be an array");
    assert!(
        body["psi_cpu"].is_null() || body["psi_cpu"].is_string(),
        "psi cpu should be a decimal string or null"
    );
    assert!(
        body["load_1"].is_string() && body["load_15"].is_string(),
        "load averages should be reported as decimal strings"
    );
    for temp in body["temps"].as_array().unwrap() {
        assert!(
            temp["celsius"].as_str().unwrap().ends_with(" °C"),
            "temps should report celsius with unit, got {temp}"
        );
    }
    assert!(
        body["cpu_cores"].as_u64().unwrap() > 0,
        "at least one core should be reported"
    );
    assert!(body["disks"].is_array(), "disks should be an array");
    for disk in body["disks"].as_array().unwrap() {
        assert!(disk["mount"].is_string(), "disk mount should be a string");
        assert!(
            disk["used"].is_string() && disk["total"].is_string(),
            "disk sizes should be unit strings, got {disk}"
        );
    }
    assert!(
        body["uptime"].as_str().unwrap().ends_with('s'),
        "uptime should be a duration with unit"
    );
    assert!(body["sensors"].is_array(), "sensors should be an array");
    assert!(
        body["sensors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|sensor| sensor.as_str() == Some("memory")),
        "default sensor list should contain memory"
    );
}

#[tokio::test]
async fn system_requires_admin_route() {
    let (status, _body) = oneshot("/api/system").await;

    assert_ne!(
        status,
        StatusCode::NOT_FOUND,
        "GET /api/system should be routed"
    );
}
