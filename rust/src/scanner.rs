//! Discovery Scanner (Client) Implementation
//!
//! This module implements the client-side scanner that:
//! - Sends UDP broadcast discovery requests
//! - Collects responses from all services

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
}

impl DiscoveredService {
    /// Get service IP address
    pub fn ip(&self) -> &str {
        &self.service_info.ip
    }

    /// Get service port
    pub fn port(&self) -> u16 {
        self.service_info.port
    }

    /// Get service base URL
    pub fn base_url(&self) -> String {
        self.service_info.base_url()
    }

    /// Get manifest data
    pub fn manifest(&self) -> &serde_json::Value {
        &self.service_info.manifest
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
        _fetch_manifest: Option<bool>,
    ) -> std::result::Result<Vec<DiscoveredService>, ScannerError> {
        // Broadcast and collect responses
        let services = self.broadcast_and_collect().await?;

        if services.is_empty() {
            return Ok(vec![]);
        }

        info!("Discovered {} service(s)", services.len());

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
        let mut services: Vec<DiscoveredService> = Vec::new();
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
                            if cmd == DISCOVER_RES {
                                let service_info = ServiceInfo::from_payload(
                                    &payload,
                                    addr.ip().to_string().as_str(),
                                );

                                debug!(
                                    "Discovered: {} @ {}:{}",
                                    service_info.port,
                                    addr.ip(),
                                    service_info.port
                                );

                                services.push(DiscoveredService { service_info });
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

        Ok(services)
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
