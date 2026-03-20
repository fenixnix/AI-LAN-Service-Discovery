//! Discovery Scanner (Client) Implementation
//!
//! This module implements the client-side scanner that:
//! - Sends UDP broadcast discovery requests
//! - Collects responses from all services
//! - Fetches service manifests via HTTP
//! - Returns standardized service information

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::{debug, error, info};

use crate::config::ClientConfig;
use crate::protocol::{build_discover_req, parse_message, ServiceInfo, DISCOVER_RES};

/// Complete discovered service with manifest
#[derive(Debug, Clone)]
pub struct DiscoveredService {
    pub service_info: ServiceInfo,
    pub manifest: Option<serde_json::Value>,
    pub manifest_loaded: bool,
    pub manifest_error: Option<String>,
}

impl DiscoveredService {
    /// Get service IP address
    pub fn ip(&self) -> &str {
        &self.service_info.ip
    }

    /// Get service HTTP port
    pub fn port(&self) -> u16 {
        self.service_info.http_port
    }

    /// Get service name
    pub fn name(&self) -> &str {
        &self.service_info.service_name
    }

    /// Get service ID
    pub fn service_id(&self) -> &str {
        &self.service_info.service_id
    }

    /// Get service tags
    pub fn tags(&self) -> &[String] {
        &self.service_info.tags
    }

    /// Get service base URL
    pub fn base_url(&self) -> String {
        self.service_info.base_url()
    }

    /// Get service manifest URL
    pub fn manifest_url(&self) -> String {
        self.service_info.manifest_url()
    }
}

/// Discovery scanner that broadcasts queries and collects responses
pub struct DiscoveryScanner {
    config: ClientConfig,
}

impl DiscoveryScanner {
    /// Create a new discovery scanner
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Create with default config
    pub fn default_config() -> Self {
        Self::new(ClientConfig::default())
    }

    /// Scan for services on the network
    pub async fn scan(
        &self,
        fetch_manifest: Option<bool>,
    ) -> std::result::Result<Vec<DiscoveredService>, ScannerError> {
        let should_fetch = fetch_manifest.unwrap_or(self.config.fetch_manifest);

        // Phase 1: Broadcast and collect
        let mut services = self.broadcast_and_collect().await?;

        if services.is_empty() {
            return Ok(vec![]);
        }

        info!("Discovered {} service(s)", services.len());

        // Phase 2: Fetch manifests (concurrent)
        if should_fetch {
            self.fetch_manifests(&mut services).await;
        }

        Ok(services)
    }

    /// Send broadcast and collect all responses
    async fn broadcast_and_collect(
        &self,
    ) -> std::result::Result<Vec<DiscoveredService>, ScannerError> {
        // Create UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| ScannerError::BindError(e.to_string()))?;

        // Enable broadcast
        let socket = socket
            .into_std()
            .map_err(|e| ScannerError::BindError(e.to_string()))?;
        socket
            .set_broadcast(true)
            .map_err(|e| ScannerError::BindError(e.to_string()))?;
        let socket =
            UdpSocket::from_std(socket).map_err(|e| ScannerError::BindError(e.to_string()))?;

        // Send discovery request
        let request_msg = build_discover_req(None);
        let broadcast_addr: SocketAddr = format!("255.255.255.255:{}", self.config.udp_port)
            .parse()
            .map_err(|e: std::net::AddrParseError| ScannerError::AddressError(e.to_string()))?;

        socket
            .send_to(&request_msg, broadcast_addr)
            .await
            .map_err(|e: std::io::Error| ScannerError::SendError(e.to_string()))?;

        debug!("Sent discovery request to broadcast address");

        // Collect responses with timeout
        let mut services: HashMap<String, DiscoveredService> = HashMap::new();
        let timeout_duration = Duration::from_secs_f64(self.config.timeout);
        let start = std::time::Instant::now();

        let mut buf = [0u8; 4096];

        while start.elapsed() < timeout_duration {
            let remaining = timeout_duration - start.elapsed();

            match timeout(remaining, socket.recv_from(&mut buf)).await {
                Ok(Ok((len, addr))) => {
                    let data = &buf[..len];

                    match parse_message(data) {
                        Ok((cmd, payload)) => {
                            if cmd == DISCOVER_RES
                                && payload.get("status").and_then(|v| v.as_str()) == Some("ok")
                            {
                                let service_info = ServiceInfo::from_payload(
                                    &payload,
                                    addr.ip().to_string().as_str(),
                                );
                                let service_id = service_info.service_id.clone();

                                if !service_id.is_empty() && !services.contains_key(&service_id) {
                                    services.insert(
                                        service_id.clone(),
                                        DiscoveredService {
                                            service_info,
                                            manifest: None,
                                            manifest_loaded: false,
                                            manifest_error: None,
                                        },
                                    );

                                    debug!(
                                        "Discovered: {} @ {}:{}",
                                        services.get(&service_id).unwrap().name(),
                                        addr.ip(),
                                        services.get(&service_id).unwrap().port()
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            debug!("Failed to parse response from {}: {}", addr, e);
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("Error receiving UDP packet: {}", e);
                }
                Err(_) => {
                    // Timeout, check if we should continue
                    if !services.is_empty() {
                        break;
                    }
                }
            }
        }

        Ok(services.into_values().collect())
    }

    /// Fetch manifests for all discovered services concurrently
    async fn fetch_manifests(&self, services: &mut Vec<DiscoveredService>) {
        debug!("Fetching manifests for {} service(s)", services.len());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();

        let mut handles = Vec::new();

        for service in services.iter() {
            let client = client.clone();
            let url = service.manifest_url();

            handles.push(tokio::spawn(async move {
                match client.get(&url).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.json::<serde_json::Value>().await {
                                Ok(manifest) => {
                                    return Some((url, Ok(manifest)));
                                }
                                Err(e) => {
                                    return Some((url, Err(e.to_string())));
                                }
                            }
                        } else {
                            return Some((url, Err(format!("HTTP {}", response.status()))));
                        }
                    }
                    Err(e) => {
                        return Some((url, Err(e.to_string())));
                    }
                }
            }));
        }

        for (service, handle) in services.iter_mut().zip(handles.into_iter()) {
            if let Ok(Some((_url, result))) = handle.await {
                match result {
                    Ok(manifest) => {
                        service.manifest = Some(manifest);
                        service.manifest_loaded = true;
                        debug!("Loaded manifest for {}", service.name());
                    }
                    Err(e) => {
                        service.manifest_error = Some(e.clone());
                        debug!("Failed to fetch manifest for {}: {}", service.name(), e);
                    }
                }
            }
        }
    }
}

/// Scanner errors
#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    #[error("Failed to bind to UDP port: {0}")]
    BindError(String),

    #[error("Failed to send UDP packet: {0}")]
    SendError(String),

    #[error("Invalid address: {0}")]
    AddressError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),
}
