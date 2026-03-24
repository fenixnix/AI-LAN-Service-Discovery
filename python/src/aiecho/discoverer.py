"""Service Discoverer for .echo files

This module implements the service discovery logic that:
- Recursively scans for .echo files
- Validates service configurations
- Checks if ports are occupied
- Loads manifest.json files
- Creates ServiceConfig instances
"""

import json
import socket
from pathlib import Path
from typing import List, Dict, Optional, Tuple

from ai_discover.config import EchoConfig, ServiceConfig


def scan_echo_files(root_dir: Path) -> List[Path]:
    """Recursively scan for .echo files.
    
    Args:
        root_dir: Root directory to start scanning from
        
    Returns:
        List of .echo file paths
    """
    echo_files = []
    for path in root_dir.rglob("*.echo"):
        echo_files.append(path)
    return echo_files


def is_port_occupied(port: int) -> bool:
    """Check if a port is occupied.
    
    Args:
        port: Port number to check
        
    Returns:
        True if port is occupied, False otherwise
    """
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
            s.settimeout(0.1)
            s.bind(("", port))
        return False
    except OSError:
        return True


def load_manifest(echo_path: Path) -> Optional[Dict]:
    """Load manifest.json from the same directory as .echo file.
    
    Args:
        echo_path: Path to .echo file
        
    Returns:
        Manifest dictionary if found, None otherwise
    """
    manifest_path = echo_path.parent / "manifest.json"
    if not manifest_path.exists():
        return None
    
    try:
        with open(manifest_path, "r", encoding="utf-8") as f:
            return json.load(f)
    except json.JSONDecodeError:
        return None


def discover_services(root_dir: Path) -> List[Tuple[Path, ServiceConfig]]:
    """Discover services from .echo files.
    
    Args:
        root_dir: Root directory to scan
        
    Returns:
        List of (echo_path, service_config) tuples
    """
    services = []
    
    # Scan for .echo files
    echo_files = scan_echo_files(root_dir)
    
    for echo_path in echo_files:
        try:
            # Load echo config
            echo_config = EchoConfig.from_file(echo_path)
            
            # Check if enabled
            if not echo_config.enable:
                continue
            
            # Check if port is occupied
            if not is_port_occupied(echo_config.port):
                continue
            
            # Load manifest
            manifest = load_manifest(echo_path)
            if not manifest:
                continue
            
            # Create ServiceConfig from manifest
            service_config = ServiceConfig.from_manifest(manifest, echo_config.port)
            services.append((echo_path, service_config))
            
        except Exception:
            # Skip invalid .echo files
            continue
    
    return services


def get_local_ip() -> str:
    """Get local LAN IP address.
    
    Returns:
        Local LAN IP address
    """
    try:
        # Use UDP socket to get local IP
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as s:
            # Connect to a public DNS server
            s.connect(("8.8.8.8", 80))
            return s.getsockname()[0]
    except:
        # Fallback to localhost
        return "127.0.0.1"
