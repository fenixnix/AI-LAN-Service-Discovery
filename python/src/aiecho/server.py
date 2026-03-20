"""Discovery Server (Agent) Implementation

This module implements the service-side discovery agent that:
- Listens for UDP discovery requests on port 53535
- Responds with service information
- Announces service on startup
- Announces service on shutdown (goodbye)
"""

import asyncio
import logging
import socket
import threading
import time
from typing import Optional

from ai_discover.protocol import (
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
from ai_discover.config import ServiceConfig

logger = logging.getLogger(__name__)


class DiscoveryServer:
    """UDP Discovery Server that responds to broadcast queries.
    
    This server runs alongside your main service and handles:
    - Responding to discovery requests (AI_DISCOVER_REQ)
    - Announcing service availability on startup
    - Announcing service shutdown on exit
    
    Example:
        ```python
        config = ServiceConfig(
            service_name="My Service",
            service_id="my-service-001",
            http_port=8080,
        )
        server = DiscoveryServer(config)
        server.start()
        # ... service is now discoverable ...
        server.stop()
        ```
    """
    
    def __init__(
        self, 
        config: ServiceConfig,
        http_server=None,  # Optional FastAPI/Flask server to manage
    ):
        """Initialize the discovery server.
        
        Args:
            config: Service configuration
            http_server: Optional HTTP server instance for manifest endpoint
        """
        self.config = config
        self.http_server = http_server
        self._running = False
        self._thread: Optional[threading.Thread] = None
        self._socket: Optional[socket.socket] = None
        self._announce_task: Optional[threading.Thread] = None
        self._stop_event = threading.Event()
        
    def is_running(self) -> bool:
        """Check if the server is running."""
        return self._running

    def start(self) -> None:
        """Start the discovery server in a background thread.
        
        This method:
        1. Creates a UDP socket bound to the discovery port
        2. Starts the message handling loop in a daemon thread
        3. Sends initial service announcement if configured
        4. Starts periodic announcement thread if configured
        """
        if self._running:
            logger.warning("Server already running")
            return
        
        logger.info(f"Starting discovery server for '{self.config.service_name}'")
        
        # Create and configure UDP socket
        self._socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self._socket.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
        self._socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        
        try:
            # Bind to all interfaces on the configured UDP port
            self._socket.bind(("", self.config.udp_port))
            self._socket.settimeout(0.5)  # Non-blocking with timeout
        except OSError as e:
            logger.error(f"Failed to bind to UDP port {self.config.udp_port}: {e}")
            raise
        
        self._running = True
        self._stop_event.clear()
        
        # Start message handling thread
        self._thread = threading.Thread(target=self._run, daemon=True, name="ai-discover-server")
        self._thread.start()
        
        # Send initial announcement
        if self.config.announce_on_startup:
            time.sleep(0.1)  # Small delay to ensure socket is ready
            self._send_announce()
        
        # Start periodic announcement thread
        if self.config.announce_interval > 0:
            self._announce_task = threading.Thread(
                target=self._run_periodic_announce, 
                daemon=True,
                name="ai-discover-announcer"
            )
            self._announce_task.start()
        
        logger.info(
            f"Discovery server started on UDP port {self.config.udp_port}. "
            f"Service: {self.config.service_name} (ID: {self.config.service_id})"
        )

    def stop(self) -> None:
        """Stop the discovery server gracefully.
        
        This method:
        1. Sends goodbye announcement if configured
        2. Stops the message handling loop
        3. Closes the UDP socket
        """
        if not self._running:
            return
        
        logger.info(f"Stopping discovery server for '{self.config.service_name}'")
        
        # Send goodbye message
        self._send_goodbye()
        
        # Signal stop
        self._running = False
        self._stop_event.set()
        
        # Wait for threads to finish
        if self._thread and self._thread.is_alive():
            self._thread.join(timeout=2.0)
        
        if self._announce_task and self._announce_task.is_alive():
            self._announce_task.join(timeout=1.0)
        
        # Close socket
        if self._socket:
            try:
                self._socket.close()
            except Exception as e:
                logger.debug(f"Error closing socket: {e}")
            self._socket = None
        
        logger.info(f"Discovery server stopped for '{self.config.service_name}'")

    def _run(self) -> None:
        """Main server loop - handles incoming UDP messages."""
        logger.debug("Discovery server loop started")
        
        while self._running and self._socket:
            try:
                try:
                    data, addr = self._socket.recvfrom(4096)
                    self._handle_message(data, addr)
                except socket.timeout:
                    continue
            except Exception as e:
                if self._running:
                    logger.error(f"Error in server loop: {e}")
                break
        
        logger.debug("Discovery server loop ended")

    def _handle_message(self, data: bytes, addr: tuple) -> None:
        """Handle incoming UDP message.
        
        Args:
            data: Raw UDP message data
            addr: Source address (ip, port)
        """
        try:
            cmd, payload = parse_message(data)
            
            if cmd == DISCOVER_REQ:
                query_id = payload.get("query_id", "")
                logger.debug(f"Discovery request from {addr[0]}, query_id={query_id}")
                
                # Build and send response
                response = build_discover_res(
                    query_id=query_id,
                    status="ok",
                    service_name=self.config.service_name,
                    service_id=self.config.service_id,
                    http_port=self.config.http_port,
                    manifest_path=self.config.manifest_path,
                    tags=self.config.tags,
                    priority=self.config.priority,
                )
                
                self._socket.sendto(response, addr)
                logger.debug(f"Sent discovery response to {addr[0]}")
                
            else:
                logger.debug(f"Ignored unknown command: {cmd}")
                
        except ValueError as e:
            logger.debug(f"Failed to parse message from {addr[0]}: {e}")
        except Exception as e:
            logger.error(f"Error handling message from {addr[0]}: {e}")

    def _send_announce(self) -> None:
        """Broadcast service announcement (online)."""
        if not self._socket:
            return
        
        try:
            msg = build_announce(
                service_id=self.config.service_id,
                service_name=self.config.service_name,
                http_port=self.config.http_port,
                manifest_path=self.config.manifest_path,
                tags=self.config.tags,
                priority=self.config.priority,
            )
            self._socket.sendto(msg, ("255.255.255.255", self.config.udp_port))
            logger.info("Service announcement sent")
        except Exception as e:
            logger.error(f"Failed to send announcement: {e}")

    def _send_goodbye(self) -> None:
        """Broadcast service goodbye (offline)."""
        if not self._socket:
            return
        
        try:
            msg = build_goodbye(
                service_id=self.config.service_id,
                service_name=self.config.service_name,
            )
            self._socket.sendto(msg, ("255.255.255.255", self.config.udp_port))
            logger.info("Service goodbye sent")
        except Exception as e:
            logger.debug(f"Failed to send goodbye: {e}")

    def _run_periodic_announce(self) -> None:
        """Run periodic announcement broadcasts."""
        logger.debug("Periodic announcer started")
        
        while self._running and not self._stop_event.is_set():
            self._stop_event.wait(self.config.announce_interval)
            if self._running and not self._stop_event.is_set():
                self._send_announce()
        
        logger.debug("Periodic announcer stopped")


class AsyncDiscoveryServer:
    """Async version of DiscoveryServer using asyncio.
    
    This is an alternative implementation using Python's asyncio
    for environments that prefer async I/O.
    """
    
    def __init__(self, config: ServiceConfig):
        """Initialize the async discovery server."""
        self.config = config
        self._running = False
        self._socket: Optional[asyncio.DatagramProtocol] = None
        self._transport: Optional[asyncio.DatagramTransport] = None
        
    def is_running(self) -> bool:
        """Check if the server is running."""
        return self._running
    
    async def start(self) -> None:
        """Start the async discovery server."""
        if self._running:
            return
            
        logger.info(f"Starting async discovery server for '{self.config.service_name}'")
        
        loop = asyncio.get_event_loop()
        
        # Create UDP socket
        self._transport, self._socket = await loop.create_datagram_endpoint(
            lambda: _DiscoveryProtocol(self),
            local_addr=("", self.config.udp_port),
            reuse_address=True,
        )
        
        # Enable broadcast
        self._transport.set_broadcast(True)
        
        self._running = True
        
        # Send initial announcement
        if self.config.announce_on_startup:
            await asyncio.sleep(0.1)
            await self._send_announce()
            
        logger.info(f"Async discovery server started on UDP port {self.config.udp_port}")
        
    async def stop(self) -> None:
        """Stop the async discovery server."""
        if not self._running:
            return
            
        logger.info(f"Stopping async discovery server for '{self.config.service_name}'")
        
        await self._send_goodbye()
        
        self._running = False
        
        if self._transport:
            self._transport.close()
            
        logger.info(f"Async discovery server stopped")
        
    async def _send_announce(self) -> None:
        """Send service announcement."""
        if not self._transport:
            return
            
        msg = build_announce(
            service_id=self.config.service_id,
            service_name=self.config.service_name,
            http_port=self.config.http_port,
            manifest_path=self.config.manifest_path,
            tags=self.config.tags,
            priority=self.config.priority,
        )
        
        self._transport.sendto(msg, ("255.255.255.255", self.config.udp_port))
        
    async def _send_goodbye(self) -> None:
        """Send service goodbye."""
        if not self._transport:
            return
            
        msg = build_goodbye(
            service_id=self.config.service_id,
            service_name=self.config.service_name,
        )
        
        self._transport.sendto(msg, ("255.255.255.255", self.config.udp_port))


class _DiscoveryProtocol:
    """Async UDP protocol handler for AsyncDiscoveryServer."""
    
    def __init__(self, server: AsyncDiscoveryServer):
        self.server = server
        
    def datagram_received(self, data: bytes, addr: tuple) -> None:
        """Handle incoming UDP datagram."""
        try:
            cmd, payload = parse_message(data)
            
            if cmd == DISCOVER_REQ:
                query_id = payload.get("query_id", "")
                
                response = build_discover_res(
                    query_id=query_id,
                    status="ok",
                    service_name=self.server.config.service_name,
                    service_id=self.server.config.service_id,
                    http_port=self.server.config.http_port,
                    manifest_path=self.server.config.manifest_path,
                    tags=self.server.config.tags,
                    priority=self.server.config.priority,
                )
                
                if self.server._transport:
                    self.server._transport.sendto(response, addr)
                    
        except Exception as e:
            logger.debug(f"Error handling datagram: {e}")
