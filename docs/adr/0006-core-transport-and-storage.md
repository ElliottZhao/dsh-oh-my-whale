# Core 的传输与持久化

Core 与 Adapter 之间走 **loopback HTTP + JSON**；Core 的长期状态放在**一个 SQLite 数据文件**里，用 `PRAGMA user_version` 记 schema 版本。

备选「自定义 TCP 分帧」少一层协议栈，但分帧、超时、错误映射都要自己写，也没有现成工具能手工调试一次握手；HTTP 让 `curl` 就能验一次契约，代价是每个请求多几个头部字节，对单用户本机通信无足轻重。备选「JSON 文件 + 原子替换」最接近「一个文件」的字面意思，但读写并发、崩溃一致性与按时间/来源的查询都要自己实现，而 SQLite 已经把这些做对了：单文件、事务、索引、迁移位。备选 gRPC 需要 protobuf 代码生成，与 ADR-0007 的「Rust 单一 schema 源」重复，也不比 HTTP 更适合本机单人通信。

## Consequences

- Core 只绑定 `127.0.0.1`，不提供关闭这一行为的开关；端口默认 7419，`--port 0` 交给系统分配。
- 端口经 stdout 的 `core.ready` 行公布，不写端口文件——数据目录因此只有一个数据文件，符合「单文件持久化」的验收。
- 数据目录里唯一的持久化产物是 `core.db`；SQLite 的回滚日志只在一次写入事务期间短暂存在。
- schema 版本由 `PRAGMA user_version` 承载，版本高于当前 Core 时拒绝启动并说明原因；迁移只加结构，旧数据改名归档而不是删除。
- 协议主版本号一致才允许沟通，次版本号差异不拒绝：新 Adapter 连旧 Core 只会少用新字段。
- 所有错误共用 `{"error":{"code","message","details"}}` 信封，包括请求体超限的 `413`；`message` 必须是完整可读的一句话，让「起不来」不需要翻日志才能解释。
- 本地校验失败（协议版本不符、响应畸形）没有对应的 HTTP 状态码，`CoreProtocolError.status` 因此是 `number | undefined`，不伪造 `200`。
