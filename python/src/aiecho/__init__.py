"""AI-LAN Service Discovery System

A lightweight, zero-config, high-performance LAN AI microservice discovery mechanism
that enables AI Agents to dynamically discover and invoke various AI tool services
deployed within a local network.

Core Features:
- UDP broadcast discovery (port 53535)
- Service announcement on startup
- HTTP Manifest API for capability description
- Real-time service monitoring
- CLI tools for easy usage

Usage:
    # As server (service provider)
    ai-discover-agent --config service_config.json
    
    # As client (AI scanner)
    ai-scan --output json
    ai-scan --watch --output-file services.json
"""

__version__ = "0.1.0"
__author__ = "AI Server Discover Team"

from ai_discover.protocol import (
    DISCOVERY_PORT,
    DISCOVER_REQ,
    DISCOVER_RES,
    SERVICE_ANNOUNCE,
    SERVICE_GOODBYE,
    PROTOCOL_VERSION,
    parse_message,
    build_discover_req,
    build_discover_res,
    build_announce,
    build_goodbye,
    ServiceInfo,
    ServiceEvent,
)
from ai_discover.config import ServiceConfig, ClientConfig
from ai_discover.server import DiscoveryServer
from ai_discover.scanner import DiscoveryScanner, DiscoveredService
from ai_discover.listener import DiscoveryListener, ServiceState

__all__ = [
    # Version
    "__version__",
    # Protocol
    "DISCOVERY_PORT",
    "DISCOVER_REQ",
    "DISCOVER_RES",
    "SERVICE_ANNOUNCE",
    "SERVICE_GOODBYE",
    "PROTOCOL_VERSION",
    "parse_message",
    "build_discover_req",
    "build_discover_res",
    "build_announce",
    "build_goodbye",
    "ServiceInfo",
    "ServiceEvent",
    # Config
    "ServiceConfig",
    "ClientConfig",
    # Server
    "DiscoveryServer",
    # Scanner
    "DiscoveryScanner",
    "DiscoveredService",
    # Listener
    "DiscoveryListener",
    "ServiceState",
]
