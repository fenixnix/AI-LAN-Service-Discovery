# AI-LAN Service Discovery (aidis)

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

## 概述

AI-LAN 服务发现系统是一套轻量级、零配置、高性能的局域网 AI 微服务发现机制。该系统使 AI Agent 能够动态发现并调用局域网内部署的各种 AI 工具服务（如 PDF 处理、图像生成、知识库检索等）。

### 核心特性

- 🚀 **毫秒级发现** - UDP 广播机制，毫秒级响应
- 🔄 **实时感知** - 服务上线自动公告，客户端动态更新服务列表
- 📦 **零代码接入** - 仅需 JSON 配置文件即可完成服务注册
- 🌐 **多语言支持** - Python 和 Rust 原生实现
- 🔌 **标准化接口** - RESTful API + JSON Schema 能力描述

---

## 系统架构

### 核心组件

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

### 两阶段发现协议

| 阶段                    | 协议                    | 用途                 | 延迟  |
| ----------------------- | ----------------------- | -------------------- | ----- |
| **Discovery**     | UDP Broadcast (53535)   | 快速定位服务 IP/端口 | <10ms |
| **Introspection** | HTTP GET (/ai_manifest) | 获取完整能力描述     | <50ms |

---

## 快速开始

### 1. 服务端 (Service Provider)

**仅需要一个 JSON 配置文件：**

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

**启动命令：**

```bash
# Python
pip install ai-discover
ai-discover-agent --config service_config.json

# or Rust (cargo)
cargo install aidis
aidis --config service_config.json
```

### 2. 客户端 (AI Scanner)

**一行命令获取所有服务：**

```bash
# 安装
pip install ai-discover

# Python 版本
ai-scan --output json
ai-scan --output table

# Rust 版本
aidis scan --output json
aidis scan --output table
```

**输出示例：**

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

## 协议规范

### UDP 发现协议

| 参数                     | 值            | 说明            |
| ------------------------ | ------------- | --------------- |
| **Discovery Port** | `53535`     | UDP 监听端口    |
| **Transport**      | UDP Broadcast | 255.255.255.255 |
| **Encoding**       | UTF-8         | 所有载荷编码    |
| **Timeout**        | `2.0s`      | 客户端等待超时  |

#### 发现请求 (Discovery Request)

```text
AI_DISCOVER_REQ
{"query_id": "uuid-1234-5678", "version": "1.0"}
```

#### 发现响应 (Discovery Response)

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

### HTTP Manifest 接口

**端点:** `GET /ai_manifest`

**响应:**

```json
{
  "meta": {
    "service_id": "pdf-proc-001",
    "name": "Local PDF Processor",
    "version": "1.2.0",
    "description": "PDF 转 Markdown 工具",
    "author": "MyTeam"
  },
  "capabilities": [
    {
      "id": "convert_pdf_to_md",
      "name": "Convert PDF to Markdown",
      "description": "将 PDF 文件转换为 Markdown",
      "input_schema": {
        "type": "object",
        "properties": {
          "file_path": {
            "type": "string",
            "description": "PDF 文件路径"
          }
        },
        "required": ["file_path"]
      },
      "output_schema": {
        "type": "object",
        "properties": {
          "content": {
            "type": "string",
            "description": "Markdown 内容"
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

## 新服务上线公告机制

### 核心设计

当服务端上线时，会**主动广播**公告消息，客户端可以监听并实时更新本地服务列表。

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
│   Wait for                                              │
│   discovery requests                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 公告消息格式

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

### 客户端监听模式

客户端可以选择两种模式：

1. **主动扫描模式** - 定时广播请求，获取所有服务
2. **被动监听模式** - 监听公告消息，实时更新服务列表

```bash
# 主动扫描模式 (Python)
ai-scan --mode active --interval 30

# 被动监听模式 (Python)
ai-scan --mode passive --watch-file services.json

# Rust 版本使用 aidis 命令
aidis scan --output json
aidis listen --output-file services.json --interval 30
```

---

## 安装

### Python 实现

```bash
# 从 PyPI 安装
pip install ai-discover

# 或从源码安装
cd python
pip install -e .
```

### Rust 实现

```bash
# 从 crates.io 安装
cargo install aidis

# 或从源码构建
cd rust
cargo build --release
```

---

## 使用示例

### 示例 1: 服务端配置

创建 `my_service.json`:

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

启动服务代理:

```bash
ai-discover-agent --config my_service.json
```

### 示例 2: 客户端扫描 (Python)

```python
from ai_discover import Scanner

scanner = Scanner()
services = scanner.scan(timeout=2.0)

for service in services:
    print(f"{service.name} @ {service.ip}:{service.port}")
    print(f"  Capabilities: {service.manifest.capabilities}")
```

### 示例 2b: 客户端扫描 (Rust CLI)

```bash
# 扫描局域网，输出 JSON 到 stdout
aidis scan --output json

# 扫描并输出表格形式
aidis scan --output table

# 扫描并保存到文件
aidis scan --output json --output-file services.json

# 扫描不获取 manifest（更快）
aidis scan --no-manifest

# 自定义超时
aidis scan --timeout 5.0
```

### 示例 3: 实时监听服务变化

```python
from ai_discover import Listener

def on_service_online(service):
    print(f"Service online: {service.name}")
    # 更新 services.json

def on_service_offline(service_id):
    print(f"Service offline: {service_id}")
    # 从 services.json 移除

listener = Listener()
listener.on("online", on_service_online)
listener.on("offline", on_service_offline)
listener.start()
```

### 示例 3b: 实时监听 (Rust CLI)

```bash
# 监听服务变化，保存到文件
ai-discover listen --output-file services.json

# 自定义扫描间隔
ai-discover listen --output-file services.json --interval 60
```

---

## 配置参考

### 服务端配置

| 字段                    | 类型   | 必填 | 说明                              |
| ----------------------- | ------ | ---- | --------------------------------- |
| `service_name`        | string | ✅   | 服务显示名称                      |
| `service_id`          | string | ✅   | 唯一服务标识                      |
| `http_port`           | int    | ✅   | HTTP 服务端口                     |
| `manifest_path`       | string | ❌   | Manifest 路径 (默认 /ai_manifest) |
| `tags`                | array  | ❌   | 服务标签                          |
| `priority`            | int    | ❌   | 优先级 (默认 1)                   |
| `announce_on_startup` | bool   | ❌   | 启动时公告 (默认 true)            |
| `health_check_url`    | string | ❌   | 健康检查 URL                      |
| `description`         | string | ❌   | 服务描述                          |
| `version`             | string | ❌   | 服务版本                          |

### 配置示例

**完整配置示例** (`service_config.json`):

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
  "description": "PDF 转 Markdown 工具，支持批量处理",
  "version": "1.2.0"
}
```

**最小配置示例**:

```json
{
  "service_name": "My Service",
  "service_id": "my-service-001",
  "http_port": 8000
}
```

**多服务配置示例** (`service_config_multi.json`):

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

### 客户端配置

| 字段              | 类型   | 默认值 | 说明          |
| ----------------- | ------ | ------ | ------------- |
| `udp_port`      | int    | 53535  | UDP 监听端口  |
| `timeout`       | float  | 2.0    | 扫描超时 (秒) |
| `output_format` | string | "json" | 输出格式      |
| `watch_mode`    | bool   | false  | 监听模式      |
| `output_file`   | string | null   | 输出文件      |

---

## 常见问题

### Q: 需要在路由器上配置 mDNS 吗？

不需要。系统使用自定义 UDP 协议，不依赖 mDNS。只需要确保防火墙允许 UDP 53535 端口。

### Q: 如何处理多网卡环境？

服务端会自动绑定 `0.0.0.0`，监听所有网卡。客户端会收集所有网卡的响应。

### Q: 服务下线如何感知？

客户端会在服务列表中标记超时未响应的服务。可以通过监听 `AI_SERVICE_GOODBYE` 消息获得即时通知。

---

## 性能基准

| 指标         | 数值      |
| ------------ | --------- |
| 发现延迟     | < 10ms    |
| 单次扫描内存 | < 1MB     |
| 并发支持     | 100+ 服务 |
| CPU 占用     | < 1%      |

---

## 贡献指南

欢迎提交 Issue 和 PR！请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

### 开发环境设置

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

## 许可证

MIT License - 查看 [LICENSE](LICENSE) 文件。

---

## 相关项目

- [zeroconf](https://github.com/jstasiak/zeroconf) - Python mDNS 库
- [bonjour-service](https://github.com/watson/bonjour-service) - Node.js 服务发现
