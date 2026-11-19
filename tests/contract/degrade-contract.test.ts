/**
 * 接缝一：#31 降级编码与注入解释契约。
 *
 * 降级编码表是「全仓唯一」，它的跨端一致性必须在接缝层断言：Rust 侧导出的编码全集、
 * 每条的文案、跳过原因集合、注入解释字段，都要与 Adapter 生成的常量/类型对得上。
 * 每条按 Pass^k 重复。
 */
import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  DEGRADATION_CODES,
  DEGRADATION_MESSAGES,
  INJECTION_SKIP_CODES,
  PROTOCOL_SCHEMA,
} from "../../packages/adapter/src/index.ts";
import { PASS_K, passK } from "./helpers.ts";

/** Rust `degrade.rs` 里的显式全集；改表时必须同步这三处。 */
const EXPECTED_CODES = [
  "llm_not_configured",
  "role_unconfigured",
  "recall_timeout",
  "extraction_failed",
  "index_unavailable",
  "core_unreachable",
  "outbound_blocked",
  "injection_deadline",
  "extraction_timeout",
  "extraction_provider_error",
  "extraction_invalid_json",
  "extraction_schema_mismatch",
  "evidence_not_found",
  "extraction_empty_response",
  "extraction_unavailable",
];

const EXPECTED_SKIP = ["budget_exhausted", "duplicate", "low_score", "abstained"];

interface SchemaDefinition {
  oneOf?: Array<{ const?: string }>;
  properties?: Record<string, unknown>;
  required?: string[];
}

/** Schemars 对带文档注释的枚举输出 `oneOf: [{const}]`；兼容 `enum` 形态。 */
function enumValues(definition: SchemaDefinition | undefined, label: string): string[] {
  assert.ok(definition !== undefined, `schema 应包含 ${label}`);
  if (definition.oneOf !== undefined) {
    return definition.oneOf
      .map((variant) => variant.const)
      .filter((value): value is string => value !== undefined);
  }
  return [];
}

describe("降级编码契约（#31）", () => {
  it(`降级编码全集与 Rust 侧一致，且每一条都有文案（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      assert.deepEqual(
        [...DEGRADATION_CODES].sort(),
        [...EXPECTED_CODES].sort(),
        "生成出的降级编码集与 Rust 侧清单不一致",
      );
      for (const code of DEGRADATION_CODES) {
        assert.match(code, /^[a-z][a-z_]*$/, `编码 ${code} 只允许小写字母与下划线`);
        const message = DEGRADATION_MESSAGES[code];
        assert.ok(
          typeof message === "string" && message.length > 0,
          `编码 ${code} 缺少可读文案`,
        );
      }
    });
  });

  it(`注入跳过原因是稳定集合，且与降级编码不重叠（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      assert.deepEqual(
        [...INJECTION_SKIP_CODES].sort(),
        [...EXPECTED_SKIP].sort(),
        "注入跳过原因集合不一致",
      );
      for (const skip of INJECTION_SKIP_CODES) {
        assert.ok(
          !(DEGRADATION_CODES as readonly string[]).includes(skip),
          `${skip} 是跳过原因，不该混进降级编码`,
        );
      }
    });
  });

  it(`注入解释的 schema 带有可直接建表的字段（Pass^${PASS_K}）`, async () => {
    await passK(async () => {
      const definitions = (PROTOCOL_SCHEMA as { definitions?: Record<string, SchemaDefinition> })
        .definitions;
      assert.ok(definitions !== undefined, "生成的 schema 应带 definitions");

      const explanation = definitions.InjectionExplanation;
      assert.ok(explanation !== undefined, "schema 应包含 InjectionExplanation");
      assert.deepEqual(
        [...(explanation.required ?? [])].sort(),
        ["budgetTokens", "degraded", "items", "sessionId", "skipped", "tokenCount", "turn"],
        "注入解释的必填字段应覆盖 id、分数、路径、预算与降级编码",
      );

      const item = definitions.InjectedItem;
      assert.ok(item !== undefined, "schema 应包含 InjectedItem");
      assert.deepEqual(
        [...(item.required ?? [])].sort(),
        ["itemId", "path", "score", "tokenCount"],
        "入选条目应带 id、分数、路径与 token 占用",
      );

      const skipped = definitions.SkippedRecall;
      assert.ok(skipped !== undefined, "schema 应包含 SkippedRecall");
      assert.deepEqual([...(skipped.required ?? [])].sort(), ["itemId", "reason"]);

      const ledgerKind = definitions.LedgerKind;
      assert.deepEqual(
        enumValues(ledgerKind, "LedgerKind").sort(),
        ["change", "degradation", "injection", "outbound"],
        "LedgerKind 应覆盖出站、注入、降级、变更四类",
      );
    });
  });
});
