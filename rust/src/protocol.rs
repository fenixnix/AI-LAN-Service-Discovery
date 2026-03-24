//! UDP Discovery Protocol Implementation
//!
//! This module implements the AI-LAN Service Discovery protocol for UDP broadcast
//! communication on port 53535.
//!
//! ## Message Format
//!
//! COMMAND\nJSON_PAYLOAD
//!
//! ## Commands
//!
//! - AI_DISCOVER_REQ: Discovery request from client
//! - AI_DISCOVER_RES: Discovery response from service
//! - AI_SERVICE_ANNOUNCE: Service online announcement
//! - AI_SERVICE_GOODBYE: Service offline announcement

use serde::{Deserialize, Serialize};

/// Protocol constants
pub const DISCOVERY_PORT: u16 = 53535;
pub const DISCOVER_REQ: &str = "AI_DISCOVER_REQ";
pub const DISCOVER_RES: &str = "AI_DISCOVER_RES";
pub const SERVICE_ANNOUNCE: &str = "AI_SERVICE_ANNOUNCE";
pub const SERVICE_GOODBYE: &str = "AI_SERVICE_GOODBYE";
pub const PROTOCOL_VERSION: &str = "1.0";
pub const BROADCAST_ADDR: &str = "255.255.255.255";

/// Parse UDP message into command and payload
pub fn parse_message(
    data: &[u8],
) -> std::result::Result<(String, serde_json::Value), ProtocolError> {
    let text = String::from_utf8(data.to_vec())
        .map_err(|e| ProtocolError::InvalidEncoding(e.to_string()))?;

    let text = text.trim();
    let lines: Vec<&str> = text.splitn(2, '\n').collect();

    if lines.len() != 2 {
        return Err(ProtocolError::InvalidFormat(format!(
            "Expected COMMAND\\nJSON, got: {}",
            &text[..text.len().min(100)]
        )));
    }

    let cmd = lines[0].trim().to_string();
    let payload: serde_json::Value =
        serde_json::from_str(lines[1]).map_err(|e| ProtocolError::InvalidJson(e.to_string()))?;

    Ok((cmd, payload))
}

pub fn build_discover_req(query_id: Option<&str>) -> Vec<u8> {
    let query_id = query_id
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let payload = serde_json::json!({
        "query_id": query_id,
        "version": PROTOCOL_VERSION,
    });
    format!(
        "{}\n{}",
        DISCOVER_REQ,
        serde_json::to_string(&payload).unwrap()
    )
    .into_bytes()
}

/// Discovery response parameters
#[derive(Debug)]
pub struct DiscoverResParams<'a> {
    pub query_id: &'a str,
    pub http_port: u16,
    pub manifest_data: &'a serde_json::Value,
}

/// Build discovery response message
pub fn build_discover_res(params: DiscoverResParams) -> Vec<u8> {
    let payload = serde_json::json!({
        "query_id": params.query_id,
        "port": params.http_port,
        "manifest": params.manifest_data,
    });
    format!(
        "{}\n{}",
        DISCOVER_RES,
        serde_json::to_string(&payload).unwrap()
    )
    .into_bytes()
}

/// Build service announcement message (online)
pub fn build_announce(
    http_port: u16,
    manifest_data: &serde_json::Value,
) -> Vec<u8> {
    let payload = serde_json::json!({
        "event": "online",
        "port": http_port,
        "manifest": manifest_data,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    });
    format!(
        "{}\n{}",
        SERVICE_ANNOUNCE,
        serde_json::to_string(&payload).unwrap()
    )
    .into_bytes()
}

/// Build service goodbye message (offline)
pub fn build_goodbye(service_id: &str, service_name: &str) -> Vec<u8> {
    let payload = serde_json::json!({
        "event": "offline",
        "service_id": service_id,
        "service_name": service_name,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "version": PROTOCOL_VERSION,
    });
    format!(
        "{}\n{}",
        SERVICE_GOODBYE,
        serde_json::to_string(&payload).unwrap()
    )
    .into_bytes()
}

/// Discovered service basic information from UDP response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    pub query_id: String,
    pub port: u16,
    pub ip: String,
    pub manifest: serde_json::Value,
}

impl ServiceInfo {
    /// Get base URL for the service
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.ip, self.http_port)
    }

    /// Get full manifest URL
    pub fn manifest_url(&self) -> String {
        format!("{}{}", self.base_url(), self.manifest_path)
    }

    /// Create ServiceInfo from parsed payload
    pub fn from_payload(payload: &serde_json::Value, ip: &str) -> Self {
        Self {
            query_id: payload
                .get("query_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            status: payload
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("ok")
                .to_string(),
            service_name: payload
                .get("service_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            service_id: payload
                .get("service_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            http_port: payload
                .get("http_port")
                .and_then(|v| v.as_u64())
                .unwrap_or(80) as u16,
            manifest_path: payload
                .get("manifest_path")
                .and_then(|v| v.as_str())
                .unwrap_or("/ai_manifest")
                .to_string(),
            tags: payload
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            priority: payload
                .get("priority")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as u8,
            ip: ip.to_string(),
            version: payload
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or(PROTOCOL_VERSION)
                .to_string(),
        }
    }
}

/// Service announcement/goodbye event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceEvent {
    pub event: String,
    pub service_id: String,
    pub service_name: String,
    pub http_port: u16,
    pub manifest_path: String,
    pub tags: Vec<String>,
    pub priority: u8,
    pub ip: String,
    pub timestamp: u64,
    pub version: String,
}

impl ServiceEvent {
    /// Get base URL for the service
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.ip, self.http_port)
    }

    /// Get full manifest URL
    pub fn manifest_url(&self) -> String {
        format!("{}{}", self.base_url(), self.manifest_path)
    }

    /// Create ServiceEvent from parsed payload
    pub fn from_payload(payload: &serde_json::Value, ip: &str) -> Self {
        Self {
            event: payload
                .get("event")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            service_id: payload
                .get("service_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            service_name: payload
                .get("service_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            http_port: payload
                .get("http_port")
                .and_then(|v| v.as_u64())
                .unwrap_or(80) as u16,
            manifest_path: payload
                .get("manifest_path")
                .and_then(|v| v.as_str())
                .unwrap_or("/ai_manifest")
                .to_string(),
            tags: payload
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            priority: payload
                .get("priority")
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as u8,
            ip: ip.to_string(),
            timestamp: payload
                .get("timestamp")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            version: payload
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or(PROTOCOL_VERSION)
                .to_string(),
        }
    }
}

/// Protocol errors
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
}
