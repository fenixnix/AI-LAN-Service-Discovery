"""Discovery Scanner (Client) Implementation

This module implements the client-side scanner that:
- Sends UDP broadcast discovery requests
- Collects responses from all services
- Fetches service manifests via HTTP
- Returns standardized service information
"""

import asyncio
import json
import logging
import socket
import time
from dataclasses import dataclass, field
from typing import Optional, Any

import aiohttp

from ai_discover.protocol import (
    DISCOVERY_PORT,
    DISCOVER_REQ,
    DISCOVER_RES,
    parse_message,
    build_discover_req,
    ServiceInfo,
)
from ai_discover.config import ClientConfig, Manifest

logger = logging.getLogger(__name__)


@dataclass
class DiscoveredService:
    """Complete discovered service with manifest.
    
    This is the main result object returned by DiscoveryScanner.
    """
    service_info: ServiceInfo
    manifest: Optional[dict] = None
    manifest_loaded: bool = False
    manifest_error: Optional[str] = None

    @property
    def ip(self) -> str:
        """Get service IP address."""
        return self.service_info.ip
    
    @property
    def port(self) -> int:
        """Get service HTTP port."""
        return self.service_info.http_port
    
    @property
    def name(self) -> str:
        """Get service name."""
        return self.service_info.service_name
    
    @property
    def service_id(self) -> str:
        """Get service ID."""
        return self.service_info.service_id
    
    @property
    def tags(self) -> list[str]:
        """Get service tags."""
        return self.service_info.tags
    
    @property
    def base_url(self) -> str:
        """Get service base URL."""
        return self.service_info.base_url
    
    @property
    def manifest_url(self) -> str:
        """Get service manifest URL."""
        return self.service_info.manifest_url

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for serialization."""
        return {
            "service_name": self.name,
            "service_id": self.service_id,
            "ip": self.ip,
            "port": self.port,
            "tags": self.tags,
            "base_url": self.base_url,
            "manifest_url": self.manifest_url,
            "manifest": self.manifest if self.manifest_loaded else None,
            "manifest_loaded": self.manifest_loaded,
        }


class DiscoveryScanner:
    """UDP Discovery Scanner that broadcasts queries and collects responses.
    
    Example:
        ```python
        scanner = DiscoveryScanner()
        services = await scanner.scan()
        
        for service in services:
            print(f"Found: {service.name} @ {service.ip}:{service.port}")
            if service.manifest:
                print(f"  Capabilities: {len(service.manifest.get('capabilities', []))}")
        ```
    """
    
    def __init__(
        self, 
        config: Optional[ClientConfig] = None,
        timeout: Optional[float] = None,
    ):
        """Initialize the discovery scanner.
        
        Args:
            config: Client configuration (uses defaults if not provided)
            timeout: Scan timeout in seconds (overrides config)
        """
        self.config = config or ClientConfig()
        self.timeout = timeout or self.config.timeout

    async def scan(
        self, 
        fetch_manifest: Optional[bool] = None,
    ) -> list[DiscoveredService]:
        """Scan for services on the network.
        
        This method:
        1. Creates a UDP socket
        2. Sends broadcast discovery request
        3. Collects all responses within timeout
        4. Optionally fetches manifests for each service
        
        Args:
            fetch_manifest: Whether to fetch manifests (defaults to config value)
            
        Returns:
            List of discovered services with their information
        """
        should_fetch = fetch_manifest if fetch_manifest is not None else self.config.fetch_manifest
        
        # Phase 1: Broadcast and collect
        services = await self._broadcast_and_collect()
        
        if not services:
            logger.debug("No services discovered")
            return []
        
        logger.info(f"Discovered {len(services)} service(s)")
        
        # Phase 2: Fetch manifests (concurrent)
        if should_fetch:
            await self._fetch_manifests(services)
        
        return services

    async def _broadcast_and_collect(self) -> list[DiscoveredService]:
        """Send broadcast and collect all responses.
        
        Returns:
            List of discovered services (without manifests)
        """
        services: dict[str, DiscoveredService] = {}
        
        loop = asyncio.get_event_loop()
        
        # Create UDP socket
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        
        try:
            # Bind to random ephemeral port
            sock.bind(("", 0))
            sock.setblocking(False)
            
            # Send discovery request
            request_msg = build_discover_req()
            await loop.sock_sendto(
                sock,
                request_msg,
                ("255.255.255.255", self.config.udp_port)
            )
            logger.debug(f"Sent discovery request to broadcast address")
            
            # Collect responses with timeout
            start_time = time.time()
            last_activity = start_time
            
            while time.time() - start_time < self.timeout:
                # Check if we should continue (no activity timeout)
                if time.time() - last_activity > 0.5:
                    # No new responses for 500ms, might be done
                    if len(services) > 0:
                        break
                
                try:
                    # Use select for timeout
                    ready, _, _ = await asyncio.wait_for(
                        asyncio.select([sock], [], [], 0.1),
                        timeout=0.1
                    )
                    
                    if sock in ready:
                        data, addr = await loop.sock_recvfrom(sock, 4096)
                        last_activity = time.time()
                        
                        try:
                            cmd, payload = parse_message(data)
                            
                            if cmd == DISCOVER_RES and payload.get("status") == "ok":
                                service_info = ServiceInfo.from_payload(
                                    payload, ip=addr[0]
                                )
                                service_id = service_info.service_id
                                
                                if service_id and service_id not in services:
                                    services[service_id] = DiscoveredService(
                                        service_info=service_info
                                    )
                                    logger.debug(
                                        f"Discovered: {service_info.service_name} "
                                        f"@ {addr[0]}:{service_info.http_port}"
                                    )
                                    
                        except ValueError as e:
                            logger.debug(f"Failed to parse response from {addr[0]}: {e}")
                            
                except asyncio.TimeoutError:
                    continue
        
        finally:
            sock.close()
        
        return list(services.values())

    async def _fetch_manifests(self, services: list[DiscoveredService]) -> None:
        """Fetch manifest for each discovered service concurrently.
        
        Args:
            services: List of discovered services to fetch manifests for
        """
        if not services:
            return
            
        logger.debug(f"Fetching manifests for {len(services)} service(s)")
        
        # Limit concurrent requests
        semaphore = asyncio.Semaphore(self.config.max_concurrent)
        
        async def fetch_with_limit(service: DiscoveredService):
            async with semaphore:
                await self._fetch_manifest(service)
        
        # Run all fetches concurrently
        await asyncio.gather(
            *[fetch_with_limit(s) for s in services],
            return_exceptions=True
        )

    async def _fetch_manifest(self, service: DiscoveredService) -> None:
        """Fetch manifest for a single service.
        
        Args:
            service: Service to fetch manifest for
        """
        url = service.manifest_url
        timeout = aiohttp.ClientTimeout(total=3.0)
        
        try:
            async with aiohttp.ClientSession(timeout=timeout) as session:
                async with session.get(url) as response:
                    if response.status == 200:
                        service.manifest = await response.json()
                        service.manifest_loaded = True
                        logger.debug(f"Loaded manifest for {service.name}")
                    else:
                        service.manifest_error = f"HTTP {response.status}"
                        logger.warning(
                            f"Manifest request for {service.name} returned {response.status}"
                        )
        except asyncio.TimeoutError:
            service.manifest_error = "Timeout"
            logger.debug(f"Manifest request timeout for {service.name}")
        except aiohttp.ClientError as e:
            service.manifest_error = str(e)
            logger.debug(f"Manifest request failed for {service.name}: {e}")
        except Exception as e:
            service.manifest_error = str(e)
            logger.debug(f"Unexpected error fetching manifest for {service.name}: {e}")


class SyncDiscoveryScanner:
    """Synchronous wrapper for DiscoveryScanner.
    
    Provides a simple synchronous interface for environments
    where async/await is not available.
    """
    
    def __init__(
        self, 
        config: Optional[ClientConfig] = None,
        timeout: Optional[float] = None,
    ):
        """Initialize the sync scanner."""
        self.config = config or ClientConfig()
        self.timeout = timeout or self.config.timeout
    
    def scan(
        self, 
        fetch_manifest: Optional[bool] = None,
    ) -> list[DiscoveredService]:
        """Scan for services synchronously.
        
        Args:
            fetch_manifest: Whether to fetch manifests
            
        Returns:
            List of discovered services
        """
        scanner = DiscoveryScanner(self.config, self.timeout)
        return asyncio.run(scanner.scan(fetch_manifest=fetch_manifest))
