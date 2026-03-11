# AI-LAN 服务发现系统 - 设计文档

## 1. 系统架构

### 1.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          AI-LAN 服务发现系统                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────────────┐    ┌───────────────────────────────┐    │
│  │        客户端 (Scanner)       │    │       服务端 (Agent)           │    │
│  │  ┌────────────────────────┐  │    │  ┌────────────────────────┐   │    │
│  │  │   CLI Interface       │  │    │  │   Config Loader        │   │    │
│  │  │   ai-scan --output    │  │    │  │   (JSON)              │   │    │
│  │  └───────────┬────────────┘  │    │  └───────────┬────────────┘   │    │
│  │              │                │    │              │                │    │
│  │              ▼                │    │              ▼                │    │
│  │  ┌────────────────────────┐  │    │  ┌────────────────────────┐   │    │
│  │  │   UDP Discovery       │  │    │  │   UDP Listener        │   │    │
│  │  │   Scanner             │  │◄───│  │   (port 53535)        │   │    │
│  │  └───────────┬────────────┘  │    │  └───────────┬────────────┘   │    │
│  │              │                │    │              │                │    │
│  │              ▼                │    │              ▼                │    │
│  │  ┌────────────────────────┐  │    │  ┌────────────────────────┐   │    │
│  │  │   Manifest Fetcher     │  │    │  │   Response Builder     │   │    │
│  │  │   (HTTP Client)        │  │    │  │                        │   │    │
│  │  └───────────┬────────────┘  │    │  └───────────┬────────────┘   │    │
│  │              │                │    │              │                │    │
│  └──────────────┼────────────────┘    └──────────────┼────────────────┘    │
│                 │                                     │                      │
│                 │    UDP Broadcast (255.255.255.255) │                      │
│                 │    Port: 53535                     │                      │
│                 └─────────────────────────────────────┘                      │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                      HTTP Layer (Manifest API)                       │  │
│  │                                                                       │  │
│  │   GET /ai_manifest                                                   │  │
│  │   Response: { "meta": {...}, "capabilities": [...], "endpoints": {} } │  │
│  │                                                                       │  │
│  │   GET /health                                                        │  │
│  │   Response: { "status": "ok" }                                       │  │
│  │                                                                       │  │
│  │   POST /api/v1/invoke                                                │  │
│  │   Body: { "action": "xxx", "params": {...} }                        │  │
│  │                                                                       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 技术栈

| 层级 | 技术选型 | 版本 | 说明 |
|------|----------|------|------|
| **Python 实现** | | | |
| 运行时 | Python | >= 3.8 | 核心语言 |
| Web 框架 | FastAPI | >= 0.100 | 可选，用于示例服务 |
| 网络库 | asyncio | 内置 | 异步 I/O |
| CLI | Click | >= 8.0 | 命令行工具 |
| 数据验证 | Pydantic | >= 2.0 | 配置模型 |
| 输出美化 | Rich | >= 13.0 | 终端输出 |
| mDNS | zeroconf | >= 0.40 | 可选的 mDNS 注册 |
| HTTP 客户端 | aiohttp | >= 3.8 | 异步 HTTP 请求 |
| **Rust 实现** | | | |
| 运行时 | Rust | >= 1.70 | 核心语言 |
| Web 框架 | Axum | >= 0.6 | HTTP 服务 |
| 异步运行时 | Tokio | >= 1.0 | 异步运行时 |
| CLI | Clap | >= 4.0 | 命令行工具 |
| 序列化 | serde | >= 1.0 | JSON 序列化 |
| mDNS | mdns-sd | >= 0.5 | mDNS 注册 |

### 1.3 模块划分

#### Python 实现模块

| 模块名称 | 职责 | 依赖模块 |
|----------|------|----------|
| `protocol.py` | UDP 协议消息解析与构建 | - |
| `config.py` | 配置模型定义与验证 | Pydantic |
| `server.py` | 服务端发现代理 | protocol, config, zeroconf |
| `scanner.py` | 客户端扫描器 | protocol, config, aiohttp |
| `listener.py` | 实时监听器 | protocol, config, aiohttp |
| `cli.py` | CLI 入口点 | server, scanner, listener, Click, Rich |

#### Rust 实现模块

| 模块名称 | 职责 | 依赖模块 |
|----------|------|----------|
| `protocol` | UDP 协议消息解析与构建 | serde |
| `config` | 配置模型定义与验证 | serde, clap |
| `server` | 服务端发现代理 | protocol, config, tokio, mdns-sd |
| `scanner` | 客户端扫描器 | protocol, config, reqwest |
| `cli` | CLI 入口点 | server, scanner, clap |

---

## 2. 模块设计

### 2.1 协议模块设计

**核心类图**:

```
┌─────────────────────────────────────┐
│         Protocol Constants          │
├─────────────────────────────────────┤
│ + DISCOVERY_PORT: int = 53535      │
│ + DISCOVER_REQ: str = "AI_DISCOVER_REQ"
│ + DISCOVER_RES: str = "AI_DISCOVER_RES"
│ + SERVICE_ANNOUNCE: str = "AI_SERVICE_ANNOUNCE"
│ + SERVICE_GOODBYE: str = "AI_SERVICE_GOODBYE"
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│        parse_message(data: bytes)   │
├─────────────────────────────────────┤
│ + Returns: (command: str, payload: dict)
└─────────────────────────────────────┘
                │
                ├──────────────────────┐
                ▼                      ▼
┌─────────────────────────┐  ┌─────────────────────────┐
│ build_discover_req()   │  │ build_discover_res()   │
├─────────────────────────┤  ├─────────────────────────┤
│ Returns: bytes          │  │ Returns: bytes          │
└─────────────────────────┘  └─────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│         ServiceInfo                 │
├─────────────────────────────────────┤
│ - query_id: str                    │
│ - service_name: str                 │
│ - service_id: str                  │
│ - http_port: int                    │
│ - manifest_path: str                │
│ - tags: list[str]                   │
│ - priority: int                     │
│ - ip: str (运行时)                  │
├─────────────────────────────────────┤
│ + base_url: str                     │
│ + manifest_url: str                 │
│ + from_payload(): ServiceInfo       │
└─────────────────────────────────────┘
```

**时序图 - 服务发现流程**:

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│ Scanner │     │ Socket  │     │  Agent  │     │ Manifest│
└────┬────┘     └────┬────┘     └────┬────┘     └────┬────┘
     │               │               │               │
     │  1. scan()   │               │               │
     │──────────────>│               │               │
     │               │               │               │
     │  2. sendto   │               │               │
     │ (broadcast)  │               │               │
     │──────────────>│──────────────>│               │
     │               │               │               │
     │               │  3. recvfrom │               │
     │               │<─────────────│               │
     │               │               │               │
     │  4. Response  │               │               │
     │<──────────────│               │               │
     │               │               │               │
     │  5. GET /ai_manifest         │               │
     │─────────────────────────────────────────────>│
     │               │               │               │
     │  6. Manifest JSON            │               │
     │<─────────────────────────────────────────────│
     │               │               │               │
```

### 2.2 服务端设计

**类图**:

```
┌─────────────────────────────────────┐
│       DiscoveryServer               │
├─────────────────────────────────────┤
│ - config: ServiceConfig             │
│ - _running: bool                    │
│ - _thread: Thread                   │
│ - _socket: socket                  │
├─────────────────────────────────────┤
│ + start(): None                    │
│ + stop(): None                     │
│ + is_running(): bool               │
│ - _run(): None                    │
│ - _handle_message(): None          │
│ - _announce(): None                │
│ - _goodbye(): None                │
│ - _register_mdns(): None           │
└─────────────────────────────────────┘
```

**启动流程**:

```
Service Start
     │
     ▼
┌─────────────────┐
│ Load Config     │
│ (JSON file)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Create Server   │
│ instance        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ start()         │
│ - create thread │
│ - bind UDP      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│ _announce()     │     │ _register_mdns()│
│ (broadcast)     │     │ (optional)      │
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │
                     ▼
              Ready to serve
```

### 2.3 客户端设计

**类图 - Scanner**:

```
┌─────────────────────────────────────┐
│       DiscoveryScanner              │
├─────────────────────────────────────┤
│ - config: ClientConfig              │
│ - timeout: float                    │
├─────────────────────────────────────┤
│ + scan(): List[DiscoveredService]  │
│ - _broadcast_and_collect(): ...    │
│ - _fetch_manifests(): ...          │
│ - _fetch_manifest(): ...           │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│     DiscoveredService              │
├─────────────────────────────────────┤
│ - service_info: ServiceInfo        │
│ - manifest: Optional[dict]          │
│ - manifest_loaded: bool             │
├─────────────────────────────────────┤
│ + ip: str                          │
│ + port: int                        │
│ + name: str                        │
└─────────────────────────────────────┘
```

**类图 - Listener**:

```
┌─────────────────────────────────────┐
│      DiscoveryListener              │
├─────────────────────────────────────┤
│ - config: ClientConfig              │
│ - _running: bool                    │
│ - _services: Dict[str, ServiceState]│
│ - _callbacks: Dict                  │
├─────────────────────────────────────┤
│ + start(): None                    │
│ + stop(): None                     │
│ + on(): None                       │
│ + off(): None                      │
│ + get_services(): Dict             │
│ - _listen(): None                  │
│ - _handle_message(): None           │
│ - _handle_online(): None           │
│ - _handle_offline(): None          │
└─────────────────────────────────────┘
```

---

## 3. 数据模型

### 3.1 服务配置模型

**JSON 配置文件结构**:

```json
{
  "service_name": "PDF Converter Pro",
  "service_id": "pdf-converter-001",
  "http_port": 8080,
  "manifest_path": "/ai_manifest",
  "tags": ["pdf", "convert", "tool"],
  "priority": 10,
  "udp_port": 53535,
  "announce_on_startup": true
}
```

**配置字段说明**:

| 字段名 | 类型 | 必填 | 默认值 | 说明 |
|--------|------|------|--------|------|
| service_name | string | ✅ | - | 服务显示名称 |
| service_id | string | ✅ | - | 唯一服务标识 |
| http_port | integer | ✅ | - | HTTP 服务端口 (1-65535) |
| manifest_path | string | ❌ | "/ai_manifest" | Manifest 接口路径 |
| tags | array[string] | ❌ | [] | 服务标签 |
| priority | integer | ❌ | 1 | 优先级 (1-100) |
| udp_port | integer | ❌ | 53535 | UDP 监听端口 |
| announce_on_startup | boolean | ❌ | true | 启动时是否公告 |

### 3.2 Manifest 数据结构

```json
{
  "meta": {
    "service_id": "pdf-proc-001",
    "name": "Local PDF Processor",
    "version": "1.2.0",
    "description": "一个运行在局域网内的 PDF 处理工具",
    "author": "MyTeam",
    "uptime_seconds": 3600
  },
  "capabilities": [
    {
      "id": "convert_pdf_to_md",
      "name": "Convert PDF to Markdown",
      "description": "将 PDF 文件转换为 Markdown 文本",
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
    "health_check": "/health",
    "invoke": "/api/v1/invoke"
  },
  "auth": {
    "type": "bearer_token",
    "token_location": "header"
  }
}
```

---

## 4. 接口设计

### 4.1 UDP 协议

**网络参数**:

| 参数 | 值 | 说明 |
|------|------|------|
| Discovery Port | 53535 | UDP 监听端口 |
| Broadcast Address | 255.255.255.255 | 广播地址 |
| Encoding | UTF-8 | 载荷编码 |
| Timeout | 2.0s | 客户端等待超时 |

**消息格式**: `COMMAND\nJSON_PAYLOAD`

#### 发现请求 (Discovery Request)

```
AI_DISCOVER_REQ
{"query_id": "uuid-1234-5678", "version": "1.0"}
```

#### 发现响应 (Discovery Response)

```
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

#### 服务公告 (Service Announcement)

```
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

#### 服务告别 (Service Goodbye)

```
AI_SERVICE_GOODBYE
{
  "event": "offline",
  "service_id": "pdf-proc-001",
  "service_name": "PDF Processor"
}
```

### 4.2 HTTP API

#### 获取 Manifest

- **URL**: `GET /ai_manifest`
- **认证**: 无
- **响应 200**:

```json
{
  "meta": {...},
  "capabilities": [...],
  "endpoints": {...},
  "auth": {...}
}
```

#### 健康检查

- **URL**: `GET /health`
- **响应 200**:

```json
{
  "status": "ok",
  "service_id": "xxx",
  "uptime_seconds": 3600
}
```

#### 调用能力

- **URL**: `POST /api/v1/invoke`
- **请求体**:

```json
{
  "action": "convert_pdf_to_md",
  "params": {
    "file_path": "/path/to/file.pdf"
  }
}
```

- **响应 200**:

```json
{
  "status": "ok",
  "result": {
    "content": "# Markdown content..."
  }
}
```

### 4.3 CLI 接口

#### ai-discover-agent

```bash
ai-discover-agent --config service_config.json
```

| 参数 | 简写 | 说明 |
|------|------|------|
| --config | -c | 服务配置文件路径 (必填) |

#### ai-scan

```bash
ai-scan [OPTIONS]
```

| 参数 | 简写 | 默认值 | 说明 |
|------|------|--------|------|
| --output | -o | json | 输出格式: json, yaml, table |
| --timeout | -t | 2.0 | 扫描超时 (秒) |
| --no-manifest | - | false | 跳过获取 manifest |
| --output-file | -f | - | 输出文件路径 |

#### ai-scan watch

```bash
ai-scan watch --output-file services.json --interval 30
```

| 参数 | 简写 | 默认值 | 说明 |
|------|------|--------|------|
| --output-file | -f | - | 输出文件路径 (必填) |
| --interval | -i | 30 | 自动扫描间隔 (秒) |

---

## 5. 安全设计

### 5.1 网络隔离

- 服务发现仅在局域网内工作
- 默认不暴露到公网
- 建议在企业网络/家庭网络环境下使用

### 5.2 防火墙配置

| 端口 | 协议 | 方向 | 说明 |
|------|------|------|------|
| 53535 | UDP | 入站/出站 | 服务发现端口 |
| [http_port] | TCP | 入站 | HTTP 服务端口 |

### 5.3 认证方案

- 默认局域网内部发现不开启认证
- 可通过 Manifest 中的 auth 字段配置 bearer_token 或 api_key
- AI Agent 调用具体能力时需根据 auth 配置添加认证信息
