---
name: "aidis-service-discovery"
description: "Integrates with AI-LAN Service Discovery (aidis) protocol for service discovery and capability management. Invoke when Agent needs to discover local AI services or when implementing service registration for AI capabilities."
---

# aidis-service-discovery

## 何时使用
当Agent需要发现局域网内的AI服务，或需要注册新的AI能力作为可发现服务时使用此技能。

## 执行指示
1. **服务发现**：使用Scanner类扫描局域网内的AI服务
2. **服务注册**：创建配置文件并启动服务代理
3. **实时监控**：使用Listener类监听服务上线/下线事件
4. **能力过滤**：根据特定能力过滤服务
5. **manifest访问**：获取服务的详细能力描述

## 核心功能
- 发现局域网内的AI服务
- 注册新的AI能力
- 实时监控服务状态
- 访问服务manifest
- 按能力过滤服务

## 输出格式
返回服务发现结果、注册状态或监控事件，包括服务名称、ID、URL、标签和优先级等信息。

## 示例

### 示例1：发现服务
```python
from ai_discover import Scanner

scanner = Scanner()
services = scanner.scan(timeout=2.0)

for service in services:
    print(f"Service: {service.service_name}")
    print(f"  ID: {service.service_id}")
    print(f"  URL: {service.base_url}")
    print(f"  Tags: {service.tags}")
```

### 示例2：注册服务
配置文件 `service_config.json`：
```json
{
  "service_name": "PDF Processor",
  "service_id": "pdf-processor-001",
  "http_port": 8080,
  "manifest_path": "/ai_manifest",
  "tags": ["pdf", "convert", "tool"]
}
```

启动服务：
```bash
# 使用系统安装的命令
ai-discover-agent --config service_config.json

# 或使用技能目录中的二进制文件
.trae/skills/aidis-service-discovery/bin/aidis --config service_config.json
```

### 示例3：实时监控
```python
from ai_discover import Listener

def on_service_online(service):
    print(f"Service online: {service.service_name}")

def on_service_offline(service_id):
    print(f"Service offline: {service_id}")

listener = Listener()
listener.on("online", on_service_online)
listener.on("offline", on_service_offline)
listener.start()
```