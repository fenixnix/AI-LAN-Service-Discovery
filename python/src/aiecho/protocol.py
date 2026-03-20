"""UDP Discovery Protocol Implementation

This module implements the AI-LAN Service Discovery protocol for UDP broadcast
communication on port 53535.

Message Format:
    COMMAND\nJSON_PAYLOAD

Commands:
    - AI_DISCOVER_REQ: Discovery request from client
    - AI_DISCOVER_RES: Discovery response from service
    - AI_SERVICE_ANNOUNCE: Service online announcement
    - AI_SERVICE_GOODBYE: Service offline announcement
"""

import json
import uuid
from dataclasses import dataclass, asdict, field
from typing import Optional, Any


# Protocol Constants
DISCOVERY_PORT = 53535
DISCOVER_REQ = "AI_DISCOVER_REQ"
DISCOVER_RES = "AI_DISCOVER_RES"
SERVICE_ANNOUNCE = "AI_SERVICE_ANNOUNCE"
SERVICE_GOODBYE = "AI_SERVICE_GOODBYE"
PROTOCOL_VERSION = "1.0"
BROADCAST_ADDR = "255.255.255.255"


def parse_message(data: bytes) -> tuple[str, dict]:
    """Parse UDP message into command and payload.
    
    Args:
        data: Raw UDP message bytes
        
    Returns:
        Tuple of (command, payload_dict)
        
    Raises:
        ValueError: If message format is invalid
    """
    try:
        text = data.decode("utf-8").strip()
        lines = text.split("\n", 1)
        if len(lines) != 2:
            raise ValueError(f"Invalid message format: expected COMMAND\\nJSON, got {repr(text)}")
        cmd = lines[0].strip()
        payload = json.loads(lines[1]) if lines[1] else {}
        return cmd, payload
    except UnicodeDecodeError as e:
        raise ValueError(f"Failed to decode message: {e}")
    except json.JSONDecodeError as e:
        raise ValueError(f"Failed to parse JSON payload: {e}")


def build_discover_req(query_id: Optional[str] = None, version: str = PROTOCOL_VERSION) -> bytes:
    """Build discovery request message.
    
    Args:
        query_id: Optional query ID (auto-generated if not provided)
        version: Protocol version
        
    Returns:
        Encoded UDP message bytes
    """
    if query_id is None:
        query_id = str(uuid.uuid4())
    payload = {"query_id": query_id, "version": version}
    return f"{DISCOVER_REQ}\n{json.dumps(payload)}".encode("utf-8")


def build_discover_res(
    query_id: str,
    status: str = "ok",
    service_name: str = "",
    service_id: str = "",
    http_port: int = 80,
    manifest_path: str = "/ai_manifest",
    tags: Optional[list[str]] = None,
    priority: int = 1,
    version: str = PROTOCOL_VERSION,
) -> bytes:
    """Build discovery response message.
    
    Args:
        query_id: Query ID from the request
        status: Response status ("ok" or "error")
        service_name: Human-readable service name
        service_id: Unique service identifier
        http_port: HTTP service port
        manifest_path: Manifest endpoint path
        tags: Service tags
        priority: Service priority
        version: Protocol version
        
    Returns:
        Encoded UDP message bytes
    """
    payload = {
        "query_id": query_id,
        "status": status,
        "service_name": service_name,
        "service_id": service_id,
        "http_port": http_port,
        "manifest_path": manifest_path,
        "tags": tags or [],
        "priority": priority,
        "version": version,
    }
    return f"{DISCOVER_RES}\n{json.dumps(payload)}".encode("utf-8")


def build_announce(
    service_id: str,
    service_name: str,
    http_port: int,
    manifest_path: str = "/ai_manifest",
    tags: Optional[list[str]] = None,
    priority: int = 1,
    version: str = PROTOCOL_VERSION,
) -> bytes:
    """Build service announcement message (online).
    
    Args:
        service_id: Unique service identifier
        service_name: Human-readable service name
        http_port: HTTP service port
        manifest_path: Manifest endpoint path
        tags: Service tags
        priority: Service priority
        version: Protocol version
        
    Returns:
        Encoded UDP message bytes
    """
    payload = {
        "event": "online",
        "service_id": service_id,
        "service_name": service_name,
        "http_port": http_port,
        "manifest_path": manifest_path,
        "tags": tags or [],
        "priority": priority,
        "timestamp": int(uuid.uuid1().time),
        "version": version,
    }
    return f"{SERVICE_ANNOUNCE}\n{json.dumps(payload)}".encode("utf-8")


def build_goodbye(
    service_id: str,
    service_name: str,
    version: str = PROTOCOL_VERSION,
) -> bytes:
    """Build service goodbye message (offline).
    
    Args:
        service_id: Unique service identifier
        service_name: Human-readable service name
        version: Protocol version
        
    Returns:
        Encoded UDP message bytes
    """
    payload = {
        "event": "offline",
        "service_id": service_id,
        "service_name": service_name,
        "timestamp": int(uuid.uuid1().time),
        "version": version,
    }
    return f"{SERVICE_GOODBYE}\n{json.dumps(payload)}".encode("utf-8")


@dataclass
class ServiceInfo:
    """Discovered service basic information from UDP response."""
    query_id: str = ""
    status: str = "ok"
    service_name: str = ""
    service_id: str = ""
    http_port: int = 80
    manifest_path: str = "/ai_manifest"
    tags: list[str] = field(default_factory=list)
    priority: int = 1
    ip: str = ""
    version: str = PROTOCOL_VERSION

    @property
    def base_url(self) -> str:
        """Get base URL for the service."""
        return f"http://{self.ip}:{self.http_port}"
    
    @property
    def manifest_url(self) -> str:
        """Get full manifest URL."""
        return f"{self.base_url}{self.manifest_path}"

    @classmethod
    def from_payload(cls, payload: dict, ip: str = "") -> "ServiceInfo":
        """Create ServiceInfo from parsed payload.
        
        Args:
            payload: Parsed JSON payload from UDP message
            ip: Source IP address
            
        Returns:
            ServiceInfo instance
        """
        return cls(
            query_id=payload.get("query_id", ""),
            status=payload.get("status", "ok"),
            service_name=payload.get("service_name", ""),
            service_id=payload.get("service_id", ""),
            http_port=payload.get("http_port", 80),
            manifest_path=payload.get("manifest_path", "/ai_manifest"),
            tags=payload.get("tags", []),
            priority=payload.get("priority", 1),
            ip=ip,
            version=payload.get("version", PROTOCOL_VERSION),
        )
    
    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)


@dataclass
class ServiceEvent:
    """Service announcement/goodbye event."""
    event: str = ""  # "online" or "offline"
    service_id: str = ""
    service_name: str = ""
    http_port: int = 80
    manifest_path: str = "/ai_manifest"
    tags: list[str] = field(default_factory=list)
    priority: int = 1
    ip: str = ""
    timestamp: int = 0
    version: str = PROTOCOL_VERSION

    @property
    def base_url(self) -> str:
        """Get base URL for the service."""
        return f"http://{self.ip}:{self.http_port}"
    
    @property
    def manifest_url(self) -> str:
        """Get full manifest URL."""
        return f"{self.base_url}{self.manifest_path}"

    @classmethod
    def from_payload(cls, payload: dict, ip: str = "") -> "ServiceEvent":
        """Create ServiceEvent from parsed payload.
        
        Args:
            payload: Parsed JSON payload from UDP message
            ip: Source IP address
            
        Returns:
            ServiceEvent instance
        """
        return cls(
            event=payload.get("event", ""),
            service_id=payload.get("service_id", ""),
            service_name=payload.get("service_name", ""),
            http_port=payload.get("http_port", 80),
            manifest_path=payload.get("manifest_path", "/ai_manifest"),
            tags=payload.get("tags", []),
            priority=payload.get("priority", 1),
            ip=ip,
            timestamp=payload.get("timestamp", 0),
            version=payload.get("version", PROTOCOL_VERSION),
        )
    
    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)
