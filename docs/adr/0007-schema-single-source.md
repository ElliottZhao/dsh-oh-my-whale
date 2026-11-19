# 领域 schema 单一来源：Rust 定义，TypeScript 生成

协议类型只在 `crates/core/src/protocol.rs` 定义一次。Rust 侧用 `schemars` 导出 JSON Schema，Node 侧据此生成 TypeScript 类型与运行时校验器：`cargo run --bin export_schema` → `scripts/generate-protocol-types.mjs` → `packages/adapter/src/generated/`。

备选「两端各写一份类型」最省事，但漂移是必然结果——改了一侧忘了另一侧，直到运行期才炸。备选「用 TypeScript 生成 Rust」把权威方向反过来，可 Core 持有存储与迁移，字段变更的起点在它这里。备选 gRPC/protobuf 在 ADR-0006 里已因「引入第二套类型来源」被否。

## Consequences

- Adapter 不手写字段守卫：运行时校验读的是同一份 JSON Schema（ajv），schema 变了校验自动跟着变。
- 生成物随源码一起提交，构建不需要先跑 Rust；`pnpm run gen:protocol` 只在改了 `protocol.rs` 之后执行，`pnpm run check:protocol` 在测试里守住生成物不落后于源码。
- 生成文件带「请勿手改」的头部注释，改类型只能改 Rust，这是有意的摩擦。
- 协议版本常量 `PROTOCOL_VERSION` 也只从 Rust 生成，两端不再各写一份。
- 类型校验只在边界发生：Adapter 收响应时按 schema 校验，校验通过后内部信任该类型。
- `tests/contract/protocol-drift.ts` 按生成的请求类型构造对象字面量：Core 给请求加必填字段或删字段时，这里直接编译失败并指到具体字段（已实测）。
