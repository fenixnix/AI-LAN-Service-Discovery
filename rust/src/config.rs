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
}

/// Multiple services configuration
///
/// This configuration allows defining multiple services in a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    /// List of services
    pub services: Vec<ServiceConfig>,
}

impl ServiceConfig {
    /// Load configuration from JSON file
    ///
    /// This method supports both single service configuration and multiple services configuration.
    /// - For single service: returns a vector with one element
    /// - For multiple services: returns a vector with all services
    pub fn from_file<P: AsRef<Path>>(path: P) -> std::result::Result<Vec<Self>, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        // Try to parse as multiple services first
        if let Ok(services_config) = serde_json::from_str::<ServicesConfig>(&content) {
            Ok(services_config.services)
        } else {
            // If that fails, try to parse as single service
            match serde_json::from_str::<Self>(&content) {
                Ok(service) => Ok(vec![service]),
                Err(e) => Err(ConfigError::ParseError(e.to_string())),
            }
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

/// Manifest metadata section
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestMeta {
    #[serde(rename = "serviceId")]
    pub service_id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "uptimeSeconds")]
    pub uptime_seconds: Option<u64>,
}

/// Capability input schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInputSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: serde_json::Value,
    pub required: Option<Vec<String>>,
}

/// Capability output schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityOutputSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: serde_json::Value,
}

/// Service capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: CapabilityInputSchema,
    #[serde(rename = "outputSchema")]
    pub output_schema: CapabilityOutputSchema,
}

/// Manifest endpoints section
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEndpoints {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    #[serde(rename = "healthCheck")]
    pub health_check: Option<String>,
    pub invoke: String,
}

/// Manifest authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(rename = "tokenLocation")]
    pub token_location: Option<String>,
}

/// Service Manifest - complete capability description
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub meta: ManifestMeta,
    pub capabilities: Vec<Capability>,
    pub endpoints: ManifestEndpoints,
    pub auth: ManifestAuth,
}

impl Manifest {
    /// Create Manifest from dictionary
    pub fn from_json(data: serde_json::Value) -> std::result::Result<Self, serde_json::Error> {
        serde_json::from_value(data)
    }

    /// Convert to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
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
