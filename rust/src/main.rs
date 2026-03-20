//! AI-LAN Service Discovery CLI
//!
//! Command-line tools for service discovery:
//! - ai-discover-agent: Run the discovery server agent
//! - ai-discover scan: Scan for services on the network

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Duration;
use tokio;
use tracing::{debug, info, error};

use aiecho::{ServiceConfig, ClientConfig, DiscoveryServer, DiscoveryScanner};

/// AI-LAN Service Discovery System
///
/// A lightweight, zero-config LAN service discovery mechanism for AI agents.
#[derive(Parser)]
#[command(name = "aiecho")]
#[command(version = "0.1.0")]
#[command(about = "AI-LAN Service Discovery System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the discovery agent (service side)
    Agent {
        /// Service configuration JSON file path
        #[arg(short, long, value_name = "FILE")]
        config: PathBuf,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
        
        /// Override UDP discovery port
        #[arg(long)]
        udp_port: Option<u16>,
    },
    
    /// Scan for services on the network
    Scan {
        /// Output format: json, yaml, table
        #[arg(short, long, default_value = "json")]
        output: String,
        
        /// Scan timeout in seconds
        #[arg(short, long, default_value = "2.0")]
        timeout: f64,
        
        /// Skip fetching service manifests
        #[arg(long)]
        no_manifest: bool,
        
        /// Output to file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output_file: Option<PathBuf>,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Listen for service changes in real-time
    Listen {
        /// Output services JSON file to watch
        #[arg(short, long, value_name = "FILE")]
        output_file: PathBuf,
        
        /// Auto-scan interval in seconds
        #[arg(short, long, default_value = "30")]
        interval: u32,
        
        /// Skip fetching service manifests
        #[arg(long)]
        no_manifest: bool,
        
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Setup logging
    let verbose = match &cli.command {
        Commands::Agent { verbose, .. } => *verbose,
        Commands::Scan { verbose, .. } => *verbose,
        Commands::Listen { verbose, .. } => *verbose,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(if verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();
    
    // Execute command
    match cli.command {
        Commands::Agent { config, verbose: _, udp_port } => {
            run_agent(config, udp_port).await;
        }
        Commands::Scan { output, timeout, no_manifest, output_file, verbose: _ } => {
            run_scan(output, timeout, no_manifest, output_file).await;
        }
        Commands::Listen { output_file, interval, no_manifest, verbose: _ } => {
            run_listen(output_file, interval, no_manifest).await;
        }
    }
}

async fn run_agent(config_path: PathBuf, udp_port: Option<u16>) {
    // Load configuration
    let service_configs = match ServiceConfig::from_file(&config_path) {
        Ok(configs) => configs,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };
    
    if service_configs.is_empty() {
        error!("No services found in configuration file");
        std::process::exit(1);
    }
    
    info!("Found {} service(s) in configuration", service_configs.len());
    
    // Override UDP port if specified and start servers
    let mut servers = Vec::new();
    for mut config in service_configs {
        if let Some(port) = udp_port {
            config.udp_port = port;
        }
        
        info!("Starting discovery agent: {}", config.service_name);
        info!("  Service ID: {}", config.service_id);
        info!("  HTTP Port: {}", config.http_port);
        info!("  UDP Port: {}", config.udp_port);
        info!("  Announce on startup: {}", config.announce_on_startup);
        
        let mut server = DiscoveryServer::new(config);
        if let Err(e) = server.start().await {
            error!("Failed to start server: {}", e);
            std::process::exit(1);
        }
        
        servers.push(server);
    }
    
    info!("All agents started. Press Ctrl+C to stop.");
    
    // Keep running
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
    
    info!("Stopping agents...");
    for mut server in servers {
        if let Err(e) = server.stop().await {
            error!("Error stopping server: {}", e);
        }
    }
    
    info!("All agents stopped.");
}

async fn run_scan(
    output: String,
    timeout: f64,
    no_manifest: bool,
    output_file: Option<PathBuf>,
) {
    info!("Scanning for services...");
    
    let config = ClientConfig {
        timeout,
        fetch_manifest: !no_manifest,
        output_format: output.clone(),
        ..Default::default()
    };
    
    let scanner = DiscoveryScanner::new(config);
    
    match scanner.scan(Some(!no_manifest)).await {
        Ok(services) => {
            if services.is_empty() {
                info!("No services found.");
                return;
            }
            
            info!("Found {} service(s)", services.len());
            
            // Format output
            match output.as_str() {
                "json" => {
                    let result: Vec<serde_json::Value> = services.iter().map(|s| {
                        serde_json::json!({
                            "serviceName": s.name(),
                            "serviceId": s.service_id(),
                            "ip": s.ip(),
                            "port": s.port(),
                            "tags": s.tags(),
                            "baseUrl": s.base_url(),
                            "manifest": if s.manifest_loaded { s.manifest.clone() } else { None },
                        })
                    }).collect();
                    
                    let json = serde_json::to_string_pretty(&result).unwrap();
                    
                    if let Some(path) = output_file {
                        std::fs::write(&path, &json).unwrap();
                        info!("Output written to {}", path.display());
                    } else {
                        println!("{}", json);
                    }
                }
                "table" => {
                    println!("\n=== Discovered Services ===\n");
                    for s in &services {
                        println!("  {} @ {}:{}", s.name(), s.ip(), s.port());
                        println!("    Tags: {:?}", s.tags());
                        println!("    Manifest: {}", if s.manifest_loaded { "Yes" } else { "No" });
                        println!();
                    }
                }
                _ => {
                    error!("Unsupported output format: {}", output);
                }
            }
        }
        Err(e) => {
            error!("Scan failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_listen(output_file: PathBuf, interval: u32, no_manifest: bool) {
    info!("Listening for service changes...");
    info!("  Output file: {}", output_file.display());
    info!("  Auto-scan interval: {}s", interval);
    
    let config = ClientConfig {
        scan_interval: interval,
        fetch_manifest: !no_manifest,
        ..Default::default()
    };
    
    let scanner = DiscoveryScanner::new(config);
    
    // Initial scan
    info!("Running initial scan...");
    match scanner.scan(Some(!no_manifest)).await {
        Ok(services) => {
            if !services.is_empty() {
                info!("Found {} service(s)", services.len());
            }
        }
        Err(e) => {
            error!("Initial scan failed: {}", e);
        }
    }
    
    // Keep running with periodic scans
    loop {
        tokio::time::sleep(Duration::from_secs(interval as u64)).await;
        
        match scanner.scan(Some(!no_manifest)).await {
            Ok(services) => {
                if !services.is_empty() {
                    let result: Vec<serde_json::Value> = services.iter().map(|s| {
                        serde_json::json!({
                            "serviceName": s.name(),
                            "serviceId": s.service_id(),
                            "ip": s.ip(),
                            "port": s.port(),
                            "tags": s.tags(),
                            "manifest": if s.manifest_loaded { s.manifest.clone() } else { None },
                        })
                    }).collect();
                    
                    let json = serde_json::to_string_pretty(&result).unwrap();
                    let _ = std::fs::write(&output_file, &json);
                }
            }
            Err(e) => {
                debug!("Periodic scan failed: {}", e);
            }
        }
    }
}
