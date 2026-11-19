# 记忆引擎对标调研：MemOS / LongMemory / agent-memory / memvid / headroom

> 调研日期：2026-11-18。星标数、许可证、主语言均经 GitHub API（`api.github.com/repos/...`）核实；其余结论以各仓库 README / 架构文档原文为准。
> 场景：本项目（oh-my-whale）要选型/参考一个给 agent 用的长期记忆引擎，倾向本地优先、Node/TS 原生、无 Python 依赖。

---

## 1. MemTensor/MemOS（MemOS 2.0 "Stardust"）
- **一句话**：一个"记忆操作系统"，把 store/retrieve/manage 统一成单一图结构 API，主打超持久记忆、混合检索（FTS5+向量）与跨任务技能复用；自 2026-08 起官方支持 DeepSeek Harness。
- **核实**：Apache-2.0；TypeScript；★ 12,444 / fork 1,127（活跃，2026-11-18 仍有 push）；开放 issue 152。
- **存储与抽象**：图结构记忆（非黑盒 embedding 库），多模态（文本/图片/工具轨迹/人格）、多 Cube 知识库管理、异步摄取（MemScheduler，毫秒级延迟）、自然语言反馈纠错。本地插件数据落在 SQLite（官方文档 `~/.dsh/memos-plugin/data/memos.db`），四层记忆：L1 轨迹 / L2 策略 / L3 世界模型 / Skill 结晶，反射加权奖励反向传播把高价值模式结晶成可复用 Skill。
- **注入与上下文**：DSH 侧是**树外 Cordis bundle**（不修改官方仓库）。每个直连用户 turn 在 `agent/pre-step` 做一次有界自动召回（默认 3,000ms 硬上限，上下文封顶 6,000 字符），turn 结束时后台捕获；`plugin/memos-local-memory/recall` 来源标注，注入块明确标记为"不可信历史数据"而非指令。recall 后于 query 注入、带 `<memos_context>` 包裹。六个显式 `memos_*` 工具（search/get/timeline/environment/skill_list/skill_get）。辅助 LLM 复用 DSH 已配置的 provider/model/凭证，无需二次配 API key。Viewer 在 `http://127.0.0.1:18801`（仅 loopback，独立于 DSH 的 web 端口）。
- **token 节省说法的核实**：35.24% 是仓库描述与标题自述；在 OpenClaw 云插件处有"72% 更低 token 用量"的**营销声明**。README 未提供 35.24% 的可复现方法学（对比基线、场景、tokenizer）。实测基准是任务完成率与记忆评测（LoCoMo 88.83、LongMemEval 89.20），不是严格的 token 压缩基准。→ **结论：35.24% 属未佐证的营销数字，勿当作可靠事实引用。** 但它确有降 token 的机制（召回/技能复用替代重贴全文），方向可信，数字不可考。
- **开拓性思路（对比 Mem0/Letta/Graphiti）**：三级检索器（Skill → 轨迹/片段 → 世界模型）按需注入"对的内容到对的时机"，比 Mem0 的单层向量召回多一层"技能结晶"；比 Letta/Graphiti 的重记忆服务更本地优先、文件优先；L1/L2/L3 分层的"自我演化"是区别于纯 RAG 的核心。
- **可借鉴点**：① skill 结晶（把高价值模式沉淀为可调用技能）② 混合检索 FTS5+向量+RRF ③ 显式 fail-open 边界（召回/注入绝不拖慢主 agent）④ 单进程复用 DSH 的 LLM 路由、不重复配 key。**坑**：依赖 `better-sqlite3` 等原生/transitive 构建脚本，DSH 需逐个 `approve-builds`（pnpm 严格策略）；DSH 开发栈钉在 `0.2.1-rc.3` 预发布线，跨 DSH 版本兼容面窄；安装/更新需重启 DSH 进程。
- **可用性**：Node/TS 原生，符合"无 Python 依赖"；单用户本地优先可行（installer 一条命令装进 DSH `web` profile）。macOS/Linux 有 one-command installer；Windows 走底层 `dsh plugin` 流程。

---

## 2. CaviraOSS/LongMemory
- **一句话**：一个"耐用、时序、受管"的记忆引擎，明确自比"不只是 RAG、不只是向量库"，核心卖点是时序真值 + 不可变记忆 + 图语义 + 多级治理。
- **核实**：Apache-2.0；TypeScript；★ 4,585 / fork 514（活跃，2026-11-10 push）；开放 issue 仅 6。npm `longmemory` + PyPI `longmemory-sdk`（Python 侧是零依赖 HTTP client，引擎仍是 TS 服务）。
- **存储与抽象**：Hydrograph 底层，不可变节点 + 可执行边 + worlds/entities/facets/traces；recorded time 与 valid time 分离（时序真值）；显式生命周期（确定性衰减/强化/合并/压缩/再巩固）。本地持久用 SQLite，嵌入可用内存存储。治理覆盖 project/tenant/user/team/role/agent/task/framework 作用域。
- **生命周期与召回**：四种召回模式——strict（严格）、historical（历史/按 valid_time）、associative（关联）、world_grounded（世界接地）；召回被相关性/矛盾/接地/权限/token 成本共同约束。`answer_from_evidence` 适配器做证据接地的回答（最多一次 model 调用）。
- **开拓性思路（对比经典三件套）**：补上了 Mem0/Letta/Graphiti 多缺的"时间真值"与"治理"——什么在某个时刻为真、新事实是否覆盖旧事实、谁有权看、为什么进上下文，都显式建模。`valid_time` 回溯查询（"1 月的部署区域"）是独特能力。
- **可借鉴点**：① strict/historical/associative 分层召回 ② recorded/valid time 分离做超迭与历史 ③ 治理 ACL 生命周期。**坑**：功能面极广（dashboard/VS Code/n8n/MCP/CLI），选型成本高；定位偏"认知记忆引擎"而非轻量插件，做进单 agent 有复杂度过剩风险。
- **可用性**：TS 引擎原生，单用户本地 SQLite 开箱即用（`createMemory({store:'sqlite'})` 两行上手）；Python 只当瘦 client，不引入 Python 运行时到引擎，符合无 Python 依赖。是"项目记忆/架构决策记忆"的好参考。

---

## 3. tigerless-labs/agent-memory
- **一句话**：以纯 Markdown 为唯一真源、本地排序检索、独立"睡眠期 Manage 层"的记忆运行时；Claude Code / Codex / Hermes 共享一个 store，无需 API key。
- **核实**：MIT；Python（3.12+，需 `uv`）；★ 4,324 / fork 213（活跃，2026-11-12 push）；开放 issue 10。尚未发 PyPI，从 checkout `uv sync --all-packages`。
- **存储与抽象**：**Markdown 文件是真源**，旁边的 SQLite 索引（`.index/`）只是可删缓存（`rm -rf .index && mem rebuild` 零知识丢失，由测试保证）。一条记忆 = 一个 `.md`，frontmatter 带稳定名/一句话摘要/类型/时间戳/links/权重/provenance；正文自由 markdown。`valid_from`/`invalid_at` 定义有效区间，被替换/删除的文件仍留档供 `recall --as-of` 与 trace。
- **检索机制**：三条读轨（一次 miss 不是全 miss）——① 会话开始确定性注入 `MEMORY.md` ② FTS5 上跑 BM25 + 可选向量插件按 RRF 融合（默认 `BAAI/bge-small-en-v1.5`）③ 纯目录 `ls`/`grep`。召回给 L0 列表（一行摘要+路径+锚点+分数），agent 按需逐级展开（L0→outline→全文→原始材料），不把正文粘进上下文。same-directory 记忆是免费邻域，frontmatter `links` 承载图而无需图数据库。
- **睡眠期 Manage 层具体机制**（重点）：`mem sleep` 在**独立时钟**跑 T0/T1 两级合并——T0 直接应用，T1 只**提交提案**（deletion 永远只以提案出现，等你确认）。`mem proposals` 看待决项，`mem decide <id> --accept` 确认。写轨迹先整体复制到 `archive/sessions/`，"distiller 漏了"不等于"系统丢了"。`mem sleep --if-due` 按持久阈值跳过。可装 cron 定时唤醒（默认不装常驻进程）。Manage 推理借用 host CLI（Claude/Codex/Hermes/Muse），写时也借 host，故库内无 LLM client、零 key、零计费面。
- **实测**：LongMemEval-S（120 episodes，bounded haystack）52.9% vs 无记忆 5.8%；三条 host 两两互通 9 对读写全过。可选向量把 Recall@5 从 79.0%→86.6%（代价中位延迟 5.1ms→139.2ms）。
- **开拓性思路（对比 Mem0/Letta/Graphiti）**：用"文件系统"压过"黑盒检索"——agent 能直接 `ls`/`grep`/`git` 记忆；检索精确度借图+向量，但不失去可徒步的纯目录。Sleep 层是真正独立、按自己时钟跑的 Manage（多数竞品要么没有 M、要么埋在写路径里）。
- **可借鉴点**：① "索引=可重建缓存、文件=真源"的可迁移性 ② 按层展开召回省 context ③ 提案式删除（人确认）的治理 ④ 一个 store 多 host 共享。**坑**：Python 3.12 + uv 前置，**不满足"无 Python 依赖"**；CJK 记忆默认 tokenizer 只取 ASCII（中文查中文匹配不到），需 `[cjk]` extra（jieba）+ 中文向量模型——对中文项目是硬门槛；无 PyPI 发布。
- **可用性**：单用户本地优先优秀，但 Python 依赖是硬约束；中文需额外配置。

---

## 4. memvid/memvid
- **一句话**：把 RAG 管线替换成"无服务器、单文件记忆层"，全部打包进一个 `.mv2` 文件，即时检索 + 长期记忆。
- **核实**：Apache-2.0；Rust（核心 `memvid-core`）；★ 17,428 / fork 1,466（最后 push 2026-07-14，近数月偏静）。SDK 多语言：npm CLI / `@memvid/sdk`（Node）/ PyPI `memvid-sdk` / Rust crate。
- **可视化记忆是什么机制**：**Smart Frames（智能帧）**——借鉴视频编码，但存的是记忆而非视频。一个 Smart Frame 是不可变单元，存内容 + 时间戳 + 校验和 + 元数据；帧按组分，支持压缩/索引/并行读。因此：
  - 追加式写入，不改/不破坏已有数据；
  - 可查过去任意记忆状态（timeline 式回放）；
  - 崩溃安全（提交式不可变帧）；
  - 用视频编码那套压缩技巧省空间。
- **为什么能替代复杂 RAG**：传统 RAG = 分块→embed→存向量库→查相似；memvid 把数据+embedding+搜索结构+元数据全塞进一个文件，免去 server-based 向量库和 RAG 管线。内置 Tantivy 全文（BM25）、HNSW 向量、时间索引、TOC 页脚。宣称 P50 0.025ms / P99 0.075ms，LoCoMo 相对 SOTA +35%。
- **文件格式（.mv2）**：Header(4KB) + 内嵌 WAL(1–64MB) + 数据段 + Lex 索引(Tantivy) + Vec 索引(HNSW) + Time 索引 + TOC；**无任何 sidecar 文件**（`.wal`/`.lock`/`.shm`）。可选加密胶囊 `.mv2e`。feature flags 开闭：`lex`(全文)、`vec`(HNSW+本地 ONNX)、`clip`(图像)、`whisper`(音频转写)、`api_embed`(OpenAI)、`encryption` 等。
- **坑**：Rust 核心 + 本地 embedding 需手动下载 ONNX 模型（BGE 等，120MB+）；`vec` 用 ONNX Runtime；视频编码类比偏概念包装，实际价值=单文件便携 + 不可变帧时间线，而非真"视频"；仓库近 4 个月未 push，活跃度存疑；核心语言 Rust 与 TS/Node 优先的项目不完全对齐（虽有 `@memvid/sdk` Node 包）。
- **可借鉴点**：① 单文件自包含记忆（可携带/可版本化）② 不可变帧做时间线回溯 ③ 内嵌 WAL 崩溃安全。**可用性**：Node SDK 存在，但引擎在 Rust；适合"要一个便携记忆文件、不想跑向量库"的场景，不适合作为 TS 原生深度集成。

---

## 5. headroomlabs-ai/headroom（补强，另一份文档已见则只取结论）
- **一句话**：上下文压缩层，在内容到达 LLM 之前压缩工具输出/日志/文件/RAG 块，同答案更少 token；library / proxy / MCP 三形态。
- **核实**：Apache-2.0；Python + TypeScript；★ 79,965 / fork 6,269（**极活跃**，2026-11-17 仍在 push）。**双发布确认**：PyPI `headroom-ai`（含 CLI/proxy）+ npm `headroom-ai`（仅 TS SDK library，无 CLI）。
- **按内容类型分级压缩**：ContentRouter 检测内容类型并选压缩机——SmartCrusher(JSON)、CodeCompressor(AST，多语言)、Kompress-v2-base(prose，HF 模型)、图像压缩。CacheAligner 标记会击穿 provider KV-cache 前缀的易变内容（不重写 prompt）。CCR 可逆压缩：原件本地缓存，模型需全文本时调 `headroom_retrieve`。实测：编码 agent 场景 21–57% 输入 token 削减，重复 JSON 数组/日志行 90%+，散文几乎不压缩；压缩延迟 <1ms（0.21ms p50 @10K token）。支持 DeepSeek 等第三方 OpenAI 兼容上游（`--openai-api-url`）。
- **可直接依赖性**：Python `compress(messages)` 进程内、无 key、`HEADROOM_OFFLINE=1` 零网络；TS `compress()` 走本机 proxy。这是最容易被现有 agent 项目直接引用的"通用上下文压缩"中间层。
- **注意**：匿名遥测**默认开启**（`HEADROOM_BEACON=off` 可关）；ONNX 特征需 AVX2（无则回退 BM25/heuristic）；这是**压缩层**而非记忆存储层，与上面四个是不同轴——可与记忆引擎叠用。

---

## 横向结论（针对本项目）

| 维度 | MemOS | LongMemory | agent-memory | memvid | headroom |
|---|---|---|---|---|---|
| 许可证 | Apache-2.0 | Apache-2.0 | MIT | Apache-2.0 | Apache-2.0 |
| 主语言 | TS | TS | **Python** | Rust | Python+TS |
| ★(核实) | 12.4k | 4.6k | 4.3k | 17.4k | 80k |
| 活跃(2026-11) | ✅ 活跃 | ✅ 活跃 | ✅ 活跃 | ⚠️ 近静 | ✅ 极活跃 |
| 本地优先 | ✅ SQLite | ✅ SQLite | ✅ Markdown+SQLite | ✅ 单文件 | ✅ 进程内 |
| TS 原生/无 Python 依赖 | ✅ | ✅ | ❌ 需 Python+uv | ⚠️ 核心 Rust | ⚠️ CLI 需 Python，TS 是 library |
| 记忆→agent 集成 | ✅ 官方 DSH bundle | ⚠️ MCP/插件多 host | ⚠️ Claude/Codex/Hermes(非 DSH) | ⚠️ SDK | ❌ 压缩非记忆 |

- **最贴合"TS 原生、本地优先、无 Python 依赖、深度集成 agent"的是 MemOS**：官方 DSH Cordis bundle，单进程复用 DSH LLM 路由，技能结晶 + L1/L2/L3 分层是区别于经典 Mem0/Letta/Graphiti 的最大增量；但它有 `approve-builds` 原生依赖与 DSH 版本钉死两个运维坑，且 35.24% token 数字不可采信。
- **若更看重"可审计、可迁移、纯文件真源 + 提案式治理"**，agent-memory 思路最正（文件=真源、索引=缓存、sleep 独立 Manage 层），但 Python 依赖 + CJK 需额外 extra 是中文项目硬门槛。
- **LongMemory** 的时序真值（valid_time 回溯）与治理是最深的语义模型，适合"项目/架构决策记忆"，复杂度偏高。
- **memvid** 的"单文件便携记忆 + 不可变帧时间线"是差异化卖点，但 Rust 核心 + 活跃度存疑。
- **headroom 是正交的压缩层**，与任一记忆引擎可叠用；最易直接依赖（Python 进程内 / TS 走 proxy），是省 token 的即插即用选项。