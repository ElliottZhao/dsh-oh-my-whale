# 对标项目候选清单（2026-11-18 检索快照）

> 检索来源：GitHub Search API（高星 + 活跃排序）与 OpenSERP（Google）。★ 数与活跃度为检索当日值，仅用于初筛。
> 本清单是种子；逐项目深挖见 `agent-memory-systems.md`（经典记忆系统）、`module-benchmarks.md`（经典逐模块对标）、`frontier-projects.md`（本清单新发现项目）。

## 模块 × 候选

### 记忆（含存储抽象层）

经典：Cognee、Mem0、Letta/MemGPT、Graphiti(Zep)、A-MEM、MemTensor/MemOS、MIRIX、Memobase、Supermemory、LangMem、HippoRAG、Generative Agents

新发现（活跃、高星）：

| 项目 | ★ | 语言 | 一句话 |
|---|---|---|---|
| thedotmack/claude-mem | 118k | TypeScript | 跨会话持久上下文，捕获一切 |
| vectorize-io/hindsight | 62k | Python | "会学习的" agent 记忆 |
| volcengine/OpenViking | 42k | Python | 自进化上下文数据库：统一 agent 记忆 + 知识 RAG |
| TencentCloud/TencentDB-Agent-Memory | 31k | TypeScript | 团队级记忆枢纽 |
| MemoriLabs/Memori | 18k | Python | agent 原生记忆基础设施，LLM 无关层 |
| memvid/memvid | 17k | Rust | 用"可视化记忆"替代复杂 RAG 管线 |
| NevaMind-AI/memU | 15k | Python | Personal memory across agents |
| EverMind-AI/EverOS | 14k | Python | 本地优先、Markdown 原生的便携记忆层 |
| CaviraOSS/LongMemory | 4.6k | TypeScript | LLM 应用本地持久记忆 |
| FlowElement…/m_flow | 4.5k | Python | 生物启发认知记忆引擎（Graph RAG 新范式） |
| tigerless-labs/agent-memory | 4.3k | Python | 以纯 Markdown 为唯一真源的记忆运行时 |
| MemMachine/MemMachine | 3.1k | Python | 通用可扩展记忆层 |
| BAI-LAB/MemoryOS | 1.6k | Python | EMNLP 2025 Oral，记忆操作系统 |
| salishforge/memforge | — | — | 神经科学启发记忆 |
| loganionian/0gmem | — | — | 长对话记忆 |

### 用户画像

经典：Memobase（user-profile-based memory）、Honcho、Letta core-memory、LangMem profiles
新发现：zhayujie/CowAgent（48k★，个人助理，"先理解你"）、OpenBiliClaw（3.8k★，"先理解你，再主动"）

### 自我模型（自省）

经典：Voyager、Reflexion、ExpeL、Agent Workflow Memory
新发现：facebookresearch/HyperAgents（2.9k★，自指自我改进）、letta-ai/letta-code（3.9k★，TS）、metaauto-ai/GPTSwarm（1.1k★）

### 上下文装配

经典：MemGPT、Letta context management、Zep context API、LLMLingua
新发现：headroomlabs-ai/headroom（80k★，工具输出/RAG 块进入上下文前压缩）、gsd-build/get-shit-done（64k★）与 gsd-2（7.7k★，TS）、volcengine/MineContext（5.6k★，主动上下文伙伴）、Kaelio/ktx（1.6k★，TS）

### 策略仲裁

经典：AutoGen speaker selection、CrewAI hierarchical、LangGraph supervisor、Semantic Router
新发现：ruvnet/ruflo（82k★，multi-player swarm harness）——更偏 harness，仲裁面参考价值待验证

### 感知

经典：Generative Agents observation stream、Cognee ECL、LlamaIndex ingestion、Unstructured
新发现：OpenBiliClaw（外部内容信号源）、DeusData/codebase-memory-mcp（48.8k★，代码侧感知）

### 主动性 / 目标引擎

经典：AutoGPT、BabyAGI、Voyager curriculum、ProactiveAgent、OpenHands
新发现：MineContext（主动上下文伙伴）、CopilotKit/OpenDuet（2.1k★，主动 agent 模板，TS）、proma-ai/Proma（2.2k★）、simplexai-labs/linger（451★，本机个人 agent，TS）、TencentCloud/Octop（12.2k★）

### 情感与关系

经典：SillyTavern、ESConv、EmpatheticDialogues
新发现：moeru-ai/airi（52k★，TS，"container of souls"自托管伴侣）、Project-N-E-K.O/N.E.K.O（3.3k★）、Pal-AI-Lab/Coopanion（3.4k★，TS）、heshengtao/super-agent-party（2.8k★，JS）

### 世界模型

经典：microsoft/graphrag（37k★）、HKUDS/LightRAG（41k★）、KAG(OpenSPG)、Graphiti、HippoRAG
新发现：deeplethe/utopia（8.7k★，企业世界模型）、m_flow

### 持续学习

经典：Voyager、ExpeL、DSPy、TextGrad
新发现：Human-Agent-Society/reef（7.8k★，持续自我改进基础设施）、PrimeIntellect-ai/prime-agent（22.8k★，Rust）、aiming-lab/MetaClaw（3.5k★，"talk to it, it learns and evolves"）、AMAP-ML/SkillClaw（2.8k★，技能集体进化）、letta-code

### 权限边界 / 评估（横切）

经典：Open Interpreter、HumanLayer、E2B；LoCoMo、LongMemEval、MemBench、τ-bench、RAGAS
新发现：VibeBench/VibeSearchBench（107★，主动式检索 benchmark）、HybridAIOne/hybridclaw（163★，沙箱执行 runtime）

## 检索方法备忘

- `gh search repos <query> --sort=stars` 与 `gh api search/repositories?q=topic:<topic>&sort=stars`（topic：long-term-memory、context-engineering、proactive-agent、ai-companion、world-models、continual-learning、agent-orchestration、personalization）。
- OpenSERP：google 引擎，`--limit 5-6 --format markdown`；伴侣/主动性两个查询被反爬挡空，第三个查询命中 memforge/0gmem。

## P0 首轮手动核实（2026-11-18）

### volcengine/OpenViking（42.4k★，Python，**AGPL-3.0**）

- 定位：**面向 AI agent 的上下文数据库**——用一套虚拟文件系统（`viking://`）装下 agent 知道的一切：知识、记忆、技能。
- 核心机制：把上下文组织成可浏览的目录树，agent 用 `ls/tree/read/write/grep` 操作；每个目录带自动生成的摘要；语义检索可**限定子树**（"搜一个目录而不是整个索引"）。
- 三层加载：**L0 摘要**（一句话，判相关性）→ **L1 概览**（规划用）→ **L2 详情**（按需读全文）。文件形态：`.abstract.md`（L0）/ `.overview.md`（L1）/ 源文件（L2）。
- **与 DSH 的关系**：官方文档有 DSH 集成页（`docs.openviking.ai/en/agent-integrations/17-dsh`，标注 "Plugin + MCP"）——即它已经提供了 DSH 插件形态。这是本清单里与我们的运行时**最贴脸**的对标。
- 对我们的意义：①L0/L1/L2 分层是 Context Composer 很好的参考（先摘要、后详情）；②"记忆 = 文件系统 + 子树检索"是一个与"扁平向量池"相反的方向；③**AGPL-3.0 意味着只可借鉴思想与接口，不可拷代码**（否则我们的仓库会被传染为 AGPL）。

### thedotmack/claude-mem（118k★，TypeScript）

- 定位：跨会话持久上下文，把 agent 的工具调用（读/改/执行）变成记忆。
- 架构（README 明说）：宿主 **hooks** → 本地 worker（Bun 管理的 HTTP 服务）→ **SQLite** 存 sessions / observations / summaries（`src/services/sqlite/`，含 FTS5）+ **Chroma** 向量库补语义检索（`src/services/worker/search/`）；对外用 **MCP server + mem-search skill** 提供检索。
- 对我们的意义：①"hook 采集 → 本地 worker → 关系库 + 向量库"这条链路与我们 Perception + Memory 的形态高度同构，**TypeScript 实现可直接对照**；②它的"一个 adapter 一个 agent"（`src/cli/adapters/`）是感知层适配器模式的现成例子；③注意它把重活放在独立 worker 进程里——DSH 插件是进程内运行，这一点不能照搬。

### headroomlabs-ai/headroom（80k★，Python + TypeScript，**Apache-2.0**）

- 定位：**上下文压缩层**——在内容进入模型前压缩（工具输出、日志、RAG 块、文件、会话历史）；实测四类 agent 场景省 21–57% input tokens，重复 JSON/日志行可到 90%+，散文几乎不省。压缩全在本地跑。
- 形态：PyPI + npm（`headroom-ai`）双发布、带 MCP server；可作 proxy 也可作库用。
- 对我们的意义：①Context Composer 的预算控制可以借"按内容类型分级压缩"的思路（不是所有内容都值得同样预算）；②它证明了"重复结构（JSON/日志）压缩率高、散文压缩率低"这一经验分布；③Apache-2.0 允许直接依赖或改，**是三个 P0 里唯一可以放心吃进代码的**。

