# AI-LAN 服务发现系统 - 开发文档

## 1. 开发环境

### 1.1 环境要求

| 环境 | 配置要求 |
|------|----------|
| 操作系统 | Windows 10+ / macOS / Linux |
| 内存 | >= 8GB RAM |
| 磁盘 | >= 10GB SSD |
| 网络 | 局域网环境 |

### 1.2 软件依赖

#### Python 开发环境

| 软件 | 版本 | 用途 |
|------|------|------|
| Python | >= 3.8 | 运行时 |
| pip | latest | 包管理 |
| Poetry | >= 1.0 | 依赖管理 |
| Git | >= 2.x | 版本控制 |

#### Rust 开发环境

| 软件 | 版本 | 用途 |
|------|------|------|
| Rust | >= 1.70 | 运行时 |
| Cargo | latest | 包管理 |
| Rustup | latest | Rust 工具链管理 |
| Git | >= 2.x | 版本控制 |

### 1.3 环境变量

```bash
# Python 开发环境 (.env.example)
PYTHONPATH=src
LOG_LEVEL=debug

# Rust 开发环境
RUST_LOG=debug
```

---

## 2. 编码规范

### 2.1 命名规范

| 类型 | 规则 | 示例 |
|------|------|------|
| 变量/函数 | camelCase | `get_service_info` |
| 常量 | UPPER_SNAKE_CASE | `DISCOVERY_PORT` |
| 类名 | PascalCase | `DiscoveryServer` |
| 文件名 | snake_case | `discovery_server.py` |
| 模块名 | snake_case | `discovery_scanner` |

### 2.2 代码风格

#### Python

- 使用 [Black](https://black.readthedocs.io/) 格式化代码
- 缩进: 4 空格
- 行尾: LF
- 字符串: 单引号优先
- 类型注解: 使用 Python 3.8+ 类型注解

```python
from typing import Optional, List

def discover_services(timeout: float = 2.0) -> List[ServiceInfo]:
    """Discover services on the network.
    
    Args:
        timeout: Discovery timeout in seconds.
        
    Returns:
        List of discovered services.
    """
    ...
```

#### Rust

- 使用 [rustfmt](https://rustformatter.com/) 格式化代码
- 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- 适当使用 `#[derive(...)]` 派生常用 trait

### 2.3 注释规范

#### Python

```python
"""Module docstring - brief description."""

class DiscoveryServer:
    """Class docstring."""
    
    def start(self) -> None:
        """Start the discovery server.
        
        Starts the server in a background thread and announces
        the service to the network if configured.
        
        Raises:
            RuntimeError: If server is already running.
        """
        ...
```

#### Rust

```rust
/// Module documentation.

/// Discovery server that responds to UDP broadcast queries.
pub struct DiscoveryServer {
    // Field documentation
    config: ServiceConfig,
}

impl DiscoveryServer {
    /// Start the discovery server.
    ///
    /// # Errors
    ///
    /// Returns an error if the server is already running.
    pub fn start(&mut self) -> Result<(), Error> {
        // Implementation
    }
}
```

---

## 3. 项目结构

### 3.1 Python 项目结构

```
python/
├── pyproject.toml              # 项目配置
├── poetry.lock                  # 依赖锁定
├── src/
│   └── ai_discover/
│       ├── __init__.py          # 包入口
│       ├── protocol.py          # UDP 协议实现
│       ├── config.py            # 配置模型
│       ├── server.py            # 服务端实现
│       ├── scanner.py           # 客户端扫描器
│       ├── listener.py          # 实时监听器
│       └── cli.py               # CLI 入口
├── tests/
│   ├── __init__.py
│   ├── test_protocol.py
│   ├── test_config.py
│   ├── test_server.py
│   ├── test_scanner.py
│   └── test_listener.py
└── examples/
    ├── service_config.json      # 示例配置
    └── manifest_example.json     # 示例 Manifest
```

### 3.2 Rust 项目结构

```
rust/
├── Cargo.toml                   # 项目配置
├── Cargo.lock                   # 依赖锁定
├── src/
│   ├── main.rs                  # 入口文件
│   ├── lib.rs                   # 库入口
│   ├── protocol/
│   │   ├── mod.rs
│   │   └── message.rs           # 消息类型
│   ├── config/
│   │   ├── mod.rs
│   │   └── model.rs             # 配置模型
│   ├── server/
│   │   ├── mod.rs
│   │   └── discovery.rs         # 服务端
│   ├── scanner/
│   │   ├── mod.rs
│   │   └── client.rs            # 客户端
│   └── cli/
│       ├── mod.rs
│       └── commands.rs          # CLI 命令
├── examples/
│   └── service_config.json
└── tests/
    ├── integration_test.rs
    └── protocol_test.rs
```

---

## 4. 模块开发指南

### 4.1 新增功能步骤

#### Python

1. **创建数据模型** (在 `config.py` 或新文件)
   - 使用 Pydantic 定义数据类
   - 添加字段验证

2. **实现核心逻辑**
   - 在对应模块添加实现
   - 遵循异步编程规范 (使用 asyncio)

3. **编写测试**
   - 在 `tests/` 目录添加测试文件
   - 使用 pytest 框架

4. **更新 CLI** (如需要)
   - 在 `cli.py` 添加命令

#### Rust

1. **创建数据模型** (在 `config/model.rs`)
   - 使用 serde 派生 Serialize/Deserialize

2. **实现核心逻辑**
   - 在对应模块添加实现
   - 使用 tokio 异步运行时

3. **编写测试**
   - 在 `tests/` 目录添加集成测试
   - 或在模块内添加单元测试

4. **更新 CLI** (如需要)
   - 在 `cli/commands.rs` 添加命令

### 4.2 错误处理

#### Python

```python
import logging

logger = logging.getLogger(__name__)

class DiscoveryError(Exception):
    """Base exception for discovery errors."""
    pass

class NetworkError(DiscoveryError):
    """Network related errors."""
    pass

class ConfigError(DiscoveryError):
    """Configuration related errors."""
    pass

# 使用示例
def scan():
    try:
        # scanning logic
        pass
    except socket.timeout:
        logger.warning("Discovery timed out")
        return []
    except Exception as e:
        logger.error(f"Discovery failed: {e}")
        raise DiscoveryError(f"Failed to discover: {e}") from e
```

#### Rust

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, DiscoveryError>;
```

### 4.3 日志记录

#### Python

```python
import logging

logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)

# 使用
logger.debug("Debug message")
logger.info("Service discovered: {name} @ {ip}:{port}", name=service.name, ip=ip, port=port)
logger.warning("No services found")
logger.error("Failed to fetch manifest: {error}", error=str(e))
```

#### Rust

```rust
use tracing::{info, warn, error, debug};

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    // 使用
    info!("Starting discovery server");
    debug!("Received discovery request from {}", addr);
    warn!("Service timeout: {}", service_id);
    error!("Failed to bind socket: {}", e);
}
```

---

## 5. 版本控制

### 5.1 分支策略

| 分支 | 用途 | 命名规则 |
|------|------|----------|
| main | 生产分支 | - |
| develop | 开发分支 | - |
| feature/* | 功能开发 | feature/功能名 |
| bugfix/* | Bug 修复 | bugfix/问题描述 |
| docs/* | 文档更新 | docs/文档名 |

### 5.2 提交规范

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Type 类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具
- `perf`: 性能优化

**示例**:

```
feat(server): add service announcement on startup

- Send AI_SERVICE_ANNOUNCE message when service starts
- Add graceful shutdown with goodbye message
- Support configurable announcement interval

Closes #123
```

### 5.3 PR 流程

1. 从 `develop` 创建功能分支
2. 开发完成后发起 PR
3. 至少一人 Code Review
4. CI/CD 检查通过后合并到 `develop`
5. 定期合并 `develop` 到 `main`

---

## 6. API 调试

### 6.1 常用命令

#### 测试 UDP 发现

```bash
# 使用 netcat 发送发现请求
echo -e "AI_DISCOVER_REQ\n{\"query_id\": \"test-123\", \"version\": \"1.0\"}" | nc -u -w1 255.255.255.255 53535
```

#### 测试 HTTP Manifest

```bash
# 获取 Manifest
curl -X GET http://192.168.1.50:8080/ai_manifest

# 健康检查
curl -X GET http://192.168.1.50:8080/health
```

### 6.2 网络调试工具

| 工具 | 用途 |
|------|------|
| Wireshark | UDP 包抓取分析 |
| netcat (nc) | UDP/TCP 测试 |
| tcpdump | 命令行抓包 |
| Postman/Insomnia | HTTP API 测试 |

---

## 7. 依赖管理

### 7.1 Python 依赖

#### 使用 Poetry

```bash
# 安装依赖
cd python
poetry install

# 添加依赖
poetry add aiohttp

# 添加开发依赖
poetry add --group pytest pytest-asyncio

# 构建发布包
poetry build
```

### 7.2 Rust 依赖

```bash
cd rust

# 构建项目
cargo build

# 添加依赖
cargo add tokio --features full
cargo add serde --features derive
cargo add reqwest --features json

# 运行测试
cargo test

# 发布
cargo publish
```
