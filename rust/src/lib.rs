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
//! - HTTP Manifest API for capability description
//! - Real-time service monitoring
//! - CLI tools for easy usage
//!
//! ## Usage
//!
//! ### As server (service provider)
//! ```bash
//! aidis --config service_config.json
//! ```
//! 
//! ### As client (AI scanner)
//! ```bash
//! aidis scan --output json
//! ```

pub mod config;
pub mod protocol;
pub mod scanner;
pub mod server;

pub use config::{ClientConfig, ServiceConfig};
pub use protocol::{
    parse_message, build_discover_req, build_discover_res, build_announce, build_goodbye,
    ServiceInfo, ServiceEvent, DISCOVERY_PORT, DISCOVER_REQ, DISCOVER_RES,
    SERVICE_ANNOUNCE, SERVICE_GOODBYE, PROTOCOL_VERSION,
};
pub use scanner::{DiscoveryScanner, DiscoveredService};
pub use server::DiscoveryServer;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
