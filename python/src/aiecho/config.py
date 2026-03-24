"""Configuration Models

This module provides Pydantic models for service and client configuration.
"""

from pathlib import Path
from typing import Optional, Any

from pydantic import BaseModel, Field, field_validator


class ServiceConfig(BaseModel):
    """Service provider configuration.
    
    This configuration is used by the discovery agent (ai-discover-agent)
    to announce service presence on the network.
    
    Example:
        ```json
        {
            "service_name": "PDF Converter Pro",
            "service_id": "pdf-converter-001",
            "http_port": 8080,
            "manifest_path": "/ai_manifest",
            "tags": ["pdf", "convert", "tool"],
            "priority": 10,
            "announce_on_startup": true
        }
        ```
    """
    
    service_name: str = Field(
        ..., 
        description="Human-readable service name",
        min_length=1,
        max_length=255,
    )
    service_id: str = Field(
        ..., 
        description="Unique service identifier",
        min_length=1,
        max_length=255,
    )
    http_port: int = Field(
        ..., 
        ge=1, 
        le=65535, 
        description="HTTP service port"
    )
    manifest_path: str = Field(
        default="/ai_manifest",
        description="Manifest endpoint path (must start with /)"
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
        ge=1,
        le=65535,
        description="UDP discovery port"
    )
    announce_on_startup: bool = Field(
        default=True,
        description="Announce service on startup"
    )
    announce_interval: int = Field(
        default=30,
        ge=5,
        le=300,
        description="Announcement broadcast interval in seconds (0 to disable)"
    )

    @field_validator("manifest_path")
    @classmethod
    def validate_manifest_path(cls, v: str) -> str:
        if not v.startswith("/"):
            v = "/" + v
        return v

    @property
    def base_url(self) -> str:
        """Get base URL for the service."""
        return f"http://localhost:{self.http_port}"

    @property
    def manifest_url(self) -> str:
        """Get full manifest URL."""
        return f"{self.base_url}{self.manifest_path}"

    @classmethod
    def from_manifest(cls, manifest: dict, http_port: int) -> "ServiceConfig":
        """Create ServiceConfig from manifest dictionary.
        
        Args:
            manifest: Manifest dictionary
            http_port: HTTP service port
            
        Returns:
            ServiceConfig instance
        """
        meta = manifest.get("meta", {})
        return cls(
            service_name=meta.get("name", "Unknown Service"),
            service_id=meta.get("service_id", f"service-{http_port}"),
            http_port=http_port,
            manifest_path=manifest.get("endpoints", {}).get("invoke", "/ai_manifest"),
            tags=[],
            priority=1
        )
    
    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return self.model_dump()


class EchoConfig(BaseModel):
    """Echo configuration for service discovery.
    
    This configuration is loaded from .echo files.
    
    Example:
        ```json
        {
            "port": 8080,
            "enable": true
        }
        ```
    """
    
    port: int = Field(
        ..., 
        ge=1, 
        le=65535, 
        description="HTTP service port"
    )
    enable: bool = Field(
        default=True,
        description="Whether the service is enabled"
    )

    @classmethod
    def from_file(cls, path: str | Path) -> "EchoConfig":
        """Load configuration from .echo file.
        
        Args:
            path: Path to .echo file
            
        Returns:
            EchoConfig instance
        """
        import json
        path = Path(path)
        if not path.exists():
            raise FileNotFoundError(f"Echo file not found: {path}")
        data = json.loads(path.read_text(encoding="utf-8"))
        return cls(**data)


class ClientConfig(BaseModel):
    """Client scanner configuration.
    
    This configuration is used by the scanner (ai-scan) to discover
    services on the network.
    """
    
    udp_port: int = Field(
        default=53535,
        ge=1,
        le=65535,
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
    fetch_manifest: bool = Field(
        default=True,
        description="Automatically fetch service manifests"
    )
    max_concurrent: int = Field(
        default=10,
        ge=1,
        le=100,
        description="Maximum concurrent manifest fetches"
    )

    @field_validator("output_format")
    @classmethod
    def validate_output_format(cls, v: str) -> str:
        valid_formats = ["json", "yaml", "table"]
        if v.lower() not in valid_formats:
            raise ValueError(f"Invalid output format: {v}. Must be one of {valid_formats}")
        return v.lower()

    class Config:
        use_enum_values = True
    
    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return self.model_dump()


class ManifestMeta(BaseModel):
    """Manifest metadata section."""
    service_id: str = Field(..., description="Unique service identifier")
    name: str = Field(..., description="Service display name")
    version: str = Field(default="1.0.0", description="Service version")
    description: Optional[str] = Field(default=None, description="Service description")
    author: Optional[str] = Field(default=None, description="Service author")
    uptime_seconds: Optional[int] = Field(default=None, description="Service uptime in seconds")


class CapabilityInputSchema(BaseModel):
    """Capability input schema."""
    type: str = Field(default="object", description="JSON schema type")
    properties: dict[str, Any] = Field(default_factory=dict, description="Schema properties")
    required: Optional[list[str]] = Field(default=None, description="Required fields")


class CapabilityOutputSchema(BaseModel):
    """Capability output schema."""
    type: str = Field(default="object", description="JSON schema type")
    properties: dict[str, Any] = Field(default_factory=dict, description="Schema properties")


class Capability(BaseModel):
    """Service capability definition."""
    id: str = Field(..., description="Capability unique identifier")
    name: str = Field(..., description="Capability display name")
    description: str = Field(..., description="Capability description")
    input_schema: CapabilityInputSchema = Field(
        default_factory=CapabilityInputSchema,
        description="Input parameter schema"
    )
    output_schema: CapabilityOutputSchema = Field(
        default_factory=CapabilityOutputSchema,
        description="Output result schema"
    )


class ManifestEndpoints(BaseModel):
    """Manifest endpoints section."""
    base_url: str = Field(..., description="Service base URL")
    health_check: Optional[str] = Field(default="/health", description="Health check endpoint")
    invoke: str = Field(default="/api/v1/invoke", description="Capability invoke endpoint")


class ManifestAuth(BaseModel):
    """Manifest authentication configuration."""
    type: str = Field(default="none", description="Auth type: bearer_token, api_key, none")
    token_location: Optional[str] = Field(default=None, description="Token location: header, query")


class Manifest(BaseModel):
    """Service Manifest - complete capability description.
    
    This is the standard format that AI agents use to understand
    what capabilities a service provides and how to invoke them.
    """
    meta: ManifestMeta = Field(..., description="Service metadata")
    capabilities: list[Capability] = Field(
        default_factory=list,
        description="List of service capabilities"
    )
    endpoints: ManifestEndpoints = Field(..., description="Service endpoints")
    auth: ManifestAuth = Field(
        default_factory=ManifestAuth,
        description="Authentication configuration"
    )

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "Manifest":
        """Create Manifest from dictionary."""
        return cls(**data)
    
    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return self.model_dump()
