# oh-my-whale

一个一对一 Personal Agent 的本机内核加宿主薄适配层。设计意图见 [`docs/spec/v1.md`](docs/spec/v1.md)，领域语言见 [`GLOSSARY.md`](GLOSSARY.md)，架构取舍见 [`docs/adr/`](docs/adr/)。

当前落地的是骨架与两端契约：Core 以单文件持久化启动、只监听 loopback、按 schema 写读 Observation、持有自己的 provider 配置；Adapter 作为独立包连上 Core 并完成带协议版本号的握手；两侧起不来时都给出可读错误。

## 目录

```
crates/core/        Core（Rust）：配置、单文件状态、Observation、provider、loopback HTTP
packages/adapter/   Adapter（TypeScript）：连接层、错误翻译、从 schema 生成的协议类型
tests/contract/     接缝一：Core 进程间契约的端到端测试
scripts/            协议类型生成脚本
docs/               规格、ADR、调研与本地规则
```

## 先决条件

- Rust 1.85+（edition 2024）
- Node.js 24+
- pnpm 10+

## 构建与测试

```sh
pnpm install            # 安装 TypeScript 工具链
pnpm run gen:protocol   # Rust 类型 → JSON Schema → Adapter 的 TypeScript 类型
pnpm run build:core     # 构建 target/debug/omw-core
pnpm run build:adapter
pnpm run typecheck       # 两端类型检查
pnpm test                # Rust 单元测试 + 类型检查 + 契约测试
```

`pnpm test` 会构建 Core，然后启动真实进程跑 `tests/contract/`，每个场景按 Pass^k 重复三次（`OMW_PASS_K` 可覆盖），整轮二十秒内结束。

## 跑起来

启动 Core：

```sh
target/debug/omw-core --data-dir ./.omw-data --config ./.omw-config/config.json --port 7419
```

数据目录里只会出现一个数据文件 `core.db`。Core 就绪后向 stdout 打印一行 JSON（日志走 stderr），Adapter 与测试脚本都从这一行拿端口：

```json
{"event":"core.ready","endpoint":"http://127.0.0.1:7419","dataFile":"…/core.db","pid":12345,"protocolVersion":"1.0.0","coreVersion":"0.1.0","schemaVersion":2,"startedAt":"2026-11-19T00:00:00Z"}
```

`--port 0` 让系统分配端口，适合并行测试。端口被占用、数据目录不可写、数据文件损坏或由更新版本的 Core 创建、provider 配置不合法，都会以可读错误退出（非零码）。

在代码里用：

```ts
import { CoreConnection } from "@oh-my-whale/adapter";

const core = new CoreConnection({ endpoint: "http://127.0.0.1:7419" });
const handshake = await core.handshake();

await core.recordObservation({
  sessionId: "s1",
  source: "host.message",
  speaker: "user",
  content: "我偏好简洁的回答",
});
const page = await core.observations({ sessionId: "s1" });
```

默认地址是 `http://127.0.0.1:7419`，可用 `OMW_CORE_ENDPOINT` 覆盖。连不上时抛 `CoreUnreachableError`，Core 拒绝时抛带 `code` 与可读 `message` 的 `CoreProtocolError`。

## 两端契约（Protocol 1.0.0）

- 传输：HTTP/JSON，只绑 `127.0.0.1`，不监听外网。
- `POST /v1/handshake`，请求 `{"protocolVersion":"1.0.0","adapterVersion":"0.1.0"}`。
  - 主版本号一致 → `200`，返回 `protocolVersion`、`coreVersion`、`schemaVersion`、`startedAt`。
  - 主版本号不同 → `409`，错误码 `protocol_version_mismatch`，`message` 里写明两侧版本与下一步。
  - 请求体不是合法 JSON → `400`，错误码 `invalid_request`。
- `POST /v1/observations` → `201`，返回落盘的 Observation；`occurredAt` 可带事件时刻（RFC3339），省略则用摄入时刻。`GET /v1/observations?sessionId=&limit=` 按会话读回最近的若干条。
- `GET /v1/config` → provider 配置快照（含 `unconfiguredRoles`、`degradedProviders` 与 `outboundWhitelist`）；`PUT /v1/config` 校验后写入，不合法返回 `400` 与 `invalid_config`。
- 所有错误共用一个信封：`{"error":{"code":"…","message":"…","details":{…}}}`；请求体超限（413）也走这个信封。`message` 是给人读的完整句子。
- 类型的唯一定义处是 `crates/core/src/protocol.rs`；`packages/adapter/src/generated/` 里的 TypeScript 类型与 schema 由它生成，不手改。`tests/contract/protocol-drift.ts` 守住「Core 加/删字段 → Adapter 编译失败」。
- 凭据此刻取不到的 provider 不阻止 Core 启动：按 ADR-0003 降级，在快照的 `degradedProviders` 里显式列出。

取舍理由见 [ADR-0006](docs/adr/0006-core-transport-and-storage.md) 与 [ADR-0007](docs/adr/0007-schema-single-source.md)。
