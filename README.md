# kiro-rs

一个用 Rust 编写的 Anthropic Claude API 兼容代理服务，将 Anthropic API 请求转换为 Kiro API 请求。

## 免责声明
本项目仅供研究使用, Use at your own risk, 使用本项目所导致的任何后果由使用人承担, 与本项目无关。
本项目与 AWS/KIRO/Anthropic/Claude 等官方无关, 本项目不代表官方立场。

## 注意！
因 TLS 默认从 native-tls 切换至 rustls，你可能需要专门安装证书后才能配置 HTTP 代理。可通过 `config.json` 的 `tlsBackend` 切回 `native-tls`。
如果遇到请求报错, 尤其是无法刷新 token, 或者是直接返回 error request, 请尝试切换 tls 后端为 `native-tls`, 一般即可解决。

**Write Failed/会话卡死**: 如果遇到持续的 Write File / Write Failed 并导致会话不可用，参考 Issue [#22](https://github.com/hank9999/kiro.rs/issues/22) 和 [#49](https://github.com/hank9999/kiro.rs/issues/49) 的说明与临时解决方案（通常与输出过长被截断有关，可尝试调低输出相关 token 上限）

## 功能特性

- **Anthropic API 兼容**: 完整支持 Anthropic Claude API 格式
- **流式响应**: 支持 SSE (Server-Sent Events) 流式输出
- **Token 自动刷新**: 自动管理和刷新 OAuth Token
- **多凭据支持**: 支持配置多个凭据，按优先级自动故障转移
- **智能重试**: 单凭据最多重试 3 次，单请求最多重试 9 次
- **凭据回写**: 多凭据格式下自动回写刷新后的 Token
- **Thinking 模式**: 支持 Claude 的 extended thinking 功能
- **工具调用**: 完整支持 function calling / tool use
- **多模型支持**: 支持 Sonnet、Opus、Haiku 系列模型

## 支持的 API 端点

### 标准端点 (/v1)

| 端点 | 方法 | 描述          |
|------|------|-------------|
| `/v1/models` | GET | 获取可用模型列表    |
| `/v1/messages` | POST | 创建消息（对话）    |
| `/v1/messages/count_tokens` | POST | 估算 Token 数量 |

### Claude Code 兼容端点 (/cc/v1)

| 端点 | 方法 | 描述          |
|------|------|-------------|
| `/cc/v1/messages` | POST | 创建消息（流式响应会等待上游完成后再返回，确保 `input_tokens` 准确） |
| `/cc/v1/messages/count_tokens` | POST | 估算 Token 数量（与 `/v1` 相同） |

> **`/cc/v1/messages` 与 `/v1/messages` 的区别**：
> - `/v1/messages`：实时流式返回，`message_start` 中的 `input_tokens` 是估算值
> - `/cc/v1/messages`：缓冲模式，等待上游流完成后，用从 `contextUsageEvent` 计算的准确 `input_tokens` 更正 `message_start`，然后一次性返回所有事件
> - 等待期间会每 25 秒发送 `ping` 事件保活

## 快速开始

> **前置步骤**：编译前需要先构建前端 Admin UI（用于嵌入到二进制中）：
> ```bash
> cd admin-ui && pnpm install && pnpm build
> ```

### 1. 编译项目

```bash
cargo build --release
```

### 2. 配置文件

创建 `config.json` 配置文件：

```json
{
   "host": "127.0.0.1",   // 必配, 监听地址
   "port": 8990,  // 必配, 监听端口
   "apiKey": "sk-kiro-rs-qazWSXedcRFV123456",  // 必配, 请求的鉴权 token
   "region": "us-east-1",  // 必配, 区域, 一般保持默认即可
   "tlsBackend": "rustls", // 可选, TLS 后端: rustls / native-tls
   "kiroVersion": "0.8.0",  // 可选, 用于自定义请求特征, 不需要请删除: kiro ide 版本
   "machineId": "如果你需要自定义机器码请将64位机器码填到这里", // 可选, 用于自定义请求特征, 不需要请删除: 机器码
   "systemVersion": "darwin#24.6.0",  // 可选, 用于自定义请求特征, 不需要请删除: 系统版本
   "nodeVersion": "22.21.1",  // 可选, 用于自定义请求特征, 不需要请删除: node 版本
   "countTokensApiUrl": "https://api.example.com/v1/messages/count_tokens", // 可选, 用于自定义token统计API, 不需要请删除
   "countTokensApiKey": "sk-your-count-tokens-api-key",  // 可选, 用于自定义token统计API, 不需要请删除
   "countTokensAuthType": "x-api-key",  // 可选, 用于自定义token统计API, 不需要请删除
   "proxyUrl": "http://127.0.0.1:7890", // 可选, HTTP/SOCK5代理, 不需要请删除
   "proxyUsername": "user",  // 可选, HTTP/SOCK5代理用户名, 不需要请删除
   "proxyPassword": "pass",  // 可选, HTTP/SOCK5代理密码, 不需要请删除
   "adminApiKey": "sk-admin-your-secret-key"  // 可选, Admin API 密钥, 用于启用凭据管理 API, 填写后才会启用web管理， 不需要请删除
}
```
最小启动配置为: 
```json
{
   "host": "127.0.0.1",
   "port": 8990,
   "apiKey": "sk-kiro-rs-qazWSXedcRFV123456",
   "region": "us-east-1",
   "tlsBackend": "rustls"
}
```
### 3. 凭证文件

创建 `credentials.json` 凭证文件（从 Kiro IDE 获取）。支持两种格式：

#### 单凭据格式（旧格式，向后兼容）

```json
{
   "accessToken": "这里是请求token 一般有效期一小时",  // 可选, 不需要请删除, 可以自动刷新
   "refreshToken": "这里是刷新token 一般有效期7-30天不等",  // 必配, 根据实际填写
   "profileArn": "这是profileArn, 如果没有请你删除该字段， 配置应该像这个 arn:aws:codewhisperer:us-east-1:111112222233:profile/QWER1QAZSDFGH",  // 可选, 不需要请删除
   "expiresAt": "这里是请求token过期时间, 一般格式是这样2025-12-31T02:32:45.144Z, 在过期前 kirors 不会请求刷新请求token",  // 必配, 不确定你需要写一个已经过期的UTC时间
   "authMethod": "这里是认证方式 social / idc",  // 必配, IdC/Builder-ID/IAM 三类用户统一填写 idc
   "clientId": "如果你是 IdC 登录 需要配置这个",  // 可选, 不需要请删除
   "clientSecret": "如果你是 IdC 登录 需要配置这个"  // 可选, 不需要请删除
}
```

#### 多凭据格式（新格式，支持故障转移和自动回写）

```json
[
   {
      "refreshToken": "第一个凭据的刷新token",
      "expiresAt": "2025-12-31T02:32:45.144Z",
      "authMethod": "social",
      "priority": 0
   },
   {
      "refreshToken": "第二个凭据的刷新token",
      "expiresAt": "2025-12-31T02:32:45.144Z",
      "authMethod": "idc",
      "clientId": "xxxxxxxxx",
      "clientSecret": "xxxxxxxxx",
      "region": "us-east-2",
      "priority": 1
   }
]
```

> **多凭据特性说明**：
> - 按 `priority` 字段排序，数字越小优先级越高（默认为 0）
> - 单凭据最多重试 3 次，单请求最多重试 9 次
> - 自动故障转移到下一个可用凭据
> - 多凭据格式下 Token 刷新后自动回写到源文件
> - 可选的 `region` 字段：用于 OIDC token 刷新时指定 endpoint 区域，未配置时回退到 config.json 的 region
> - 可选的 `machineId` 字段：凭据级机器码；未配置时回退到 config.json 的 machineId；都未配置时由 refreshToken 派生

最小启动配置(social):
```json
{
   "refreshToken": "XXXXXXXXXXXXXXXX",
   "expiresAt": "2025-12-31T02:32:45.144Z",
   "authMethod": "social"
}
```

最小启动配置(idc):
```json
{
   "refreshToken": "XXXXXXXXXXXXXXXX",
   "expiresAt": "2025-12-31T02:32:45.144Z",
   "authMethod": "idc",
   "clientId": "xxxxxxxxx",
   "clientSecret": "xxxxxxxxx"
}
```
### 4. 启动服务

```bash
./target/release/kiro-rs
```

或指定配置文件路径：

```bash
./target/release/kiro-rs -c /path/to/config.json --credentials /path/to/credentials.json
```

### 5. 使用 API

```bash
curl http://127.0.0.1:8990/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-your-custom-api-key" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "max_tokens": 1024,
    "messages": [
      {"role": "user", "content": "Hello, Claude!"}
    ]
  }'
```

## 配置说明

### config.json

| 字段 | 类型 | 默认值 | 描述                      |
|------|------|--------|-------------------------|
| `host` | string | `127.0.0.1` | 服务监听地址                  |
| `port` | number | `8080` | 服务监听端口                  |
| `apiKey` | string | - | 自定义 API Key（用于客户端认证，必配） |
| `region` | string | `us-east-1` | AWS 区域                  |
| `kiroVersion` | string | `0.8.0` | Kiro 版本号                |
| `machineId` | string | - | 自定义机器码（64位十六进制）不定义则自动生成 |
| `systemVersion` | string | 随机 | 系统版本标识                  |
| `nodeVersion` | string | `22.21.1` | Node.js 版本标识            |
| `tlsBackend` | string | `rustls` | TLS 后端：`rustls` 或 `native-tls` |
| `countTokensApiUrl` | string | - | 外部 count_tokens API 地址（可选） |
| `countTokensApiKey` | string | - | 外部 count_tokens API 密钥（可选） |
| `countTokensAuthType` | string | `x-api-key` | 外部 API 认证类型：`x-api-key` 或 `bearer` |
| `proxyUrl` | string | - | HTTP/SOCKS5 代理地址（可选） |
| `proxyUsername` | string | - | 代理用户名（可选） |
| `proxyPassword` | string | - | 代理密码（可选） |
| `adminApiKey` | string | - | Admin API 密钥，配置后启用凭据管理 API, 填写后才会启用web管理（可选） |

### credentials.json

支持单对象格式（向后兼容）或数组格式（多凭据）。

| 字段 | 类型 | 描述                      |
|------|------|-------------------------|
| `id` | number | 凭据唯一 ID（可选，仅用于 Admin API 管理；手写文件可不填） |
| `accessToken` | string | OAuth 访问令牌（可选，可自动刷新）    |
| `refreshToken` | string | OAuth 刷新令牌              |
| `profileArn` | string | AWS Profile ARN（可选，登录时返回） |
| `expiresAt` | string | Token 过期时间 (RFC3339)    |
| `authMethod` | string | 认证方式（`social` / `idc`） |
| `clientId` | string | IdC 登录的客户端 ID（可选）      |
| `clientSecret` | string | IdC 登录的客户端密钥（可选）      |
| `priority` | number | 凭据优先级，数字越小越优先，默认为 0（多凭据格式时有效）|
| `region` | string | 凭据级 region（可选），用于 OIDC token 刷新时指定 endpoint 的区域。未配置时回退到 config.json 的 region。注意：API 调用始终使用 config.json 的 region |
| `machineId` | string | 凭据级机器码（可选，64位十六进制）。未配置时回退到 config.json 的 machineId；都未配置时由 refreshToken 派生 |

说明：
- IdC / Builder-ID / IAM 在本项目里属于同一种登录方式，配置时统一使用 `authMethod: "idc"`
- 为兼容旧配置，`builder-id` / `iam` 仍可被识别，但会按 `idc` 处理

## 模型映射

| Anthropic 模型 | Kiro 模型 |
|----------------|-----------|
| `*sonnet*` | `claude-sonnet-4.5` |
| `*opus*` | `claude-opus-4.5` |
| `*haiku*` | `claude-haiku-4.5` |

## 项目结构

```
kiro-rs/
├── src/
│   ├── main.rs                 # 程序入口
│   ├── model/                  # 配置和参数模型
│   │   ├── config.rs           # 应用配置（支持环境变量覆盖）
│   │   └── arg.rs              # 命令行参数
│   ├── anthropic/              # Anthropic API 兼容层
│   │   ├── router.rs           # 路由配置
│   │   ├── handlers.rs         # 请求处理器
│   │   ├── middleware.rs       # 认证中间件
│   │   ├── types.rs            # 类型定义
│   │   ├── converter.rs        # 协议转换器
│   │   ├── stream.rs           # 流式响应处理
│   │   └── token.rs            # Token 估算
│   ├── admin/                  # Admin API
│   │   ├── router.rs           # Admin 路由
│   │   ├── handlers.rs         # Admin 处理器
│   │   ├── service.rs          # 业务逻辑
│   │   ├── types.rs            # Admin 类型定义
│   │   ├── middleware.rs       # Admin 认证
│   │   └── error.rs            # 错误处理
│   ├── db/                     # 数据库存储（可选）
│   │   ├── mod.rs              # 模块入口
│   │   ├── store.rs            # CredentialStore trait
│   │   └── pg.rs               # PostgreSQL 实现
│   └── kiro/                   # Kiro API 客户端
│       ├── provider.rs         # API 提供者
│       ├── token_manager.rs    # Token 管理
│       ├── machine_id.rs       # 设备指纹生成
│       ├── model/              # 数据模型
│       │   ├── credentials.rs  # OAuth 凭证
│       │   ├── events/         # 响应事件类型
│       │   ├── requests/       # 请求类型
│       │   └── common/         # 共享类型
│       └── parser/             # AWS Event Stream 解析器
│           ├── decoder.rs      # 流式解码器
│           ├── frame.rs        # 帧解析
│           ├── header.rs       # 头部解析
│           └── crc.rs          # CRC 校验
├── admin-ui/                   # Admin UI 前端（React + Vite）
│   ├── src/
│   │   ├── components/         # UI 组件
│   │   ├── hooks/              # React Hooks
│   │   ├── api/                # API 调用
│   │   └── types/              # TypeScript 类型
│   └── dist/                   # 构建产物（嵌入二进制）
├── Cargo.toml                  # 项目配置
├── config.example.json         # 配置示例
├── tools/                      # 辅助工具
└── Dockerfile                  # Docker 构建文件
```

## 技术栈

- **Web 框架**: [Axum](https://github.com/tokio-rs/axum) 0.8
- **异步运行时**: [Tokio](https://tokio.rs/)
- **HTTP 客户端**: [Reqwest](https://github.com/seanmonstar/reqwest)
- **序列化**: [Serde](https://serde.rs/)
- **日志**: [tracing](https://github.com/tokio-rs/tracing)
- **命令行**: [Clap](https://github.com/clap-rs/clap)

## 高级功能

### Thinking 模式

支持 Claude 的 extended thinking 功能：

```json
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 16000,
  "thinking": {
    "type": "enabled",
    "budget_tokens": 10000
  },
  "messages": [...]
}
```

### 工具调用

完整支持 Anthropic 的 tool use 功能：

```json
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 1024,
  "tools": [
    {
      "name": "get_weather",
      "description": "获取指定城市的天气",
      "input_schema": {
        "type": "object",
        "properties": {
          "city": {"type": "string"}
        },
        "required": ["city"]
      }
    }
  ],
  "messages": [...]
}
```

### 流式响应

设置 `stream: true` 启用 SSE 流式响应：

```json
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 1024,
  "stream": true,
  "messages": [...]
}
```

## 认证方式

支持两种 API Key 认证方式：

1. **x-api-key Header**
   ```
   x-api-key: sk-your-api-key
   ```

2. **Authorization Bearer**
   ```
   Authorization: Bearer sk-your-api-key
   ```

## 环境变量

### 日志级别

```bash
RUST_LOG=debug ./target/release/kiro-rs
```

### 配置覆盖

所有 `config.json` 配置项均可通过 `KIRO_` 前缀的环境变量覆盖（环境变量优先级高于配置文件）：

| 环境变量 | 对应配置项 | 说明 |
|---------|-----------|------|
| `KIRO_HOST` | `host` | 监听地址 |
| `KIRO_PORT` | `port` | 监听端口 |
| `KIRO_REGION` | `region` | AWS 区域 |
| `KIRO_API_KEY` | `apiKey` | API 认证密钥 |
| `KIRO_ADMIN_API_KEY` | `adminApiKey` | Admin API 密钥 |
| `KIRO_VERSION` | `kiroVersion` | Kiro 版本号 |
| `KIRO_MACHINE_ID` | `machineId` | 机器码 |
| `KIRO_SYSTEM_VERSION` | `systemVersion` | 系统版本 |
| `KIRO_NODE_VERSION` | `nodeVersion` | Node 版本 |
| `KIRO_TLS_BACKEND` | `tlsBackend` | TLS 后端 |
| `KIRO_PROXY_URL` | `proxyUrl` | 代理地址 |
| `KIRO_PROXY_USERNAME` | `proxyUsername` | 代理用户名 |
| `KIRO_PROXY_PASSWORD` | `proxyPassword` | 代理密码 |
| `KIRO_COUNT_TOKENS_API_URL` | `countTokensApiUrl` | Token 统计 API |
| `KIRO_COUNT_TOKENS_API_KEY` | `countTokensApiKey` | Token 统计 API 密钥 |
| `KIRO_COUNT_TOKENS_AUTH_TYPE` | `countTokensAuthType` | Token 统计认证类型 |
| `KIRO_DATABASE_URL` | `databaseUrl` | PostgreSQL 连接地址 |
| `DATABASE_URL` | `databaseUrl` | PostgreSQL 连接地址（兼容通用变量名） |

示例：
```bash
KIRO_HOST=0.0.0.0 KIRO_PORT=3000 KIRO_API_KEY=sk-xxx ./kiro-rs
```

## 注意事项

1. **凭证安全**: 请妥善保管 `credentials.json` 文件，不要提交到版本控制
2. **Token 刷新**: 服务会自动刷新过期的 Token，无需手动干预
3. **WebSearch 工具**: 当 `tools` 列表仅包含一个 `web_search` 工具时，会走内置 WebSearch 转换逻辑

## Admin（可选）

当 `config.json` 配置了非空 `adminApiKey` 时，会启用：

### Admin API

所有端点需要认证（`x-api-key` 或 `Authorization: Bearer`）。

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/admin/credentials` | GET | 获取凭据列表（分页：`?page=1&pageSize=20`） |
| `/api/admin/credentials` | POST | 添加新凭据 |
| `/api/admin/credentials/:id` | DELETE | 删除凭据 |
| `/api/admin/credentials/:id/disabled` | POST | 设置凭据禁用状态 |
| `/api/admin/credentials/:id/priority` | POST | 设置凭据优先级 |
| `/api/admin/credentials/:id/reset` | POST | 重置失败计数 |
| `/api/admin/credentials/:id/balance` | GET | 获取凭据余额 |
| `/api/admin/credentials/batch-import` | POST | 批量导入凭据 |
| `/api/admin/credentials/batch-delete` | POST | 批量删除凭据 |
| `/api/admin/credentials/export` | GET | 导出凭据（`?format=json` 或 `?format=csv`） |

### Admin UI

访问 `GET /admin` 打开管理页面（需要在编译前构建 `admin-ui/dist`）。

**功能：**
- 凭据列表（分页浏览）
- 添加/删除/禁用凭据
- 设置优先级、重置失败计数
- 查看凭据余额
- 批量选择与删除
- 导入 JSON/CSV 文件
- 导出为 JSON/CSV

## 云平台部署

### Zeabur 部署

1. **Fork 或导入仓库**
   - 在 [Zeabur](https://zeabur.com) 创建项目
   - 选择 Git 部署，导入本仓库

2. **配置环境变量**

   在 Zeabur 控制台的「Variables」中添加（大部分有默认值，可选配置）：

   | 变量名 | 默认值 | 说明 |
   |--------|--------|------|
   | `KIRO_HOST` | `0.0.0.0` | 监听地址（默认已适配容器） |
   | `KIRO_PORT` | `8990` | 监听端口 |
   | `KIRO_API_KEY` | `sk-kiro-rs-default-key` | API 密钥（建议修改） |
   | `KIRO_REGION` | `us-east-1` | AWS 区域 |
   | `KIRO_ADMIN_API_KEY` | 无 | Admin API 密钥（推荐配置） |
   | `KIRO_TLS_BACKEND` | `rustls` | TLS 后端 |

   > **最小配置**：如果只是测试，可以不配置任何环境变量，直接部署即可启动。建议至少配置 `KIRO_API_KEY` 和 `KIRO_ADMIN_API_KEY`。

3. **添加凭据**

   部署成功后，通过 Admin API 添加凭据：

   ```bash
   # 添加单个凭据
   curl -X POST https://your-app.zeabur.app/api/admin/credentials \
     -H "Content-Type: application/json" \
     -H "x-api-key: YOUR_ADMIN_API_KEY" \
     -d '{
       "refreshToken": "YOUR_REFRESH_TOKEN",
       "authMethod": "social"
     }'

   # 或批量导入
   curl -X POST https://your-app.zeabur.app/api/admin/credentials/batch-import \
     -H "Content-Type: application/json" \
     -H "x-api-key: YOUR_ADMIN_API_KEY" \
     -d '{
       "credentials": [
         {"refreshToken": "TOKEN1", "authMethod": "social"},
         {"refreshToken": "TOKEN2", "authMethod": "idc", "clientId": "xxx", "clientSecret": "xxx"}
       ]
     }'
   ```

4. **访问服务**
   - API: `https://your-app.zeabur.app/v1/messages`
   - Admin UI: `https://your-app.zeabur.app/admin`

### Docker 部署

```bash
# 构建镜像
docker build -t kiro-rs .

# 方式一：配置文件模式
docker run -d -p 8990:8990 \
  -v /path/to/config:/app/config \
  kiro-rs \
  ./kiro-rs -c /app/config/config.json --credentials /app/config/credentials.json

# 方式二：环境变量模式（配合 Admin API 添加凭据）
docker run -d -p 8990:8990 \
  -e KIRO_HOST=0.0.0.0 \
  -e KIRO_PORT=8990 \
  -e KIRO_API_KEY=sk-your-api-key \
  -e KIRO_REGION=us-east-1 \
  -e KIRO_ADMIN_API_KEY=sk-admin-key \
  kiro-rs
```

### Docker Compose

```yaml
version: '3.8'
services:
  kiro-rs:
    build: .
    ports:
      - "8990:8990"
    environment:
      - KIRO_HOST=0.0.0.0
      - KIRO_PORT=8990
      - KIRO_API_KEY=sk-your-api-key
      - KIRO_REGION=us-east-1
      - KIRO_ADMIN_API_KEY=sk-admin-key
    restart: unless-stopped
```

## License

MIT

## 致谢

本项目的实现离不开前辈的努力:  
 - [kiro2api](https://github.com/caidaoli/kiro2api)
 - [proxycast](https://github.com/aiclientproxy/proxycast)

本项目部分逻辑参考了以上的项目, 再次由衷的感谢!
