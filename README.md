# AIEcho

## Language / 语言

- [English](README.en.md)
- [中文](README.zh-CN.md)

<p align="center">
  <a href="https://github.com/fenixnix/AIEcho/releases">
    <img src="https://img.shields.io/github/v/release/fenixnix/AIEcho?include_prereleases&label=release" alt="Release"/>
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

***

## Overview

AIEcho is a lightweight, zero-configuration, high-performance LAN AI microservice discovery mechanism. This system enables AI Agents to dynamically discover and call various AI tool services deployed on the local network (such as PDF processing, image generation, knowledge base retrieval, etc.).

### Core Features

- 🚀 **Millisecond Discovery** - UDP broadcast mechanism, millisecond response
- 🔄 **Real-time Awareness** - Service online auto-announcement, client dynamically updates service list
- 📦 **Zero-code Access** - Only need JSON configuration file to complete service registration
- 🌐 **Multi-language Support** - Python and Rust native implementation
- 🔌 **Standardized Interface** - RESTful API + JSON Schema capability description

***

## Quick Start

### Server Side (Service Provider)

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

````bash
# Python
pip install ai-discover
ai-discover-agent --config service_config.json

```bash
# or Rust (cargo)
cargo install aiecho
aiecho --config service_config.json
````

### Client Side (AI Scanner)

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

***

## Documentation

For more detailed documentation, please refer to the language-specific README files:

- [English Documentation](README.en.md)
- [中文文档](README.zh-CN.md)

***

## License

MIT License - see [LICENSE](LICENSE) file.
