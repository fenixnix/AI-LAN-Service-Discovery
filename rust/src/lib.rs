//! AI-LAN Service Discovery System
//!
//! A lightweight, zero-config, high-performance LAN AI microservice discovery mechanism
//! that enables AI Agents to dynamically discover and invoke various AI tool services
//! deployed within a local network.
//!
//! ## Core Features
//!
//! - UDP broadcast discovery (port 53535)
//! - Service announcement on startup
//! - .echo file based configuration
//! - Real-time service monitoring
//! - CLI tools for easy usage
//!
//! ## Usage
//!
//! ### As server (service provider)
//! ```bash
//! aiecho agent --root-path ./services
//! ```
//!
//! ### As client (AI scanner)
//! ```bash
//! aiecho scan --output json
//! ```

pub mod config;
pub mod discoverer;
pub mod protocol;
pub mod scanner;
pub mod server;

pub use config::{ClientConfig, EchoConfig, ServiceConfig};
pub use discoverer::{discover_services, get_local_ip};
pub use protocol::{
    build_announce, build_discover_req, build_discover_res, build_goodbye, parse_message,
    ServiceEvent, ServiceInfo, DISCOVERY_PORT, DISCOVER_REQ, DISCOVER_RES, PROTOCOL_VERSION,
    SERVICE_ANNOUNCE, SERVICE_GOODBYE,
};
pub use scanner::{DiscoveredService, DiscoveryScanner};
pub use server::DiscoveryServer;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
