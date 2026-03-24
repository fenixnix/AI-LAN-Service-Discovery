//! Service Discoverer for .echo files
//! 
//! This module implements the service discovery logic that:
//! - Recursively scans for .echo files
//! - Validates service configurations
//! - Checks if ports are occupied
//! - Loads manifest.json files
//! - Creates ServiceConfig instances

use std::net::{TcpListener, SocketAddr};
use std::path::{Path, PathBuf};
use std::io::ErrorKind;
use std::fs::File;
use std::io::Read;

use serde_json;

use crate::config::{EchoConfig, ServiceConfig, Manifest, ConfigError};

/// Recursively scan for .echo files
pub fn scan_echo_files(root_dir: &Path) -> Vec<PathBuf> {
    let mut echo_files = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(root_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    echo_files.extend(scan_echo_files(&path));
                } else if path.extension().map_or(false, |ext| ext == "echo") {
                    echo_files.push(path);
                }
            }
        }
    }
    
    echo_files
}

/// Check if a port is occupied
pub fn is_port_occupied(port: u16) -> bool {
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    match TcpListener::bind(addr) {
        Ok(_) => false,
        Err(e) if e.kind() == ErrorKind::AddrInUse => true,
        _ => false,
    }
}

/// Load manifest.json from the same directory as .echo file
pub fn load_manifest(echo_path: &Path) -> Option<Manifest> {
    let manifest_path = echo_path.parent()?.join("manifest.json");
    if !manifest_path.exists() {
        return None;
    }
    
    let mut file = File::open(&manifest_path).ok()?;
    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        return None;
    }
    
    serde_json::from_str(&content).ok()
}

/// Discover services from .echo files
pub fn discover_services(root_dir: &Path) -> Vec<(PathBuf, ServiceConfig)> {
    let mut services = Vec::new();
    
    let echo_files = scan_echo_files(root_dir);
    
    for echo_path in echo_files {
        match EchoConfig::from_file(&echo_path) {
            Ok(echo_config) => {
                // Check if enabled
                if !echo_config.enable {
                    continue;
                }
                
                // Check if port is occupied
                if !is_port_occupied(echo_config.port) {
                    continue;
                }
                
                // Load manifest
                if let Some(manifest) = load_manifest(&echo_path) {
                    // Create ServiceConfig from manifest
                    let service_config = ServiceConfig::from_manifest(&manifest, echo_config.port);
                    services.push((echo_path, service_config));
                }
            }
            Err(_) => {
                // Skip invalid .echo files
                continue;
            }
        }
    }
    
    services
}

/// Get local LAN IP address
pub fn get_local_ip() -> String {
    // Use UDP socket to get local IP
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    
    // Fallback to localhost
    "127.0.0.1".to_string()
}
