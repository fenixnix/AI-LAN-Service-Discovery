"""Real-time Discovery Listener Implementation

This module implements a real-time listener that:
- Monitors UDP port for service announcements and goodbyes
- Tracks service state changes in real-time
- Provides callbacks for online/offline events
- Can optionally fetch manifests on service discovery
"""

import asyncio
import json
import logging
import socket
import time
from dataclasses import dataclass, field, asdict
from typing import Callable, Optional, Any, Awaitable

import aiohttp

from ai_discover.protocol import (
    DISCOVERY_PORT,
    SERVICE_ANNOUNCE,
    SERVICE_GOODBYE,
    DISCOVER_RES,
    parse_message,
    ServiceEvent,
)
from ai_discover.config import ClientConfig

logger = logging.getLogger(__name__)


# Type alias for callbacks
ServiceCallback = Callable[["ServiceState"], Awaitable[None]]
ServiceOfflineCallback = Callable[[str], Awaitable[None]]


@dataclass
class ServiceState:
    """Current state of a discovered service.
    
    This represents a service that has been discovered through
    the real-time listener.
    """
    service_id: str
    service_name: str
    http_port: int
    manifest_path: str
    tags: list[str]
    priority: int
    ip: str
    last_seen: float = field(default_factory=time.time)
    manifest: Optional[dict] = None
    manifest_loaded: bool = False

    @property
    def base_url(self) -> str:
        """Get service base URL."""
        return f"http://{self.ip}:{self.http_port}"
    
    @property
    def manifest_url(self) -> str:
        """Get full manifest URL."""
        return f"{self.base_url}{self.manifest_path}"

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)


class DiscoveryListener:
    """Real-time listener for service announcements and goodbyes.
    
    This listener monitors the UDP discovery port for:
    - AI_SERVICE_ANNOUNCE: New service came online
    - AI_SERVICE_GOODBYE: Service went offline
    
    Example:
        ```python
        listener = DiscoveryListener()
        
        async def on_online(service: ServiceState):
            print(f"Service online: {service.service_name}")
            
        async def on_offline(service_id: str):
            print(f"Service offline: {service_id}")
        
        listener.on("online", on_online)
        listener.on("offline", on_offline)
        
        await listener.start()
        # ... listener is now running ...
        await listener.stop()
        ```
    """
    
    def __init__(self, config: Optional[ClientConfig] = None):
        """Initialize the discovery listener.
        
        Args:
            config: Client configuration (uses defaults if not provided)
        """
        self.config = config or ClientConfig()
        self._running = False
        self._task: Optional[asyncio.Task] = None
        self._services: dict[str, ServiceState] = {}
        self._callbacks: dict[str, list[ServiceCallback | ServiceOfflineCallback]] = {
            "online": [],
            "offline": [],
        }
        self._socket: Optional[socket.socket] = None
        self._fetch_manifests = True

    def is_running(self) -> bool:
        """Check if the listener is running."""
        return self._running

    def on(
        self, 
        event: str, 
        callback: ServiceCallback | ServiceOfflineCallback
    ) -> None:
        """Register callback for service events.
        
        Args:
            event: Event type ("online" or "offline")
            callback: Async callback function
            
        Callback signatures:
            - online: async def callback(service: ServiceState)
            - offline: async def callback(service_id: str)
        """
        if event in self._callbacks:
            self._callbacks[event].append(callback)
        else:
            logger.warning(f"Unknown event type: {event}")

    def off(
        self, 
        event: str, 
        callback: ServiceCallback | ServiceOfflineCallback
    ) -> None:
        """Unregister callback.
        
        Args:
            event: Event type
            callback: Previously registered callback
        """
        if event in self._callbacks:
            try:
                self._callbacks[event].remove(callback)
            except ValueError:
                pass

    def get_services(self) -> dict[str, ServiceState]:
        """Get current known services.
        
        Returns:
            Dictionary of service_id -> ServiceState
        """
        return self._services.copy()

    def set_fetch_manifests(self, enabled: bool) -> None:
        """Enable or disable automatic manifest fetching.
        
        Args:
            enabled: Whether to fetch manifests
        """
        self._fetch_manifests = enabled

    async def start(self) -> None:
        """Start the listener.
        
        This creates a UDP socket and starts the listening loop.
        """
        if self._running:
            logger.warning("Listener already running")
            return
        
        logger.info("Starting discovery listener")
        
        # Create UDP socket
        self._socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        
        try:
            self._socket.bind(("", self.config.udp_port))
            self._socket.setblocking(False)
        except OSError as e:
            logger.error(f"Failed to bind to UDP port {self.config.udp_port}: {e}")
            raise
        
        self._running = True
        
        # Start listening task
        self._task = asyncio.create_task(self._listen())
        
        logger.info(f"Discovery listener started on UDP port {self.config.udp_port}")

    async def stop(self) -> None:
        """Stop the listener."""
        if not self._running:
            return
        
        logger.info("Stopping discovery listener")
        
        self._running = False
        
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
        
        if self._socket:
            try:
                self._socket.close()
            except Exception as e:
                logger.debug(f"Error closing socket: {e}")
            self._socket = None
        
        logger.info("Discovery listener stopped")

    async def _listen(self) -> None:
        """Main listening loop."""
        loop = asyncio.get_event_loop()
        
        logger.debug("Listener loop started")
        
        while self._running:
            try:
                ready, _, _ = await asyncio.wait_for(
                    asyncio.select([self._socket], [], [], 0.5),
                    timeout=0.5
                )
                
                if self._socket in ready:
                    try:
                        data, addr = await loop.sock_recvfrom(self._socket, 4096)
                        await self._handle_message(data, addr)
                    except Exception as e:
                        logger.debug(f"Error receiving data: {e}")
                        
            except asyncio.CancelledError:
                break
            except Exception as e:
                if self._running:
                    logger.error(f"Listener error: {e}")
        
        logger.debug("Listener loop ended")

    async def _handle_message(
        self, 
        data: bytes, 
        addr: tuple
    ) -> None:
        """Handle incoming announcement/goodbye message.
        
        Args:
            data: Raw UDP message data
            addr: Source address (ip, port)
        """
        try:
            cmd, payload = parse_message(data)
            ip = addr[0]
            
            if cmd == SERVICE_ANNOUNCE:
                event = ServiceEvent.from_payload(payload, ip)
                await self._handle_online(event)
                
            elif cmd == SERVICE_GOODBYE:
                event = ServiceEvent.from_payload(payload, ip)
                await self._handle_offline(event)
                
            elif cmd == DISCOVER_RES:
                # Also handle discovery responses for initial scan
                logger.debug(f"Ignored discovery response in listener mode")
                
        except ValueError as e:
            logger.debug(f"Failed to parse message from {addr[0]}: {e}")
        except Exception as e:
            logger.debug(f"Error handling message: {e}")

    async def _handle_online(self, event: ServiceEvent) -> None:
        """Handle service coming online.
        
        Args:
            event: Service event with online information
        """
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
        
        logger.info(f"Service online: {service.service_name} @ {event.ip}:{event.http_port}")
        
        # Fetch manifest
        if self._fetch_manifests:
            await self._fetch_manifest(service)
        
        # Notify callbacks
        for callback in self._callbacks["online"]:
            try:
                await callback(service)
            except Exception as e:
                logger.error(f"Error in online callback: {e}")

    async def _handle_offline(self, event: ServiceEvent) -> None:
        """Handle service going offline.
        
        Args:
            event: Service event with offline information
        """
        service_id = event.service_id
        
        if service_id in self._services:
            service_name = self._services[service_id].service_name
            del self._services[service_id]
            logger.info(f"Service offline: {service_name} ({service_id})")
            
            # Notify callbacks
            for callback in self._callbacks["offline"]:
                try:
                    await callback(service_id)
                except Exception as e:
                    logger.error(f"Error in offline callback: {e}")

    async def _fetch_manifest(self, service: ServiceState) -> None:
        """Fetch manifest for a service.
        
        Args:
            service: Service to fetch manifest for
        """
        url = service.manifest_url
        timeout = aiohttp.ClientTimeout(total=2.0)
        
        try:
            async with aiohttp.ClientSession(timeout=timeout) as session:
                async with session.get(url) as response:
                    if response.status == 200:
                        service.manifest = await response.json()
                        service.manifest_loaded = True
                        logger.debug(f"Loaded manifest for {service.service_name}")
        except Exception as e:
            logger.debug(f"Failed to fetch manifest for {service.service_name}: {e}")


class SyncDiscoveryListener:
    """Synchronous wrapper for DiscoveryListener.
    
    Provides a simple synchronous interface for environments
    where async/await is not available.
    """
    
    def __init__(self, config: Optional[ClientConfig] = None):
        """Initialize the sync listener."""
        self.config = config or ClientConfig()
        self._listener: Optional[DiscoveryListener] = None
        self._loop: Optional[asyncio.AbstractEventLoop] = None
    
    def _ensure_started(self):
        """Ensure listener is started."""
        if self._listener is None:
            self._listener = DiscoveryListener(self.config)
            self._loop = asyncio.new_event_loop()
            asyncio.set_event_loop(self._loop)
    
    def start(self) -> None:
        """Start the listener synchronously."""
        self._ensure_started()
        self._loop.run_until_complete(self._listener.start())
    
    def stop(self) -> None:
        """Stop the listener synchronously."""
        if self._listener and self._running:
            self._loop.run_until_complete(self._listener.stop())
    
    def _running(self) -> bool:
        """Check if running."""
        return self._listener.is_running() if self._listener else False
    
    def get_services(self) -> dict[str, ServiceState]:
        """Get current services."""
        if self._listener:
            return self._listener.get_services()
        return {}
    
    def on(
        self, 
        event: str, 
        callback: Callable
    ) -> None:
        """Register callback."""
        self._ensure_started()
        
        async def wrapper(*args, **kwargs):
            callback(*args, **kwargs)
        
        self._listener.on(event, wrapper)
