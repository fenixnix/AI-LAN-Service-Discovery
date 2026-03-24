//! Configuration Models
//!
//! This module provides configuration models for service and client.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Service provider configuration
///
/// This configuration is used by the discovery agent to announce
/// service presence on the network.
///
/// # Example (Single service)
///
/// ```json
/// {
///     "service_name": "PDF Converter Pro",
///     "service_id": "pdf-converter-001",
///     "http_port": 8080,
///     "manifest_path": "/ai_manifest",
///     "tags": ["pdf", "convert", "tool"],
///     "priority": 10,
///     "announce_on_startup": true
/// }
/// ```
///
/// # Example (Multiple services)
///
/// ```json
/// {
///     "services": [
///         {
///             "service_name": "PDF Converter Pro",
///             "service_id": "pdf-converter-001",
///             "http_port": 8080,
///             "manifest_path": "/ai_manifest",
///             "tags": ["pdf", "convert", "tool"],
///             "priority": 10,
///             "announce_on_startup": true
///         },
///         {
///             "service_name": "Text Generator AI",
///             "service_id": "text-generator-001",
///             "http_port": 8081,
///             "manifest_path": "/ai_manifest",
///             "tags": ["text", "generation", "ai"],
///             "priority": 8,
///             "announce_on_startup": true
///         }
///     ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServiceConfig {
    /// Human-readable service name
    pub service_name: String,

    /// Unique service identifier
    pub service_id: String,

    /// HTTP service port
    pub http_port: u16,

    /// Manifest endpoint path
    pub manifest_path: String,

    /// Service tags for categorization
    pub tags: Vec<String>,

    /// Service priority (higher = preferred)
    pub priority: u8,

    /// UDP discovery port
    pub udp_port: u16,

    /// Announce service on startup
    pub announce_on_startup: bool,

    /// Announcement broadcast interval in seconds (0 to disable)
    pub announce_interval: u32,

    /// Raw manifest JSON data
    #[serde(default)]
    pub manifest_data: serde_json::Value,
}

impl ServiceConfig {

    /// Create ServiceConfig from raw manifest JSON
    pub fn from_manifest_json(manifest: serde_json::Value, http_port: u16) -> Self {
        let meta = manifest.get("meta");
        let endpoints = manifest.get("endpoints");

        Self {
            service_name: meta.and_then(|m| m.get("name")).and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            service_id: meta.and_then(|m| m.get("serviceId")).and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            http_port,
            manifest_path: endpoints.and_then(|e| e.get("invoke")).and_then(|v| v.as_str()).unwrap_or("/ai_manifest").to_string(),
            tags: vec![],
            priority: 1,
            udp_port: 53535,
            announce_on_startup: true,
            announce_interval: 30,
            manifest_data: manifest,
        }
    }

    /// Create ServiceConfig from .echo file
    pub fn from_echo(echo_path: &Path, manifest: Option<serde_json::Value>, http_port: u16) -> Self {
        if let Some(manifest) = manifest {
            return Self::from_manifest_json(manifest, http_port);
        }

        let dir_name = echo_path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

        Self {
            service_name: format!("Service on port {}", http_port),
            service_id: format!("{}-{}", dir_name, http_port),
            http_port,
            manifest_path: "/ai_manifest".to_string(),
            tags: vec![],
            priority: 1,
            udp_port: 53535,
            announce_on_startup: true,
            announce_interval: 30,
            manifest_data: serde_json::Value::Object(Default::default()),
        }
    }

    /// Get base URL for the service
    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.http_port)
    }

    /// Get full manifest URL
    pub fn manifest_url(&self) -> String {
        format!("{}{}", self.base_url(), self.manifest_path)
    }

    /// Create a new ServiceConfig with required fields
    pub fn new(
        service_name: impl Into<String>,
        service_id: impl Into<String>,
        http_port: u16,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            service_id: service_id.into(),
            http_port,
            manifest_path: "/ai_manifest".to_string(),
            tags: vec![],
            priority: 1,
            udp_port: 53535,
            announce_on_startup: true,
            announce_interval: 30,
            manifest_data: serde_json::Value::Object(Default::default()),
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self::new("Unknown Service", "unknown-001", 8080)
    }
}

/// Client scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfig {
    /// UDP discovery port
    #[serde(rename = "udpPort")]
    pub udp_port: u16,

    /// Scan timeout in seconds
    pub timeout: f64,

    /// Output format: json, yaml, table
    #[serde(rename = "outputFormat")]
    pub output_format: String,

    /// Output file path
    #[serde(rename = "outputFile")]
    pub output_file: Option<String>,

    /// Enable real-time listening mode
    #[serde(rename = "watchMode")]
    pub watch_mode: bool,

    /// Auto-scan interval in seconds
    #[serde(rename = "scanInterval")]
    pub scan_interval: u32,

    /// Automatically fetch service manifests
    #[serde(rename = "fetchManifest")]
    pub fetch_manifest: bool,

    /// Maximum concurrent manifest fetches
    #[serde(rename = "maxConcurrent")]
    pub max_concurrent: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            udp_port: 53535,
            timeout: 2.0,
            output_format: "json".to_string(),
            output_file: None,
            watch_mode: false,
            scan_interval: 30,
            fetch_manifest: true,
            max_concurrent: 10,
        }
    }
}

/// Echo configuration for service discovery
///
/// This configuration is loaded from .echo files.
///
/// # Example
///
/// ```json
/// {
///     "port": 8080,
///     "enable": true
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EchoConfig {
    /// HTTP service port
    pub port: u16,

    /// Whether the service is enabled
    pub enable: bool,
}

impl EchoConfig {
    /// Load configuration from .echo file
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::result::Result<Self, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        let echo_config = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(echo_config)
    }
}

impl Default for EchoConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            enable: true,
        }
    }
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}
