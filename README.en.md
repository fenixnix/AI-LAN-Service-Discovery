# AIEcho

## Language / 语言

- [English](README.en.md)
- [中文](README.zh-CN.md)

<p align="center">
  <a href="https://github.com/fenixnix/AI-LAN-Service-Discovery/releases">
    <img src="https://img.shields.io/github/v/release/fenixnix/AI-LAN-Service-Discovery?include_prereleases&label=release" alt="Release"/>
  </a>
  <a href="https://www.python.org/">
    <img src="https://img.shields.io/badge/Python-3.8+-blue.svg" alt="Python"/>
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Rust-1.70+-orange.svg" alt="Rust"/>
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License"/>
  </a>
</p>

---

## Overview

AIEcho is a lightweight, zero-configuration, high-performance LAN AI microservice discovery mechanism. This system enables AI Agents to dynamically discover and call various AI tool services deployed on the local network (such as PDF processing, image generation, knowledge base retrieval, etc.).

### Core Features

- 🚀 **Millisecond Discovery** - UDP broadcast mechanism, millisecond response
- 🔄 **Real-time Awareness** - Service online auto-announcement, client dynamically updates service list
- 📦 **Zero-code Access** - Only need JSON configuration file to complete service registration
- 🌐 **Multi-language Support** - Python and Rust native implementation
- 🔌 **Standardized Interface** - RESTful API + JSON Schema capability description

---

## System Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────────┐
│                        LAN Network                              │
│                                                                 │
│  ┌──────────────────┐                           ┌───────────┐ │
│  │   AI Scanner     │◄────── UDP Broadcast ──────│  Service  │ │
│  │   (Client)      │◄────── HTTP Manifest ──────│  Agent    │ │
│  └──────────────────┘                           └───────────┘ │
│          │                                                │     │
│          │                                                │     │
│          ▼                                                ▼     │
│  ┌──────────────────┐                           ┌───────────┐ │
│  │  services.json   │◄────── Watch/Update ──────│  Main     │ │
│  │  (Local Cache)   │                           │  Service  │ │
│  └──────────────────┘                           └───────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Two-phase Discovery Protocol

| Phase                    | Protocol                    | Purpose                 | Latency  |
| ----------------------- | ----------------------- | -------------------- | ----- |
| **Discovery**     | UDP Broadcast (53535)   | Quick locate service IP/port | <10ms |
| **Introspection** | HTTP GET (/ai_manifest) | Get complete capability description     | <50ms |

---

## Quick Start

### 1. Server Side (Service Provider)

**Only need a JSON configuration file:**

```json
{
  "service_name": "PDF Converter Pro",
  "service_id": "pdf-converter-001",
  "http_port": 8080,
  "manifest_path": "/ai_manifest",
  "tags": ["pdf", "convert", "tool"],
  "priority": 10
}
```

**Start command:**

```bash
# Python
pip install ai-discover
ai-discover-agent --config service_config.json
```bash
# or Rust (cargo)
cargo install aiecho
aiecho --config service_config.json
```

### 2. Client Side (AI Scanner)

**One command to get all services:**

```bash
# Install
pip install ai-discover

# Python version
ai-scan --output json
ai-scan --output table

# Rust version
aiecho scan --output json
aiecho scan --output table
```

**Output example:**

```json
[
  {
    "service_name": "PDF Converter Pro",
    "ip": "192.168.1.50",
    "port": 8080,
    "manifest": {
      "meta": {
        "service_id": "pdf-converter-001",
        "name": "PDF Converter Pro",
        "version": "1.2.0"
      },
      "capabilities": [...]
    }
  }
]
```

---

## Protocol Specification

### UDP Discovery Protocol

| Parameter                     | Value            | Description            |
| ------------------------ | ------------- | --------------- |
| **Discovery Port** | `53535`     | UDP listening port    |
| **Transport**      | UDP Broadcast | 255.255.255.255 |
| **Encoding**       | UTF-8         | All payload encoding    |
| **Timeout**        | `2.0s`      | Client wait timeout  |

#### Discovery Request

```text
AI_DISCOVER_REQ
{"query_id": "uuid-1234-5678", "version": "1.0"}
```

#### Discovery Response

```text
AI_DISCOVER_RES
{
  "query_id": "uuid-1234-5678",
  "status": "ok",
  "service_name": "PDF-Processor",
  "service_id": "pdf-proc-001",
  "http_port": 8080,
  "manifest_path": "/ai_manifest",
  "tags": ["pdf", "tool"],
  "priority": 1,
  "version": "1.0"
}
```

### HTTP Manifest Interface

**Endpoint:** `GET /ai_manifest`

**Response:**

```json
{
  "meta": {
    "service_id": "pdf-proc-001",
    "name": "Local PDF Processor",
    "version": "1.2.0",
    "description": "PDF to Markdown tool",
    "author": "MyTeam"
  },
  "capabilities": [
    {
      "id": "convert_pdf_to_md",
      "name": "Convert PDF to Markdown",
      "description": "Convert PDF files to Markdown",
      "input_schema": {
        "type": "object",
        "properties": {
          "file_path": {
            "type": "string",
            "description": "PDF file path"
          }
        },
        "required": ["file_path"]
      },
      "output_schema": {
        "type": "object",
        "properties": {
          "content": {
            "type": "string",
            "description": "Markdown content"
          }
        }
      }
    }
  ],
  "endpoints": {
    "base_url": "http://192.168.1.50:8080",
    "invoke": "/api/v1/invoke"
  },
  "auth": {
    "type": "bearer_token",
    "token_location": "header"
  }
}
```

---

## New Service Online Announcement Mechanism

### Core Design

When the server goes online, it will **actively broadcast** announcement messages, and clients can listen and update the local service list in real-time.

```
┌─────────────────────────────────────────────────────────────────┐
│                     Service Announcement                        │
│                                                                 │
│   Service Startup                                               │
│        │                                                        │
│        ▼                                                        │
│   ┌─────────┐    Broadcast    ┌─────────────┐                  │
│   │ Service │ ──────────────► │  services   │                  │
│   │ Agent   │   ANNOUNCE       │  .json      │                  │
│   └─────────┘    on port       │  (updated)  │                  │
│        │        53535          └─────────────┘                  │
│        ▼                                                        │
│   Wait for discovery requests                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Announcement Message Format

```text
AI_SERVICE_ANNOUNCE
{
  "event": "online",
  "service_id": "pdf-proc-001",
  "service_name": "PDF Processor",
  "http_port": 8080,
  "manifest_path": "/ai_manifest",
  "tags": ["pdf", "tool"],
  "timestamp": 1699999999
}
```

### Client Listening Mode

Clients can choose two modes:

1. **Active scanning mode** - Periodically broadcast requests to get all services
2. **Passive listening mode** - Listen for announcement messages and update service list in real-time

```bash
# Active scanning mode (Python)
ai-scan --mode active --interval 30

# Passive listening mode (Python)
ai-scan --mode passive --watch-file services.json

# Rust version uses aiecho command
aiecho scan --output json
aiecho listen --output-file services.json --interval 30
```

---

## Installation

### Python Implementation

```bash
# Install from PyPI
pip install ai-discover

# or install from source
cd python
pip install -e .
```

### Rust Implementation

```bash
# Install from crates.io
cargo install aiecho

# or build from source
cd rust
cargo build --release
```

---

## Usage Examples

### Example 1: Server Configuration

Create `my_service.json`:

```json
{
  "service_name": "Image Generator",
  "service_id": "img-gen-001",
  "http_port": 3000,
  "manifest_path": "/ai_manifest",
  "tags": ["image", "ai", "generation"],
  "priority": 5
}
```

Start service agent:

```bash
ai-discover-agent --config my_service.json
```

### Example 2: Client Scanning (Python)

```python
from ai_discover import Scanner

scanner = Scanner()
services = scanner.scan(timeout=2.0)

for service in services:
    print(f"{service.name} @ {service.ip}:{service.port}")
    print(f"  Capabilities: {service.manifest.capabilities}")
```

### Example 2b: Client Scanning (Rust CLI)

```bash
# Scan local network, output JSON to stdout
aiecho scan --output json

# Scan and output in table format
aiecho scan --output table

# Scan and save to file
aiecho scan --output json --output-file services.json

# Scan without fetching manifest (faster)
aiecho scan --no-manifest

# Custom timeout
aiecho scan --timeout 5.0
```

### Example 3: Real-time Service Change Monitoring

```python
from ai_discover import Listener

def on_service_online(service):
    print(f"Service online: {service.name}")
    # Update services.json

def on_service_offline(service_id):
    print(f"Service offline: {service_id}")
    # Remove from services.json

listener = Listener()
listener.on("online", on_service_online)
listener.on("offline", on_service_offline)
listener.start()
```

### Example 3b: Real-time Monitoring (Rust CLI)

```bash
# Listen for service changes, save to file
ai-discover listen --output-file services.json

# Custom scan interval
ai-discover listen --output-file services.json --interval 60
```

---

## Configuration Reference

### Server Configuration

| Field                    | Type   | Required | Description                              |
| ----------------------- | ------ | ---- | --------------------------------- |
| `service_name`        | string | ✅   | Service display name                      |
| `service_id`          | string | ✅   | Unique service identifier                      |
| `http_port`           | int    | ✅   | HTTP service port                     |
| `manifest_path`       | string | ❌   | Manifest path (default: /ai_manifest) |
| `tags`                | array  | ❌   | Service tags                          |
| `priority`            | int    | ❌   | Priority (default: 1)                   |
| `announce_on_startup` | bool   | ❌   | Announce on startup (default: true)            |
| `health_check_url`    | string | ❌   | Health check URL                      |
| `description`         | string | ❌   | Service description                          |
| `version`             | string | ❌   | Service version                          |

### Configuration Examples

**Complete configuration example** (`service_config.json`):

```json
{
  "service_name": "PDF Converter Pro",
  "service_id": "pdf-converter-001",
  "http_port": 8080,
  "manifest_path": "/ai_manifest",
  "tags": ["pdf", "convert", "tool", "ai"],
  "priority": 10,
  "announce_on_startup": true,
  "health_check_url": "http://localhost:8080/health",
  "description": "PDF to Markdown tool, supports batch processing",
  "version": "1.2.0"
}
```

**Minimum configuration example**:

```json
{
  "service_name": "My Service",
  "service_id": "my-service-001",
  "http_port": 8000
}
```

**Multi-service configuration example** (`service_config_multi.json`):

```json
{
  "services": [
    {
      "service_name": "PDF Converter Pro",
      "service_id": "pdf-converter-001",
      "http_port": 8080,
      "manifest_path": "/ai_manifest",
      "tags": ["pdf", "convert", "tool", "ai"],
      "priority": 10,
      "announce_on_startup": true
    },
    {
      "service_name": "Text Generator AI",
      "service_id": "text-generator-001",
      "http_port": 8081,
      "manifest_path": "/ai_manifest",
      "tags": ["text", "generation", "ai", "nlp"],
      "priority": 8,
      "announce_on_startup": true
    },
    {
      "service_name": "Image Recognition AI",
      "service_id": "image-recognition-001",
      "http_port": 8082,
      "manifest_path": "/ai_manifest",
      "tags": ["image", "recognition", "ai", "computer-vision"],
      "priority": 9,
      "announce_on_startup": true
    }
  ]
}
```

### JSON Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AI-LAN Service Configuration",
  "description": "Configuration schema for AI-LAN service discovery agent",
  "type": "object",
  "required": [
    "service_name",
    "service_id",
    "http_port"
  ],
  "properties": {
    "service_name": {
      "type": "string",
      "description": "Service display name",
      "minLength": 1,
      "maxLength": 100
    },
    "service_id": {
      "type": "string",
      "description": "Unique service identifier",
      "pattern": "^[a-z0-9-]+$",
      "minLength": 3,
      "maxLength": 50
    },
    "http_port": {
      "type": "integer",
      "description": "HTTP service port",
      "minimum": 1,
      "maximum": 65535
    },
    "manifest_path": {
      "type": "string",
      "description": "Manifest path (default: /ai_manifest)",
      "default": "/ai_manifest",
      "pattern": "^/[a-zA-Z0-9_/-]+$"
    },
    "tags": {
      "type": "array",
      "description": "Service tags",
      "items": {
        "type": "string",
        "pattern": "^[a-z0-9-]+$"
      },
      "uniqueItems": true,
      "maxItems": 20
    },
    "priority": {
      "type": "integer",
      "description": "Priority (default: 1)",
      "default": 1,
      "minimum": 1,
      "maximum": 100
    },
    "announce_on_startup": {
      "type": "boolean",
      "description": "Announce on startup (default: true)",
      "default": true
    },
    "health_check_url": {
      "type": "string",
      "description": "Health check URL",
      "format": "uri"
    },
    "description": {
      "type": "string",
      "description": "Service description",
      "maxLength": 500
    },
    "version": {
      "type": "string",
      "description": "Service version",
      "pattern": "^\\d+\\.\\d+\\.\\d+$",
      "maxLength": 20
    }
  },
  "additionalProperties": false
}
```

### Client Configuration

| Field              | Type   | Default | Description          |
| ----------------- | ------ | ------ | ------------- |
| `udp_port`      | int    | 53535  | UDP listening port  |
| `timeout`       | float  | 2.0    | Scan timeout (seconds) |
| `output_format` | string | "json" | Output format      |
| `watch_mode`    | bool   | false  | Watch mode      |
| `output_file`   | string | null   | Output file      |

---

## FAQ

### Q: Do I need to configure mDNS on the router?

No. The system uses a custom UDP protocol and does not rely on mDNS. Just make sure the firewall allows UDP port 53535.

### Q: How to handle multi-network card environment?

The server will automatically bind to `0.0.0.0` and listen on all network cards. The client will collect responses from all network cards.

### Q: How to detect service offline?

The client will mark services that have timed out and not responded in the service list. You can get immediate notification by listening to the `AI_SERVICE_GOODBYE` message.

---

## Performance Benchmark

| Metric         | Value      |
| ------------ | --------- |
| Discovery latency     | < 10ms    |
| Single scan memory | < 1MB     |
| Concurrent support     | 100+ services |
| CPU usage     | < 1%      |

---

## Contribution Guide

Welcome to submit Issues and PRs! Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

### Development Environment Setup

```bash
# Python
cd python
poetry install
poetry run pytest

# Rust
cd rust
cargo test
```

---

## License

MIT License - see [LICENSE](LICENSE) file.

---

## Related Projects

- [zeroconf](https://github.com/jstasiak/zeroconf) - Python mDNS library
- [bonjour-service](https://github.com/watson/bonjour-service) - Node.js service discovery
