# 降级编码与注入解释契约（票据 #31）

> 目的：给「记降级」与「注入为什么这么做」两份跨票契约一个稳定的定义，让七张票据
> （#5、#7、#8、#10、#11、#12、#14）都能引用同一张编码表，也让评估能断言「发生了哪种降级」，
> 用户能区分「没记住」与「记住了没用上」。

## 1. 降级编码表

全仓唯一。取值一律小写 snake_case；各模块不得自造字符串。实现：`crates/core/src/degrade.rs`，
通过 `export_schema` 导出成 `DEGRADATION_CODES` / `DEGRADATION_MESSAGES` 供 Adapter 消费。

| 编码 | 含义 |
|---|---|
| `llm_not_configured` | 没配语言模型：只记录 Observation，不抽取（#8「记录但不理解」模式）。 |
| `role_unconfigured` | 某能力角色没有绑定 provider。 |
| `recall_timeout` | 召回超过预算，按无记忆继续（#11）。 |
| `extraction_failed` | 抽取失败或输出不合 schema（粗粒度；细分见抽取组）。 |
| `index_unavailable` | 向量/重排能力不可用，退回基线（#12）。 |
| `core_unreachable` | Adapter 连不上 Core（#7 的召回桩）。 |
| `outbound_blocked` | 目标不在白名单，出站被拒（#5）。 |
| `injection_deadline` | 注入超过了 Adapter 自建 deadline（#7、#14）。 |

抽取细分（#10 的失败分类，单独一组，仍是同一张表）：

| 编码 | 判定 |
|---|---|
| `extraction_timeout` | Core 自建 deadline 到期，非 provider 报错。 |
| `extraction_provider_error` | 连接 / 5xx / 429，带 statusCode。 |
| `extraction_invalid_json` | 解析失败（含被截断），带 line/column 与 finishReason。 |
| `extraction_schema_mismatch` | 合法 JSON 但结构不符，带 itemIndex/path。 |
| `evidence_not_found` | `quote` 不是来源 Observation 的逐字子串。 |
| `extraction_empty_response` | 内容为空或只有思考字段。 |
| `extraction_unavailable` | 抽取角色未配置或 fallback 链全失败，只记一次。 |

`extraction_empty`（合法空结果）**不是失败**，不在此表，单独记账。

### 纪律

- 编码全集有测试断言（`degrade.rs::code_table_matches_the_expected_full_set`）。新增编码
  必须同时改常量表与那张显式清单，否则测试红——这是「新增编码必须改表」的可执行形态。
- 每条降级在 Ledger 里带：编码、发生时间、尝试的用途、涉及的 provider/model（若适用）、
  可读原因。与 `LedgerKind` 的映射：降级一律记 `Degradation`（`degrade::ledger_kind`）。

## 2. 注入解释（R17 的字段契约）

`context-injection.md` R17 要求「Ledger 记为什么注入它」。契约落在
`protocol.rs::InjectionExplanation`，字段可直接建表：

```json
{
  "turn": 12,
  "sessionId": "s-2026-11-19-01",
  "items": [
    { "itemId": "01JC…", "score": 0.83, "path": "lexical", "tokenCount": 210 }
  ],
  "skipped": [
    { "itemId": "01JC…", "reason": "duplicate" }
  ],
  "tokenCount": 210,
  "budgetTokens": 1200,
  "degraded": ["recall_timeout"]
}
```

- `items`：本轮入选的条目，含 id、分数、路径（`lexical` / `semantic` / `rerank`）与 token 占用。
- `skipped`：被跳过的候选，`reason` 取 `INJECTION_SKIP_CODES`（`budget_exhausted` /
  `duplicate` / `low_score` / `abstained`）。跳过是正常行为，不是降级。
- `degraded`：本轮发生过的降级编码，正常轮为空数组。
- 记录本身落 `LedgerKind::Injection`。

字段口径来自：条目 token 数由 #29 的 Core 实现给出（Adapter 不自算）；分数/路径来自
#30 的召回分数模型；`degraded` 来自本表的编码。

## 3. 对标依据

- **Cognee 的「每段可独立失败重试」**：ECL 三阶段每段可单独失败重试，对应我们「抽取失败
  不拖累整理」「抽取与降级分开记账」的切分。
- **MemOS 的显式 fail-open 边界**：能力不可用时降级而不是整链失败，对应「未配能力→降级」
  与「不阻塞回合」。
- **GraphRAG 的 `derived_from`**：Ledger 溯源字段带 provider/model/用途，回答「这笔从哪来」。
- 详见 `docs/research/module-benchmarks.md`。

## 4. 验收映射

- 八个粗粒度编码 + 七个抽取细分编码都有常量与可读文案，且是一张常量表 → `degrade.rs`。
- 编码全集被测试断言，新增必须改表 → `code_table_matches_the_expected_full_set`。
- 注入解释字段契约明确到可直接建表 → `protocol.rs::InjectionExplanation`。
- 编码与 `LedgerKind` 映射写清 → `degrade::ledger_kind`（全记 `Degradation`）。
- 该场景已进入评估运行器 → 契约测试 `tests/contract/degrade-contract.ts` 按 Pass^k 跑。