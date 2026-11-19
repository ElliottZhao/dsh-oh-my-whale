# StartLux Decision API（Bamboo Relay）事实摘要

> 来源：<https://bamboonode.dev/docs/ai-model/decisions/startlux>，经 defuddle 抽取正文核实，2026-11-18。仅记录页面写明的事实；未写明的一律标「未确认」。

## 它是什么

- 模型：`startlux-decision-27b-autothink`，由 StartLux Labs 训练，Bamboo Relay 提供 API 访问（27B，AutoThink）。
- 端点：`POST https://api.bamboonode.dev/v1/decisions`；**别名** `POST https://api.bamboonode.dev/v1/systemone`。
- 定位：对**具名问题**在给定 `state` 下给出选择、分数与概率——是结构化决策，不是自由文本生成。

## 请求形态

| 字段 | 说明 |
|---|---|
| `state` | 必填。共享输入，文本或 JSON；是被判断的整体情境 |
| `questions` | 必填、非空对象。键为问题 id，响应沿用同一 id |
| `think_mode` | `auto`（默认，首个答案不确定时思考）/ `never` / `always` |
| `images` | 可选，最多 8 个 base64 或 data URI |
| `model` | 可省略，默认即该模型 |

问题类型三选一：

- `choice`：给出 `criteria` 映射（候选项 → 说明），返回选中项。
- `noul`：**none-of-the-above**，返回「以上皆非」的概率。
- `score`：给出 `criteria` 数组作为刻度锚点，返回分数。

## 响应形态

每个问题按 id 返回：`choice` + `confidence` + `none_probability` + `probabilities`（choice）；`noul` + 概率（noul）；`score` + `probabilities`（score）；另返回 `usage` 的 token 数（即使计费为零也返回）。

## 定价与准入

- 当前测试期**免费**（含零余额新账号），需有效的 Bamboo API key 且可访问 `default` 组。
- 「测试期免费」是有时限的商务状态，不是长期承诺。

## 对本项目的设计含义

1. **它属于 Decision 能力族，不属于 CrossEncoder**。返回的是带概率的类型化答案，天然可审计、可写进 Ledger；自由文本排序不是它的形状。
2. **`noul` 是我们一直缺的「弃权」信号**。长期记忆基准（LongMemEval）专门有一项 Abstention 能力——「不确定就拒答」。有了 `noul`，「这条记忆值不值得召回 / 我是否该回答」可以从阈值猜测升级为校准概率。
3. **可作为「在 N 个候选中选一个」式的排序**：`choice` 的 `criteria` 可放候选 id，`probabilities` 即分布。但它给的是**相对分布**，不是细粒度 top-k 次序。
4. **热路径风险**：27B + 可选思考，远端调用。作为召回排序器会与「召回 p95 ≤ 800ms」直接冲突，因此默认不进召回热路径。
5. **它是一次出境调用**：按出境白名单的规则，必须登记为 Core 配置里的一个 provider。

## 未确认

- 实测延迟（尤其 `think_mode: auto` 命中思考分支时）；页面未给 SLA 或超时约定。
- 免费期结束后是否仍可免额使用。
- 并发上限与速率限制。
