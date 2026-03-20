"""CLI Entry Points

This module provides command-line interface tools:
- ai-discover-agent: Run the discovery server agent
- ai-scan: Scan for services on the network
- ai-listen: Listen for service changes in real-time
"""

import asyncio
import json
import logging
import sys
from pathlib import Path
from typing import Optional

import click
from rich.console import Console
from rich.table import Table

from ai_discover import __version__
from ai_discover.config import ServiceConfig, ClientConfig
from ai_discover.server import DiscoveryServer
from ai_discover.scanner import DiscoveryScanner, DiscoveredService
from ai_discover.listener import DiscoveryListener, ServiceState


console = Console()


def setup_logging(verbose: bool = False) -> None:
    """Setup logging configuration."""
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        datefmt="%H:%M:%S",
    )


@click.group()
@click.version_option(version=__version__)
def cli():
    """AI-LAN Service Discovery System
    
    A lightweight, zero-config LAN service discovery mechanism for AI agents.
    """
    pass


@cli.command()
@click.option(
    "--config", "-c",
    type=click.Path(exists=True, path_type=Path),
    required=True,
    help="Service configuration JSON file path"
)
@click.option(
    "--verbose", "-v",
    is_flag=True,
    help="Enable verbose logging"
)
@click.option(
    "--udp-port",
    type=int,
    default=None,
    help="Override UDP discovery port"
)
def agent(config: Path, verbose: bool, udp_port: Optional[int]):
    """Run the discovery agent (service side).
    
    This starts a UDP listener that responds to discovery requests
    and announces the service on the network.
    
    Example:
        ai-discover-agent --config service_config.json
    """
    setup_logging(verbose)
    
    try:
        # Load configuration
        console.print(f"[dim]Loading configuration from {config}[/dim]")
        service_config = ServiceConfig.from_file(config)
        
        # Override UDP port if specified
        if udp_port:
            service_config.udp_port = udp_port
        
        console.print(f"[green]Starting discovery agent:[/green] {service_config.service_name}")
        console.print(f"  [dim]Service ID:[/dim] {service_config.service_id}")
        console.print(f"  [dim]HTTP Port:[/dim] {service_config.http_port}")
        console.print(f"  [dim]UDP Port:[/dim] {service_config.udp_port}")
        console.print(f"  [dim]Announce on startup:[/dim] {service_config.announce_on_startup}")
        
        # Create and start server
        server = DiscoveryServer(service_config)
        server.start()
        
        console.print("[green]Agent started. Press Ctrl+C to stop.[/green]")
        
        # Keep running
        try:
            while True:
                input()
        except KeyboardInterrupt:
            console.print("\n[yellow]Stopping agent...[/yellow]")
            server.stop()
            console.print("[green]Agent stopped.[/green]")
            
    except FileNotFoundError:
        console.print(f"[red]Error: Configuration file not found: {config}[/red]")
        sys.exit(1)
    except Exception as e:
        console.print(f"[red]Error: {e}[/red]")
        if verbose:
            raise
        sys.exit(1)


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
    help="Skip fetching service manifests"
)
@click.option(
    "--output-file", "-f",
    type=click.Path(path_type=Path),
    help="Output to file instead of stdout"
)
@click.option(
    "--verbose", "-v",
    is_flag=True,
    help="Enable verbose logging"
)
def scan(
    output: str, 
    timeout: float, 
    no_manifest: bool, 
    output_file: Optional[Path],
    verbose: bool
):
    """Scan for services on the network.
    
    Sends a UDP broadcast and collects responses from all
    discoverable services.
    
    Example:
        ai-scan --output json
        ai-scan --output table --timeout 5
    """
    setup_logging(verbose)
    
    async def run_scan():
        console.print("[yellow]Scanning for services...[/yellow]")
        
        config = ClientConfig(timeout=timeout)
        scanner = DiscoveryScanner(config)
        services = await scanner.scan(fetch_manifest=not no_manifest)
        
        return services
    
    services = asyncio.run(run_scan())
    
    if not services:
        console.print("[red]No services found.[/red]")
        return
    
    # Format output
    if output == "json":
        result = _format_json(services)
    elif output == "yaml":
        result = _format_yaml(services)
    else:  # table
        _print_table(services)
        return
    
    # Output
    if output_file:
        output_file.write_text(result, encoding="utf-8")
        console.print(f"[green]Output written to {output_file}[/green]")
    else:
        console.print(result)
    
    console.print(f"[green]Found {len(services)} service(s).[/green]")


@cli.command()
@click.option(
    "--output-file", "-f",
    type=click.Path(path_type=Path),
    required=True,
    help="Output services JSON file to watch"
)
@click.option(
    "--interval", "-i",
    type=int,
    default=30,
    help="Auto-scan interval in seconds"
)
@click.option(
    "--no-manifest",
    is_flag=True,
    help="Skip fetching service manifests"
)
@click.option(
    "--verbose", "-v",
    is_flag=True,
    help="Enable verbose logging"
)
def listen(
    output_file: Path,
    interval: int,
    no_manifest: bool,
    verbose: bool
):
    """Listen for service changes in real-time.
    
    Monitors the network for service announcements and goodbyes,
    updating the output file when services come online or go offline.
    
    Example:
        ai-listen --output-file services.json
    """
    setup_logging(verbose)
    
    console.print(f"[yellow]Listening for service changes...[/yellow]")
    console.print(f"  [dim]Output file:[/dim] {output_file}")
    console.print(f"  [dim]Auto-scan interval:[/dim] {interval}s")
    
    # Track known services
    known_services: dict = {}
    
    async def run_listen():
        config = ClientConfig(scan_interval=interval)
        listener = DiscoveryListener(config)
        
        # Disable manifest fetching if requested
        if no_manifest:
            listener.set_fetch_manifests(False)
        
        async def on_online(service: ServiceState):
            known_services[service.service_id] = {
                "service_name": service.service_name,
                "ip": service.ip,
                "port": service.http_port,
                "tags": service.tags,
                "manifest": service.manifest if service.manifest_loaded else None,
            }
            console.print(f"[green]+[/green] {service.service_name} @ {service.ip}:{service.http_port}")
            _write_services_file(known_services, output_file)
        
        async def on_offline(service_id: str):
            if service_id in known_services:
                name = known_services[service_id]["service_name"]
                del known_services[service_id]
                console.print(f"[red]-[/red] {name} (offline)")
                _write_services_file(known_services, output_file)
        
        listener.on("online", on_online)
        listener.on("offline", on_offline)
        
        await listener.start()
        
        # Initial scan to populate known services
        console.print("[dim]Running initial scan...[/dim]")
        scanner = DiscoveryScanner(config)
        services = await scanner.scan(fetch_manifest=not no_manifest)
        
        for s in services:
            known_services[s.service_id] = {
                "service_name": s.name,
                "ip": s.ip,
                "port": s.port,
                "tags": s.tags,
                "manifest": s.manifest if s.manifest_loaded else None,
            }
        
        if known_services:
            _write_services_file(known_services, output_file)
            console.print(f"[green]Found {len(known_services)} service(s)[/green]")
        
        # Keep running
        try:
            while True:
                await asyncio.sleep(interval)
                # Periodic refresh
                services = await scanner.scan(fetch_manifest=not no_manifest)
                for s in services:
                    if s.service_id not in known_services:
                        known_services[s.service_id] = {
                            "service_name": s.name,
                            "ip": s.ip,
                            "port": s.port,
                            "tags": s.tags,
                            "manifest": s.manifest if s.manifest_loaded else None,
                        }
                if services:
                    _write_services_file(known_services, output_file)
        except asyncio.CancelledError:
            pass
        finally:
            await listener.stop()
    
    try:
        asyncio.run(run_listen())
    except KeyboardInterrupt:
        console.print("\n[yellow]Listener stopped.[/yellow]")


def _format_json(services):
    """Format services as JSON."""
    result = [
        {
            "service_name": s.name,
            "service_id": s.service_id,
            "ip": s.ip,
            "port": s.port,
            "tags": s.tags,
            "base_url": s.base_url,
            "manifest": s.manifest if s.manifest_loaded else None,
        }
        for s in services
    ]
    return json.dumps(result, indent=2, ensure_ascii=False)


def _format_yaml(services):
    """Format services as YAML (simple implementation)."""
    lines = []
    for s in services:
        lines.append(f"- service_name: {s.name}")
        lines.append(f"  service_id: {s.service_id}")
        lines.append(f"  ip: {s.ip}")
        lines.append(f"  port: {s.port}")
        if s.tags:
            lines.append(f"  tags:")
            for tag in s.tags:
                lines.append(f"    - {tag}")
    return "\n".join(lines)


def _print_table(services):
    """Print services as a rich table."""
    table = Table(title="Discovered Services")
    table.add_column("Name", style="cyan", no_wrap=True)
    table.add_column("IP", style="green")
    table.add_column("Port", style="yellow", no_wrap=True)
    table.add_column("Tags", style="magenta")
    table.add_column("Manifest", style="blue")
    
    for s in services:
        table.add_row(
            s.name,
            s.ip,
            str(s.port),
            ", ".join(s.tags) if s.tags else "-",
            "Yes" if s.manifest_loaded else "No"
        )
    
    console.print(table)


def _write_services_file(services, output_file: Path):
    """Write services to JSON file."""
    # Convert to list format
    result = []
    for service_id, info in services.items():
        item = {
            "service_id": service_id,
            **info
        }
        result.append(item)
    
    output_file.write_text(
        json.dumps(result, indent=2, ensure_ascii=False),
        encoding="utf-8"
    )


if __name__ == "__main__":
    cli()
