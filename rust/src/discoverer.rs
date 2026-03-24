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
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                echo_files.extend(scan_echo_files(&path));
            } else if let Some(filename) = path.file_name() {
                let name = filename.to_string_lossy();
                eprintln!("DEBUG: Found file: {:?}", name);
                // Match .echo files (no extension, filename is ".echo")
                if name == ".echo" {
                    eprintln!("DEBUG: MATCHED .echo file: {:?}", path);
                    echo_files.push(path);
                }
            }
        }
    }
    
    eprintln!("DEBUG: Total .echo files: {}", echo_files.len());
    echo_files
}

/// Check if a port is occupied
pub fn is_port_occupied(port: u16) -> bool {
    // Check IPv4
    let ipv4_addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    if let Err(e) = TcpListener::bind(ipv4_addr) {
        if e.kind() == ErrorKind::AddrInUse {
            return true;
        }
    }
    
    // Check IPv6
    let ipv6_addr: SocketAddr = format!("[::1]:{}", port).parse().unwrap();
    if let Err(e) = TcpListener::bind(ipv6_addr) {
        if e.kind() == ErrorKind::AddrInUse {
            return true;
        }
    }
    
    false
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
    eprintln!("DEBUG: Found {} .echo files", echo_files.len());
    
    for echo_path in echo_files {
        eprintln!("DEBUG: Processing: {:?}", echo_path);
        
        match EchoConfig::from_file(&echo_path) {
            Ok(echo_config) => {
                eprintln!("DEBUG: echo_config: port={}, enable={}", echo_config.port, echo_config.enable);
                
                // Check if enabled
                if !echo_config.enable {
                    eprintln!("DEBUG: Skipping - not enabled");
                    continue;
                }
                
                // Check if port is occupied
                let port_ok = is_port_occupied(echo_config.port);
                eprintln!("DEBUG: port {} occupied: {}", echo_config.port, port_ok);
                if !port_ok {
                    eprintln!("DEBUG: Skipping - port not occupied");
                    continue;
                }
                
                // Load manifest or use default
                let manifest = load_manifest(&echo_path);
                eprintln!("DEBUG: manifest loaded: {}", manifest.is_some());
                
                let service_config = ServiceConfig::from_manifest_or_default(
                    manifest.as_ref(),
                    echo_config.port,
                    &echo_path,
                );
                eprintln!("DEBUG: service_config: name={}, id={}", service_config.service_name, service_config.service_id);
                services.push((echo_path, service_config));
            }
            Err(e) => {
                eprintln!("DEBUG: Failed to load echo config: {:?}", e);
                continue;
            }
        }
    }
    
    eprintln!("DEBUG: Total services: {}", services.len());
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
