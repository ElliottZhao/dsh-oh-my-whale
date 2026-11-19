// 协议漂移守卫：证明「Core 加/删字段 → Adapter 编译失败」这条验收真的成立。
//
// 这里不复刻 schema，只检查 Adapter 是否仍在按生成的类型构造请求。
// 手工改 `protcol.rs` 后重跑 `pnpm run gen:protocol && pnpm run typecheck`，
// 这个文件会因类型不再匹配而编译失败。

import type {
  CreateObservationRequest,
  HandshakeRequest,
  ProviderConfig,
} from "../../packages/adapter/src/generated/protocol.generated.ts";

/** 若 Core 给握手请求加了必填字段，这个对象字面量会缺字段而编译失败。 */
export const handshakeRequestShape: HandshakeRequest = {
  protocolVersion: "1.0.0",
  adapterVersion: "0.1.0",
};

/** 若 Core 给 Observation 请求加了必填字段，这里同样会编译失败。 */
export const observationRequestShape: CreateObservationRequest = {
  sessionId: "s1",
  source: "host.message",
  speaker: "user",
  content: "……",
  occurredAt: "2026-01-01T00:00:00Z",
};

/** 若 Core 删掉某个字段，多写这一项会触发「对象字面量只能指定已知属性」。 */
export const configShape: ProviderConfig = {
  providers: [],
  roles: [],
};
