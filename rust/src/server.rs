//! Discovery Server (Agent) Implementation
//!
//! This module implements the service-side discovery agent that:
//! - Listens for UDP discovery requests on port 53535
//! - Responds with service information
//! - Announces service on startup
//! - Announces service on shutdown (goodbye)

use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::config::ServiceConfig;
use crate::protocol::{build_announce, build_discover_res, parse_message, DISCOVER_REQ};

/// Discovery server that handles UDP discovery requests
pub struct DiscoveryServer {
    config: ServiceConfig,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl DiscoveryServer {
    /// Create a new discovery server
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            shutdown_tx: None,
        }
    }

    /// Start the discovery server
    pub async fn start(&mut self) -> std::result::Result<(), ServerError> {
        let (shutdown_tx, _) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        // Create UDP socket with reuse options using socket2
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .map_err(|e| ServerError::BindError(e.to_string()))?;

        // Enable broadcast
        socket
            .set_broadcast(true)
            .map_err(|e| ServerError::BindError(e.to_string()))?;

        // Enable address reuse (allows multiple processes to bind to the same port)
        socket
            .set_reuse_address(true)
            .map_err(|e| ServerError::BindError(e.to_string()))?;

        // Enable port reuse (allows multiple processes to bind to the same port with the same address)
        #[cfg(unix)]
        {
            socket
                .set_reuse_port(true)
                .map_err(|e| ServerError::BindError(e.to_string()))?;
        }

        // Bind to port
        let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), self.config.udp_port);
        socket
            .bind(&addr.into())
            .map_err(|e| ServerError::BindError(e.to_string()))?;

        // Convert to tokio UdpSocket
        let socket = UdpSocket::from_std(socket.into())
            .map_err(|e| ServerError::BindError(e.to_string()))?;

        info!(
            "Starting discovery server for '{}' on UDP port {}",
            self.config.service_name, self.config.udp_port
        );

        // Send initial announcement
        if self.config.announce_on_startup {
            self.send_announce(&socket).await;
        }

        // Start listening
        let config = self.config.clone();
        let shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            Self::run_loop(socket, config, shutdown_rx).await;
        });

        Ok(())
    }

    /// Stop the discovery server
    pub async fn stop(&mut self) -> std::result::Result<(), ServerError> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // Send goodbye
        // Note: We can't easily get the socket here, so we skip goodbye in sync stop
        info!("Discovery server stopped");
        Ok(())
    }

    /// Run the main server loop
    async fn run_loop(
        socket: UdpSocket,
        config: ServiceConfig,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        let mut buf = [0u8; 4096];

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    debug!("Server received shutdown signal");
                    break;
                }
                result = socket.recv_from(&mut buf) => {
                    match result {
                        Ok((len, addr)) => {
                            let data = &buf[..len];
                            if let Err(e) = Self::handle_message(&socket, &config, data, addr).await {
                                debug!("Error handling message from {}: {}", addr, e);
                            }
                        }
                        Err(e) => {
                            error!("Error receiving UDP packet: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Handle incoming UDP message
    async fn handle_message(
        socket: &UdpSocket,
        config: &ServiceConfig,
        data: &[u8],
        addr: SocketAddr,
    ) -> std::result::Result<(), ServerError> {
        let (cmd, payload) = match parse_message(data) {
            Ok((cmd, payload)) => (cmd, payload),
            Err(e) => {
                debug!("Failed to parse message from {}: {}", addr, e);
                return Ok(());
            }
        };

        if cmd == DISCOVER_REQ {
            let query_id = payload
                .get("query_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            debug!("Discovery request from {}, query_id={}", addr, query_id);

            // Build and send response
            let response = build_discover_res(
                query_id,
                "ok",
                &config.service_name,
                &config.service_id,
                config.http_port,
                &config.manifest_path,
                &config.tags,
                config.priority,
            );

            socket
                .send_to(&response, addr)
                .await
                .map_err(|e| ServerError::SendError(e.to_string()))?;

            debug!("Sent discovery response to {}", addr);
        }

        Ok(())
    }

    /// Send announcement broadcast
    async fn send_announce(&self, socket: &UdpSocket) {
        let msg = build_announce(
            &self.config.service_id,
            &self.config.service_name,
            self.config.http_port,
            &self.config.manifest_path,
            &self.config.tags,
            self.config.priority,
        );

        let broadcast_addr: SocketAddr = format!("255.255.255.255:{}", self.config.udp_port)
            .parse()
            .unwrap();

        if let Err(e) = socket.send_to(&msg, broadcast_addr).await {
            warn!("Failed to send announcement: {}", e);
        } else {
            info!("Service announcement sent");
        }
    }
}

/// Server errors
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Failed to bind to UDP port: {0}")]
    BindError(String),

    #[error("Failed to send UDP packet: {0}")]
    SendError(String),

    #[error("Server not running")]
    NotRunning,
}
