# AIEcho Python Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a complete Python implementation of the AIEcho system with server agent, client scanner, and real-time listener.

**Architecture:** Two-stage discovery protocol - UDP broadcast (53535) for service location + HTTP GET for manifest introspection. Services announce themselves on startup.

**Tech Stack:** Python 3.8+, asyncio, click, pydantic

---

## Project Structure

```
python/
├── pyproject.toml
├── src/
│   └── aiecho/
│       ├── __init__.py
│       ├── protocol.py      # UDP protocol constants and message parsing
│       ├── config.py        # Configuration models
│       ├── server.py        # DiscoveryServer (agent)
│       ├── scanner.py       # DiscoveryScanner (client)
│       ├── listener.py      # Real-time listener
│       └── cli.py           # CLI entry points
├── tests/
│   ├── test_protocol.py
│   ├── test_server.py
│   ├── test_scanner.py
│   └── test_listener.py
└── examples/
    └── service_config.json
```

---

## Task 1: Project Setup

**Files:**
- Create: `python/pyproject.toml`
- Create: `python/src/ai_discover/__init__.py`

**Step 1: Create pyproject.toml**

```toml
[project]
name = "aiecho"
version = "0.1.0"
description = "AIEcho System"
readme = "README.md"
requires-python = ">=3.8"
dependencies = [
    "click>=8.0.0",
    "pydantic>=2.0.0",
    "rich>=13.0.0",
    "zeroconf>=0.40.0",
]

[project.scripts]
aiecho-agent = "aiecho.cli:agent"
aiecho-scan = "aiecho.cli:scan"

[build-system]
requires = ["setuptools>=61.0"]
build-backend = "setuptools.build_meta"

[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
```

**Step 2: Create __init__.py**

```python
"""AIEcho System"""

__version__ = "0.1.0"

from aiecho.protocol import DISCOVERY_PORT, DISCOVER_REQ, DISCOVER_RES, SERVICE_ANNOUNCE, SERVICE_GOODBYE
from aiecho.config import ServiceConfig, ClientConfig
from aiecho.server import DiscoveryServer
from aiecho.scanner import DiscoveryScanner
from aiecho.listener import DiscoveryListener

__all__ = [
    "DISCOVERY_PORT",
    "DISCOVER_REQ", 
    "DISCOVER_RES",
    "SERVICE_ANNOUNCE",
    "SERVICE_GOODBYE",
    "ServiceConfig",
    "ClientConfig", 
    "DiscoveryServer",
    "DiscoveryScanner",
    "DiscoveryListener",
]
```

**Step 3: Run install**

```bash
cd python && pip install -e .
```

**Step 4: Commit**

```bash
git add python/pyproject.toml python/src/ai_discover/__init__.py
git commit -m "feat(python): initial project setup"
```

---

## Task 2: Protocol Implementation

**Files:**
- Create: `python/src/ai_discover/protocol.py`

**Step 1: Write the failing test**

```python
# tests/test_protocol.py
import pytest
from aiecho.protocol import (
    parse_message,
    build_discover_req,
    build_discover_res,
    build_announce,
    build_goodbye,
    DISCOVERY_PORT,
)
```
def test_parse_discover_req():
    raw = "AI_DISCOVER_REQ\n{\"query_id\": \"test-123\", \"version\": \"1.0\"}"
    cmd, payload = parse_message(raw.encode())
    assert cmd == "AI_DISCOVER_REQ"
    assert payload["query_id"] == "test-123"

def test_build_discover_res():
    payload = {
        "query_id": "test-123",
        "status": "ok",
        "service_name": "Test Service",
        "service_id": "test-001",
        "http_port": 8080,
        "manifest_path": "/ai_manifest",
        "tags": ["test"],
        "priority": 1,
    }
    msg = build_discover_res(payload)
    assert msg.startswith("AI_DISCOVER_RES\n")
    assert "test-123" in msg
```

**Step 2: Run test to verify it fails**

```bash
cd python && pytest tests/test_protocol.py::test_parse_discover_req -v
Expected: FAIL - protocol module not found

**Step 3: Write protocol.py implementation**

```python
"""UDP Discovery Protocol Implementation"""

import json
import uuid
from dataclasses import dataclass, asdict
from typing import Optional

# Protocol Constants
DISCOVERY_PORT = 53535
DISCOVER_REQ = "AI_DISCOVER_REQ"
DISCOVER_RES = "AI_DISCOVER_RES"
SERVICE_ANNOUNCE = "AI_SERVICE_ANNOUNCE"
SERVICE_GOODBYE = "AI_SERVICE_GOODBYE"
PROTOCOL_VERSION = "1.0"

BROADCAST_ADDR = "255.255.255.255"


def parse_message(data: bytes) -> tuple[str, dict]:
    """Parse UDP message into command and payload."""
    try:
        text = data.decode("utf-8").strip()
        lines = text.split("\n", 1)
        if len(lines) != 2:
            raise ValueError("Invalid message format")
        cmd = lines[0].strip()
        payload = json.loads(lines[1]) if lines[1] else {}
        return cmd, payload
    except Exception as e:
        raise ValueError(f"Failed to parse message: {e}")


def build_discover_req(query_id: Optional[str] = None) -> bytes:
    """Build discovery request message."""
    if query_id is None:
        query_id = str(uuid.uuid4())
    payload = {"query_id": query_id, "version": PROTOCOL_VERSION}
    return f"{DISCOVER_REQ}\n{json.dumps(payload)}".encode("utf-8")


def build_discover_res(payload: dict) -> bytes:
    """Build discovery response message."""
    msg = {**payload, "version": PROTOCOL_VERSION}
    return f"{DISCOVER_RES}\n{json.dumps(msg)}".encode("utf-8")


def build_announce(config: "ServiceConfig") -> bytes:
    """Build service announcement message."""
    payload = {
        "event": "online",
        "service_id": config.service_id,
        "service_name": config.service_name,
        "http_port": config.http_port,
        "manifest_path": config.manifest_path,
        "tags": config.tags or [],
        "priority": config.priority or 1,
    }
    return f"{SERVICE_ANNOUNCE}\n{json.dumps(payload)}".encode("utf-8")


def build_goodbye(config: "ServiceConfig") -> bytes:
    """Build service goodbye message."""
    payload = {
        "event": "offline",
        "service_id": config.service_id,
        "service_name": config.service_name,
    }
    return f"{SERVICE_GOODBYE}\n{json.dumps(payload)}".encode("utf-8")


@dataclass
class ServiceInfo:
    """Discovered service information."""
    query_id: str
    status: str
    service_name: str
    service_id: str
    http_port: int
    manifest_path: str
    tags: list[str]
    priority: int
    ip: str = ""
    version: str = PROTOCOL_VERSION

    @property
    def base_url(self) -> str:
        return f"http://{self.ip}:{self.http_port}"
    
    @property
    def manifest_url(self) -> str:
        return f"{self.base_url}{self.manifest_path}"

    @classmethod
    def from_payload(cls, payload: dict, ip: str = "") -> "ServiceInfo":
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


@dataclass
class ServiceEvent:
    """Service announcement/goodbye event."""
    event: str  # "online" or "offline"
    service_id: str
    service_name: str
    http_port: int
    manifest_path: str
    tags: list[str]
    priority: int
    ip: str = ""
    timestamp: int = 0

    @classmethod
    def from_payload(cls, payload: dict, ip: str = "") -> "ServiceEvent":
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
        )
```

**Step 4: Run test to verify it passes**

```bash
cd python && pytest tests/test_protocol.py -v
Expected: PASS

**Step 5: Commit**

```bash
git add python/src/ai_discover/protocol.py tests/test_protocol.py
git commit -m "feat(python): implement UDP discovery protocol"
```

---

## Task 3: Configuration Models

**Files:**
- Create: `python/src/ai_discover/config.py`

**Step 1: Write the failing test**

```python
# tests/test_config.py
import pytest
from pydantic import ValidationError
from aiecho.config import ServiceConfig, ClientConfig
```
def test_service_config_valid():
    config = ServiceConfig(
        service_name="Test Service",
        service_id="test-001",
        http_port=8080,
    )
    assert config.service_name == "Test Service"
    assert config.manifest_path == "/ai_manifest"

def test_service_config_missing_required():
    with pytest.raises(ValidationError):
        ServiceConfig(service_name="Test")
```

**Step 2: Run test to verify it fails**

```bash
cd python && pytest tests/test_config.py::test_service_config_valid -v
Expected: FAIL - config module not found

**Step 3: Write config.py implementation**

```python
"""Configuration Models"""

from pydantic import BaseModel, Field
from typing import Optional


class ServiceConfig(BaseModel):
    """Service provider configuration."""
    
    service_name: str = Field(..., description="Human-readable service name")
    service_id: str = Field(..., description="Unique service identifier")
    http_port: int = Field(..., ge=1, le=65535, description="HTTP service port")
    manifest_path: str = Field(
        default="/ai_manifest",
        description="Manifest endpoint path"
    )
    tags: Optional[list[str]] = Field(
        default=None,
        description="Service tags for categorization"
    )
    priority: int = Field(
        default=1,
        ge=1,
        le=100,
        description="Service priority (higher = preferred)"
    )
    udp_port: int = Field(
        default=53535,
        description="UDP discovery port"
    )
    announce_on_startup: bool = Field(
        default=True,
        description="Announce service on startup"
    )

    @property
    def base_url(self) -> str:
        return f"http://localhost:{self.http_port}"


class ClientConfig(BaseModel):
    """Client scanner configuration."""
    
    udp_port: int = Field(
        default=53535,
        description="UDP discovery port"
    )
    timeout: float = Field(
        default=2.0,
        ge=0.1,
        le=30.0,
        description="Scan timeout in seconds"
    )
    output_format: str = Field(
        default="json",
        description="Output format: json, yaml, table"
    )
    output_file: Optional[str] = Field(
        default=None,
        description="Output file path"
    )
    watch_mode: bool = Field(
        default=False,
        description="Enable real-time listening mode"
    )
    scan_interval: int = Field(
        default=30,
        ge=5,
        le=300,
        description="Auto-scan interval in seconds"
    )

    class Config:
        use_enum_values = True
```

**Step 4: Run test to verify it passes**

```bash
cd python && pytest tests/test_config.py -v
Expected: PASS

**Step 5: Commit**

```bash
git add python/src/ai_discover/config.py tests/test_config.py
git commit -m "feat(python): add configuration models"
```

---

## Task 4: Server Agent Implementation

**Files:**
- Create: `python/src/ai_discover/server.py`

**Step 1: Write the failing test**

```python
# tests/test_server.py
import pytest
import asyncio
from aiecho.server import DiscoveryServer
from aiecho.config import ServiceConfig
```
@pytest.mark.asyncio
async def test_server_start_stop():
    config = ServiceConfig(
        service_name="Test Service",
        service_id="test-001",
        http_port=8080,
    )
    server = DiscoveryServer(config)
    await server.start()
    assert server.is_running()
    await server.stop()
    assert not server.is_running()
```

**Step 2: Run test to verify it fails**

```bash
cd python && pytest tests/test_server.py::test_server_start_stop -v
Expected: FAIL - server module not found

**Step 3: Write server.py implementation**

```python
"""Discovery Server (Agent) Implementation"""

import asyncio
import json
import logging
import socket
import threading
from typing import Optional
import zeroconf

from aiecho.protocol import (
    DISCOVERY_PORT,
    DISCOVER_REQ,
    DISCOVER_RES,
    SERVICE_ANNOUNCE,
    SERVICE_GOODBYE,
    parse_message,
    build_discover_res,
    build_announce,
    build_goodbye,
)
from aiecho.config import ServiceConfig

logger = logging.getLogger(__name__)


class DiscoveryServer:
    """UDP Discovery Server that responds to broadcast queries."""
    
    def __init__(self, config: ServiceConfig):
        self.config = config
        self._running = False
        self._thread: Optional[threading.Thread] = None
        self._socket: Optional[socket.socket] = None
        self._zeroconf: Optional[zeroconf.Zeroconf] = None
        self._service: Optional[zeroconf.ServiceInfo] = None

    def is_running(self) -> bool:
        return self._running

    def start(self) -> None:
        """Start the discovery server in a background thread."""
        if self._running:
            logger.warning("Server already running")
            return
        
        self._running = True
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._thread.start()
        
        # Announce service on startup if enabled
        if self.config.announce_on_startup:
            self._announce()
        
        logger.info(f"Discovery server started for {self.config.service_name}")

    def stop(self) -> None:
        """Stop the discovery server."""
        if not self._running:
            return
        
        # Send goodbye message
        self._goodbye()
        
        self._running = False
        if self._thread:
            self._thread.join(timeout=2.0)
        
        if self._socket:
            self._socket.close()
            self._socket = None
        
        # Unregister mDNS service
        if self._zeroconf and self._service:
            try:
                self._zeroconf.unregister_service(self._service)
                self._zeroconf.close()
            except Exception as e:
                logger.debug(f"mDNS cleanup error: {e}")
        
        logger.info(f"Discovery server stopped for {self.config.service_name}")

    def _run(self) -> None:
        """Main server loop."""
        try:
            self._socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            self._socket.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
            self._socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            self._socket.bind(("", self.config.udp_port))
            self._socket.settimeout(1.0)
            
            # Also register mDNS service
            self._register_mdns()
            
            logger.info(f"Listening on UDP port {self.config.udp_port}")
            
            while self._running:
                try:
                    data, addr = self._socket.recvfrom(4096)
                    self._handle_message(data, addr)
                except socket.timeout:
                    continue
                except Exception as e:
                    if self._running:
                        logger.error(f"Error handling message: {e}")
        except Exception as e:
            logger.error(f"Server error: {e}")
        finally:
            self._running = False

    def _handle_message(self, data: bytes, addr: tuple) -> None:
        """Handle incoming UDP message."""
        try:
            cmd, payload = parse_message(data)
            
            if cmd == DISCOVER_REQ:
                query_id = payload.get("query_id", "")
                response = {
                    "query_id": query_id,
                    "status": "ok",
                    "service_name": self.config.service_name,
                    "service_id": self.config.service_id,
                    "http_port": self.config.http_port,
                    "manifest_path": self.config.manifest_path,
                    "tags": self.config.tags or [],
                    "priority": self.config.priority or 1,
                }
                msg = build_discover_res(response)
                self._socket.sendto(msg, addr)
                logger.debug(f"Sent discovery response to {addr}")
                
        except Exception as e:
            logger.error(f"Failed to handle message: {e}")

    def _announce(self) -> None:
        """Broadcast service announcement."""
        if not self._socket:
            return
        
        try:
            msg = build_announce(self.config)
            self._socket.sendto(msg, ("255.255.255.255", self.config.udp_port))
            logger.info("Service announcement sent")
        except Exception as e:
            logger.error(f"Failed to send announcement: {e}")

    def _goodbye(self) -> None:
        """Broadcast service goodbye."""
        if not self._socket:
            return
        
        try:
            msg = build_goodbye(self.config)
            self._socket.sendto(msg, ("255.255.255.255", self.config.udp_port))
            logger.info("Service goodbye sent")
        except Exception as e:
            logger.debug(f"Failed to send goodbye: {e}")

    def _register_mdns(self) -> None:
        """Register mDNS service for additional discovery."""
        try:
            self._zeroconf = zeroconf.Zeroconf()
            self._service = zeroconf.ServiceInfo(
                "_ai-service._tcp.local.",
                f"{self.config.service_id}._ai-service._tcp.local.",
                addresses=socket.gethostbyname_ex(socket.gethostname())[2],
                port=self.config.http_port,
                properties={
                    "manifest_path": self.config.manifest_path.encode(),
                    "service_id": self.config.service_id.encode(),
                },
            )
            self._zeroconf.register_service(self._service)
            logger.info("mDNS service registered")
        except Exception as e:
            logger.debug(f"mDNS registration failed: {e}")
```

**Step 4: Run test to verify it passes**

```bash
cd python && pytest tests/test_server.py -v
Expected: PASS

**Step 5: Commit**

```bash
git add python/src/ai_discover/server.py tests/test_server.py
git commit -m "feat(python): implement discovery server agent"
```

---

## Task 5: Client Scanner Implementation

**Files:**
- Create: `python/src/ai_discover/scanner.py`

**Step 1: Write the failing test**

```python
# tests/test_scanner.py
import pytest
import asyncio
from aiecho.scanner import DiscoveryScanner
```
@pytest.mark.asyncio
async def test_scanner_scan():
    scanner = DiscoveryScanner(timeout=1.0)
    services = await scanner.scan()
    assert isinstance(services, list)
```

**Step 2: Run test to verify it fails**

```bash
cd python && pytest tests/test_scanner.py::test_scanner_scan -v
Expected: FAIL - scanner module not found

**Step 3: Write scanner.py implementation**

```python
"""Discovery Scanner (Client) Implementation"""

import asyncio
import json
import socket
import logging
from typing import Optional
from dataclasses import dataclass, field

from aiecho.protocol import (
    DISCOVERY_PORT,
    DISCOVER_REQ,
    DISCOVER_RES,
    SERVICE_ANNOUNCE,
    SERVICE_GOODBYE,
    parse_message,
    build_discover_req,
    ServiceInfo,
    ServiceEvent,
)
from aiecho.config import ClientConfig

logger = logging.getLogger(__name__)


@dataclass
class DiscoveredService:
    """Complete discovered service with manifest."""
    service_info: ServiceInfo
    manifest: Optional[dict] = None
    manifest_loaded: bool = False

    @property
    def ip(self) -> str:
        return self.service_info.ip
    
    @property
    def port(self) -> int:
        return self.service_info.http_port
    
    @property
    def name(self) -> str:
        return self.service_info.service_name


class DiscoveryScanner:
    """UDP Discovery Scanner that broadcasts queries and collects responses."""
    
    def __init__(self, config: Optional[ClientConfig] = None, timeout: float = 2.0):
        self.config = config or ClientConfig()
        self.timeout = timeout or self.config.timeout

    async def scan(self, fetch_manifest: bool = True) -> list[DiscoveredService]:
        """Scan for services on the network."""
        services = await self._broadcast_and_collect()
        
        if fetch_manifest:
            await self._fetch_manifests(services)
        
        return services

    async def _broadcast_and_collect(self) -> list[DiscoveredService]:
        """Send broadcast and collect all responses."""
        services: dict[str, DiscoveredService] = {}
        
        loop = asyncio.get_event_loop()
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        
        try:
            sock.bind(("", 0))  # Bind to random ephemeral port
            sock.setblocking(False)
            
            # Send discovery request
            query_id = await loop.sock_sendto(
                sock,
                build_discover_req(),
                ("255.255.255.255", self.config.udp_port)
            )
            
            # Collect responses with timeout
            start_time = asyncio.get_event_loop().time()
            while asyncio.get_event_loop().time() - start_time < self.timeout:
                try:
                    ready, _, _ = await asyncio.wait_for(
                        asyncio.select([sock], [], [], 0.1),
                        timeout=0.1
                    )
                    
                    if sock in ready:
                        data, addr = await loop.sock_recvfrom(sock, 4096)
                        try:
                            cmd, payload = parse_message(data)
                            
                            if cmd == DISCOVER_RES and payload.get("status") == "ok":
                                service_info = ServiceInfo.from_payload(
                                    payload, ip=addr[0]
                                )
                                service_id = service_info.service_id
                                
                                if service_id not in services:
                                    services[service_id] = DiscoveredService(
                                        service_info=service_info
                                    )
                                logger.debug(f"Discovered: {service_info.service_name} @ {addr[0]}")
                                
                        except Exception as e:
                            logger.debug(f"Failed to parse response: {e}")
                            
                except asyncio.TimeoutError:
                    continue
        
        finally:
            sock.close()
        
        return list(services.values())

    async def _fetch_manifests(self, services: list[DiscoveredService]) -> None:
        """Fetch manifest for each discovered service."""
        async with asyncio.TaskGroup() as tg:
            for service in services:
                tg.create_task(self._fetch_manifest(service))

    async def _fetch_manifest(self, service: DiscoveredService) -> None:
        """Fetch manifest for a single service."""
        import aiohttp
        
        url = service.service_info.manifest_url
        timeout = aiohttp.ClientTimeout(total=2.0)
        
        try:
            async with aiohttp.ClientSession(timeout=timeout) as session:
                async with session.get(url) as response:
                    if response.status == 200:
                        service.manifest = await response.json()
                        service.manifest_loaded = True
                        logger.debug(f"Loaded manifest for {service.name}")
        except Exception as e:
            logger.debug(f"Failed to fetch manifest from {url}: {e}")
```

**Step 4: Run test to verify it passes**

```bash
cd python && pytest tests/test_scanner.py -v
Expected: PASS

**Step 5: Commit**

```bash
git add python/src/ai_discover/scanner.py tests/test_scanner.py
git commit -m "feat(python): implement discovery scanner client"
```

---

## Task 6: Real-time Listener Implementation

**Files:**
- Create: `python/src/ai_discover/listener.py`

**Step 1: Write the failing test**

```python
# tests/test_listener.py
import pytest
import asyncio
from aiecho.listener import DiscoveryListener
```
@pytest.mark.asyncio
async def test_listener_start_stop():
    listener = DiscoveryListener()
    await listener.start()
    assert listener.is_running()
    await listener.stop()
    assert not listener.is_running()
```

**Step 2: Run test to verify it fails**

```bash
cd python && pytest tests/test_listener.py::test_listener_start_stop -v
Expected: FAIL - listener module not found

**Step 3: Write listener.py implementation**

```python
"""Real-time Discovery Listener Implementation"""

import asyncio
import json
import socket
import logging
import time
from typing import Callable, Optional
from dataclasses import dataclass, field

from aiecho.protocol import (
    DISCOVERY_PORT,
    SERVICE_ANNOUNCE,
    SERVICE_GOODBYE,
    parse_message,
    ServiceEvent,
)
from aiecho.config import ClientConfig

logger = logging.getLogger(__name__)


@dataclass
class ServiceState:
    """Current state of a discovered service."""
    service_id: str
    service_name: str
    http_port: int
    manifest_path: str
    tags: list[str]
    priority: int
    ip: str
    last_seen: float = field(default_factory=time.time)
    manifest: Optional[dict] = None


class DiscoveryListener:
    """Real-time listener for service announcements and goodbyes."""
    
    def __init__(self, config: Optional[ClientConfig] = None):
        self.config = config or ClientConfig()
        self._running = False
        self._task: Optional[asyncio.Task] = None
        self._services: dict[str, ServiceState] = {}
        self._callbacks: dict[str, list[Callable]] = {
            "online": [],
            "offline": [],
        }
        self._socket: Optional[socket.socket] = None

    def is_running(self) -> bool:
        return self._running

    def on(self, event: str, callback: Callable[[ServiceState], None]) -> None:
        """Register callback for service events."""
        if event in self._callbacks:
            self._callbacks[event].append(callback)

    def off(self, event: str, callback: Callable) -> None:
        """Unregister callback."""
        if event in self._callbacks and callback in self._callbacks[event]:
            self._callbacks[event].remove(callback)

    def get_services(self) -> dict[str, ServiceState]:
        """Get current known services."""
        return self._services.copy()

    async def start(self) -> None:
        """Start the listener."""
        if self._running:
            return
        
        self._running = True
        self._task = asyncio.create_task(self._listen())
        logger.info("Discovery listener started")

    async def stop(self) -> None:
        """Stop the listener."""
        self._running = False
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
        
        if self._socket:
            self._socket.close()
            self._socket = None
        
        logger.info("Discovery listener stopped")

    async def _listen(self) -> None:
        """Main listening loop."""
        loop = asyncio.get_event_loop()
        self._socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        
        try:
            self._socket.bind(("", self.config.udp_port))
            self._socket.setblocking(False)
            
            logger.info(f"Listening on UDP port {self.config.udp_port}")
            
            while self._running:
                try:
                    ready, _, _ = await asyncio.wait_for(
                        asyncio.select([self._socket], [], [], 0.5),
                        timeout=0.5
                    )
                    
                    if self._socket in ready:
                        data, addr = await loop.sock_recvfrom(self._socket, 4096)
                        await self._handle_message(data, addr)
                        
                except asyncio.CancelledError:
                    break
                except Exception as e:
                    logger.error(f"Listener error: {e}")
        
        finally:
            if self._socket:
                self._socket.close()

    async def _handle_message(self, data: bytes, addr: tuple) -> None:
        """Handle incoming announcement/goodbye message."""
        try:
            cmd, payload = parse_message(data)
            ip = addr[0]
            
            if cmd == SERVICE_ANNOUNCE:
                event = ServiceEvent.from_payload(payload, ip)
                await self._handle_online(event)
                
            elif cmd == SERVICE_GOODBYE:
                event = ServiceEvent.from_payload(payload, ip)
                await self._handle_offline(event)
                
        except Exception as e:
            logger.debug(f"Failed to handle message: {e}")

    async def _handle_online(self, event: ServiceEvent) -> None:
        """Handle service coming online."""
        service = ServiceState(
            service_id=event.service_id,
            service_name=event.service_name,
            http_port=event.http_port,
            manifest_path=event.manifest_path,
            tags=event.tags,
            priority=event.priority,
            ip=event.ip,
        )
        
        is_new = event.service_id not in self._services
        self._services[event.service_id] = service
        
        logger.info(f"Service online: {service.service_name} @ {event.ip}")
        
        # Fetch manifest
        await self._fetch_manifest(service)
        
        # Notify callbacks
        for callback in self._callbacks["online"]:
            try:
                callback(service)
            except Exception as e:
                logger.error(f"Callback error: {e}")

    async def _handle_offline(self, event: ServiceEvent) -> None:
        """Handle service going offline."""
        service_id = event.service_id
        
        if service_id in self._services:
            del self._services[service_id]
            logger.info(f"Service offline: {event.service_name}")
            
            # Notify callbacks
            for callback in self._callbacks["offline"]:
                try:
                    callback(service_id)
                except Exception as e:
                    logger.error(f"Callback error: {e}")

    async def _fetch_manifest(self, service: ServiceState) -> None:
        """Fetch manifest for a service."""
        import aiohttp
        
        url = f"http://{service.ip}:{service.http_port}{service.manifest_path}"
        timeout = aiohttp.ClientTimeout(total=2.0)
        
        try:
            async with aiohttp.ClientSession(timeout=timeout) as session:
                async with session.get(url) as response:
                    if response.status == 200:
                        service.manifest = await response.json()
                        logger.debug(f"Loaded manifest for {service.service_name}")
        except Exception as e:
            logger.debug(f"Failed to fetch manifest: {e}")
```

**Step 4: Run test to verify it passes**

```bash
cd python && pytest tests/test_listener.py -v
Expected: PASS

**Step 5: Commit**

```bash
git add python/src/ai_discover/listener.py tests/test_listener.py
git commit -m "feat(python): implement real-time listener"
```

---

## Task 7: CLI Entry Points

**Files:**
- Create: `python/src/ai_discover/cli.py`

**Step 1: Write cli.py implementation**

```python
"""CLI Entry Points"""

import asyncio
import json
import sys
from pathlib import Path

import click
from rich.console import Console
from rich.table import Table

from aiecho.config import ServiceConfig, ClientConfig
from aiecho.server import DiscoveryServer
from aiecho.scanner import DiscoveryScanner
from aiecho.listener import DiscoveryListener

console = Console()


@click.group()
def cli():
    """AIEcho System"""
    pass


@cli.command()
@click.option(
    "--config", "-c",
    type=click.Path(exists=True),
    required=True,
    help="Service configuration JSON file"
)
def agent(config: str):
    """Run the discovery agent (service side)."""
    # Load configuration
    config_data = json.loads(Path(config).read_text())
    service_config = ServiceConfig(**config_data)
    
    console.print(f"[green]Starting discovery agent:[/green] {service_config.service_name}")
    console.print(f"  Service ID: {service_config.service_id}")
    console.print(f"  HTTP Port: {service_config.http_port}")
    console.print(f"  UDP Port: {service_config.udp_port}")
    
    # Start server
    server = DiscoveryServer(service_config)
    
    try:
        server.start()
        console.print("[green]Agent started. Press Ctrl+C to stop.[/green]")
        
        # Keep running
        while True:
            input()
    except KeyboardInterrupt:
        console.print("\n[yellow]Stopping agent...[/yellow]")
        server.stop()
        console.print("[green]Agent stopped.[/green]")


@cli.command()
@click.option(
    "--output", "-o",
    type=click.Choice(["json", "yaml", "table"]),
    default="json",
    help="Output format"
)
@click.option(
    "--timeout", "-t",
    type=float,
    default=2.0,
    help="Scan timeout in seconds"
)
@click.option(
    "--no-manifest",
    is_flag=True,
    help="Skip fetching manifests"
)
@click.option(
    "--output-file", "-f",
    type=click.Path(),
    help="Output to file instead of stdout"
)
def scan(output: str, timeout: float, no_manifest: bool, output_file: str):
    """Scan for services on the network."""
    console.print("[yellow]Scanning for services...[/yellow]")
    
    async def run_scan():
        scanner = DiscoveryScanner(timeout=timeout)
        services = await scanner.scan(fetch_manifest=not no_manifest)
        return services
    
    services = asyncio.run(run_scan())
    
    if not services:
        console.print("[red]No services found.[/red]")
        return
    
    # Output results
    if output == "json":
        result = [
            {
                "service_name": s.name,
                "ip": s.ip,
                "port": s.port,
                "manifest": s.manifest if s.manifest_loaded else None,
            }
            for s in services
        ]
        output_data = json.dumps(result, indent=2, ensure_ascii=False)
        
    elif output == "table":
        table = Table(title="Discovered Services")
        table.add_column("Name", style="cyan")
        table.add_column("IP", style="green")
        table.add_column("Port", style="yellow")
        table.add_column("Tags", style="magenta")
        
        for s in services:
            table.add_row(
                s.name,
                s.ip,
                str(s.port),
                ", ".join(s.service_info.tags) or "-"
            )
        
        console.print(table)
        return
        
    else:
        # YAML output (simple implementation)
        result = []
        for s in services:
            result.append({
                "service_name": s.name,
                "ip": s.ip,
                "port": s.port,
            })
            if s.manifest_loaded:
                result[-1]["manifest"] = s.manifest
        
        output_data = json.dumps(result, indent=2)
    
    # Write output
    if output_file:
        Path(output_file).write_text(output_data)
        console.print(f"[green]Output written to {output_file}[/green]")
    else:
        console.print(output_data)
    
    console.print(f"[green]Found {len(services)} service(s).[/green]")


@cli.command()
@click.option(
    "--output-file", "-f",
    type=click.Path(),
    required=True,
    help="Output services JSON file to watch"
)
@click.option(
    "--interval", "-i",
    type=int,
    default=30,
    help="Auto-scan interval in seconds"
)
def watch(output_file: str, interval: int):
    """Watch for service changes in real-time."""
    console.print(f"[yellow]Watching for service changes...[/yellow]")
    console.print(f"  Output file: {output_file}")
    console.print(f"  Auto-scan interval: {interval}s")
    
    # Track previous state
    previous_ids: set = set()
    
    async def run_watch():
        listener = DiscoveryListener()
        
        def on_online(service):
            console.print(f"[green]  + {service.service_name} @ {service.ip}:{service.http_port}[/green]")
        
        def on_offline(service_id):
            console.print(f"[red]  - {service_id}[/red]")
        
        listener.on("online", on_online)
        listener.on("offline", on_offline)
        
        await listener.start()
        
        # Write initial state
        _write_services_file(listener.get_services(), output_file)
        
        # Keep running
        while True:
            await asyncio.sleep(interval)
            
            # Auto-scan to refresh state
            scanner = DiscoveryScanner(timeout=2.0)
            services = await scanner.scan()
            
            current_ids = {s.service_info.service_id for s in services}
            
            # Update file
            service_states = {
                s.service_info.service_id: {
                    "service_name": s.service_info.service_name,
                    "ip": s.ip,
                    "port": s.port,
                    "manifest": s.manifest if s.manifest_loaded else None,
                }
                for s in services
            }
            _write_services_file(service_states, output_file)
            
            previous_ids = current_ids
    
    asyncio.run(run_watch())


def _write_services_file(services: dict, output_file: str):
    """Write services to JSON file."""
    Path(output_file).write_text(
        json.dumps(list(services.values()), indent=2, default vars, ensure_ascii=False)
    )


if __name__ == "__main__":
    cli()
```

**Step 2: Update pyproject.toml to add aiohttp dependency**

```toml
dependencies = [
    "click>=8.0.0",
    "pydantic>=2.0.0",
    "rich>=13.0.0",
    "zeroconf>=0.40.0",
    "aiohttp>=3.8.0",
]
```

**Step 3: Install and test**

```bash
cd python && pip install -e .
ai-scan --help
```

**Step 4: Commit**

```bash
git add python/src/ai_discover/cli.py python/pyproject.toml
git commit -m "feat(python): add CLI entry points"
```

---

## Task 8: Example Configuration

**Files:**
- Create: `python/examples/service_config.json`

**Step 1: Create example config**

```json
{
  "service_name": "PDF Converter Pro",
  "service_id": "pdf-converter-001",
  "http_port": 8080,
  "manifest_path": "/ai_manifest",
  "tags": ["pdf", "convert", "tool", "ai"],
  "priority": 10,
  "announce_on_startup": true
}
```

**Step 2: Commit**

```bash
git add python/examples/service_config.json
git commit -m "docs(python): add example configuration"
```

---

## Task 9: Final Integration Test

**Step 1: Run all tests**

```bash
cd python && pytest -v
```

**Step 2: Test CLI**

```bash
# Test agent help
ai-discover-agent --help

# Test scan help
ai-scan --help
```

**Step 3: Commit**

```bash
git add .
git commit -m "test(python): integration tests and CLI validation"
```
