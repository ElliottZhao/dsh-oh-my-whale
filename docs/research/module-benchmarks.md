# 模块对标调研：开源参考实现清单

> **本文是什么**：项目规矩（AGENTS.md「设计与调研」）的第一次执行——「每个模块进入设计前，必须先列出 3–5 个开源对标项目（附仓库 URL），并在 `docs/research/` 留下调研结论」。本文是这条规矩的产物，不是设计文档。
>
> **取数口径**（2026-11-18）：仓库名 / 许可证 / 语言 / 最近 release 与 push 时间，经 `gh api repos/<org>/<repo>` 实测；机制细节来自官方 README、官方文档与仓内源码路径。**凡标「未确认」者即未核实，本文不臆造 star 数、API 或版本号。**「文档声称」与「代码实现」在条目内分别标注（`[文档]` / `[源码]`）。
>
> **我们是谁**：单用户、local-first、Node/TS、**无 Python 运行时依赖**的一对一 Personal Agent。可用性只有三档：**借鉴设计**（抄接口/数据结构）、**可直接依赖**（npm/TS 可用）、**不适用**。
>
> **记忆模块不在此重复**：记忆的系统对标已有独立调研，见 [`docs/research/agent-memory-systems.md`](agent-memory-systems.md)（Cognee / Mem0 / Letta / Graphiti / A-MEM / MemOS / MIRIX / Memobase / Supermemory / LangMem / HippoRAG / Generative Agents）。本文只在总表保留一行。

---

## 总表：模块 × 首选 × 备选 × 可用性

| 模块 | 首选对标 | 备选 | 我们的可用性 |
|---|---|---|---|
| 记忆 | 见 `agent-memory-systems.md`（此行仅为占位） | — | 见该文 |
| 用户画像 | Letta memory blocks（`{label,value,limit,description}`） | LangMem（update-in-place）· SillyTavern Card V2 外壳 | 借鉴设计 |
| 自我模型（自省） | ExpeL（跨任务洞察） | Reflexion（语言化反思）· AWM（可复用 workflow）· Voyager（能力库） | 借鉴设计 |
| 上下文装配 | MemGPT/Letta（三段预算 + 显式溢出） | OpenHands Condenser（可插拔压缩策略）· LangMem（两档延迟）· Zep（失效语义） | 借鉴设计 |
| 策略仲裁（Governor） | LangGraph supervisor（state + checkpointer） | Semantic Router（语义路由 + 阈值兜底）· CrewAI（manager-as-agent）· AutoGen（候选+description 路由签名） | 借鉴设计 |
| 感知 | Generative Agents ConceptNode（观察流） | Cognee ECL（抽取/建图/加载切分）· LlamaIndex（transformations 数组）· Unstructured（element schema） | 借鉴设计 |
| 主动性 / 目标引擎 | Voyager 课程生成器（能力边界自适应） | AutoGPT block 图 + 预算护栏 · ProactiveAgent（时机/意图分离）· Agent Zero（子 agent 隔离） | 借鉴设计 |
| 情感与关系 | Honcho（有向 `(observer,observed)` + Peer Card） | SillyTavern 世界书（条件注入）· ESConv 策略标签（评测） | 借鉴设计 |
| 世界模型 | Graphiti（双时间有效边） | HippoRAG（图上游走 + 便宜离线索引）· LightRAG（双层检索）· GraphRAG（社区摘要+溯源） | 借鉴设计 |
| 持续学习 | DSPy（signature + 编译优化，理念层） | ExpeL（insight 增删改）· AWM（workflow 归纳）· TextGrad（文本梯度） | 借鉴设计 |
| 权限边界（横切） | Open Interpreter（确认消息 + 生成器阻塞门） | HumanLayer（daemon 拓扑 + 计划/复查驾驭面）· E2B（沙箱生命周期接口） | 借鉴设计 |
| 评估（横切） | LongMemEval（MIT 数据可直接吃 + 五能力矩阵） | τ-bench（Pass^k / 终态比对）· MemBench（噪声注入）· AgentBench（容器化 harness） | 数据可直接依赖 + 协议借鉴 |

**一句话总结**：本清单里**没有一个是 Node/TS 可直接依赖的运行时**；唯二 TS 通路是 Honcho（`@honcho-ai/sdk` + 公开的 `@honcho-ai/dsh-honcho` 插件）与 Cognee（TypeScript SDK），两者都带独立 Python/Postgres 后端且 Honcho 是 AGPL。所以本轮的产出全部是**设计借鉴**，不是依赖选型。

---

## 逐模块

### 用户画像

**Letta memory blocks**｜https://github.com/letta-ai/letta ｜Apache-2.0｜Python（核心服务 + SDK）｜`0.16.8`（2026-05-14），push 2026-10-18
- 定位与机制 `[源码]`：长期记忆拆成**具名、有上限、可独立读写的文本块**（经典 `persona` / `human`），常驻拼进 system prompt；块是独立资源，由 `core_memory_append` / `core_memory_replace` 让 agent 自己改；与 archival memory 构成「常驻小 + 档案大」两层。
- 可借鉴：块结构 `{label, value, limit, description}`（label 寻址、limit 强制压缩、description 教模型该放什么）；`replace` 要求 `old_content` 精确匹配才写（乐观并发，天然防覆盖）。
- 坑：`limit` 按**字符**非 token，中文场景会算偏；attach/detach 是为多 agent 设计，单用户是多余复杂度。
- 可用性：**借鉴设计**（画像数据结构首选参考）。

**Honcho**｜https://github.com/plastic-labs/honcho ｜**AGPL-3.0**｜Python（FastAPI）+ 官方 **TypeScript SDK** `@honcho-ai/sdk`｜`v3.6.1`（2026-11-11），push 2026-11-17
- 定位与机制 `[文档+源码]`：把人 / agent 建模为随时间演化的 **Peer** 实体；`Workspace → Peer → Session → Scope → Message`；后台 deriver 异步推导 peer 表征，观察按 **`(observer, observed)`** 键存为向量文档（相等即自我表征，不等即「A 眼中的 B」）。
- 可借鉴：Peer 统一建模（人不分 user/assistant 两套表）；同步存储 / 异步推导分离；`session.context(tokens=N).to_openai()` 的 token 预算打包签名。
- 坑：AGPL；需 Postgres + pgvector；**Honcho 官方另有公开 npm 包 `@honcho-ai/dsh-honcho`**（注入 system prompt + 记忆工具），与本项目画像/关系模块**功能重叠**——引用前须先划边界（把它当竞品/可选后端，而不是只当"对标项目"）。
- 可用性：**借鉴设计为主**；因有 TS SDK，属「可评估直接依赖」，但需先决策 AGPL 与竞合。

**LangMem profiles**｜https://github.com/langchain-ai/langmem ｜MIT｜Python｜无 release，push 2026-11-16
- 定位与机制 `[文档]`：LangChain 官方的记忆抽取库；画像是其中主路径。用 Pydantic/JSON schema 定义画像形状，`create_memory_manager(..., enable_inserts=False)` 让画像**只维护单一实例、原地更新**；再次对话把 `existing` 传回，返回**同一 id 的被改写对象**。
- 可借鉴：**schema 即抽取契约**（TS 侧用 zod/interface，同时约束 LLM 输出与存储）；**update-in-place 语义**（天然去重，远胜「LLM 自由 append 事实」）；`namespace` 元组寻址约定。
- 坑：强绑 LangChain store；`enable_inserts=False` 导致**无法追加列表型事实**（多个「喜欢的乐队」要另建 collection）。
- 可用性：**借鉴设计**（画像写入语义最该照抄的一个）。

**SillyTavern 角色/用户卡**｜https://github.com/SillyTavern/SillyTavern ｜**AGPL-3.0**｜JavaScript（前端，非可复用库）｜`1.19.0`（2026-09-14），push 2026-11-15
- 定位与机制 `[文档]`：角色卡格式是社区事实标准。**Card V2** 外壳 `{spec:'chara_card_v2', spec_version:'2.0', data:{...}, extensions:{}}`，字段用 MUST/SHOULD/MAY 分级；`creator_notes` **MUST NOT** 进 prompt；`extensions` **MUST NOT** 销毁未知键、**SHOULD** 命名空间化。角色私有 `character_book` 条目含 `keys / secondary_keys / selective / constant / insertion_order / position / priority`。
- 可借鉴：**`data` 包裹 + `extensions` 命名空间 + 「不得销毁未知键」**（可长期演化的前向兼容档案外壳，直接采用）；**世界书激活模型** `keys+secondary_keys(双键触发) / constant(恒插入) / insertion_order / priority(超预算先丢谁)`——现成的条件注入数据结构。
- 坑：AGPL——**只可借鉴社区规范文档，不得拷 ST 源码**；`first_mes`/`mes_example` 是 RP 场景假设，对话式产品不必照搬；不要继承 V1/V2 双格式包袱。
- 可用性：**借鉴设计**（档案外壳 + 条件激活模型）。

**Memobase**｜https://github.com/**memodb-io**/memobase ｜Apache-2.0｜Python（FastAPI + Postgres）｜push 2026-01-11，**无 release → 停滞**
- 定位与机制 `[文档]`：以用户画像为检索单元的长期记忆服务；写入时把对话抽成结构化用户信息挂到用户维度，画像常驻、情节按需检索。
- 可借鉴：画像常驻 / 情节按需的分层存储；自带同数据集对比两个记忆后端的评测脚手架。
- 坑：仅服务形态（无嵌入式库）；停更约 10 个月，接口腐化风险高。
- 可用性：**借鉴设计**，不引入其 Python 服务。

> **本模块结论**：最值得抄 **Letta 的 block 结构**（数据结构）与 **LangMem 的 update-in-place**（写入语义）——两者合起来就是「有上限的常驻块 + 原地更新的抽取契约」，可直接落成 TS 类型 + zod schema。SillyTavern 的 `extensions` 外壳作为持久化格式的兼容层。

---

### 自我模型（自省）

**ExpeL**｜https://github.com/**LeapLabTHU**/ExpeL ｜Apache-2.0｜Python｜push 2024-12-20
- 定位与机制 `[文档+论文]`：跨任务「 experiential learning」——收集多任务轨迹，抽取自然语言 **insight（规则）** 存库；推理时检索 insight 注入 prompt；并对成败轨迹做对比以**增删改** insight。
- 可借鉴：insight 存储需支持 **add / edit / remove**（ExpeL 用赞/踩票做规则精炼）——自省不能只增不减；按任务簇键控检索。
- 坑：抽取需多次 LLM 调用；insight 质量和数量都难控，易积累互相矛盾的规则。
- 可用性：**借鉴设计**（自省条目的生命周期管理）。

**Reflexion**｜https://github.com/noahshinn/reflexion ｜MIT｜Python｜push 2025-01-14
- 定位与机制 `[文档+论文]`：Actor / Evaluator / Self-Reflection 三件套；任务失败后让 LLM 写一段**自然语言反思**存入 episodic buffer，下一次尝试前取 top-k 注入。
- 可借鉴：反射对象 `{task, failure_signal, lesson}` 追加式缓冲 + 按任务相似度取 top-k；**必须设上限**（1–3 条），否则上下文被旧教训占满。
- 坑：反思质量取决于失败信号质量；无验证的反思会把「错的教训」固化。
- 可用性：**借鉴设计**（最轻的自省骨架，几十行 TS）。

**Agent Workflow Memory**｜https://github.com/**zorazrw**/agent-workflow-memory ｜Apache-2.0｜Python｜push 2025-12-22
- 定位与机制 `[文档]`：从历史轨迹**归纳可复用 workflow（子程序）**，抽象掉示例特定上下文；支持 offline（从标注样例归纳）与 online（边跑边归纳）两种模式；检索后注入。
- 可借鉴：`workflow = {name, steps[], preconditions, source_task_ids}`；**成功后归纳**、按任务描述相似度检索；workflow 可作为「我知道怎么做这类事」的程序性自我模型。
- 坑：归纳步骤本身要 LLM，成本高；workflow 粒度难定（太细不复用、太粗不适用）。
- 可用性：**借鉴设计**。

**Voyager skill library**｜https://github.com/MineDojo/Voyager ｜MIT｜JS bridge + Python 循环｜最后提交 2023-07，push 2024-04（停滞）
- 定位与机制 `[文档+源码]`：技能 = **可执行代码 + 自然语言描述**，存向量库按描述检索复用；执行成功才入库，失败不入。
- 可借鉴：能力登记项 `{id, description, code/toolRef, success_count, last_used}`；检索走描述 embedding；**只登记成功案例**——这是「自我模型」里"我会什么"的可信基线。
- 坑：强绑 Minecraft；技能库只增不减，规模上去后检索噪声大；生成代码直接执行无沙箱。
- 可用性：**借鉴设计**（能力登记表结构）。

**Letta self-editing memory**｜（同 1.3 仓库）｜Apache-2.0｜Python｜`0.16.8`（2026-05-14）
- 定位与机制 `[源码]`：`persona` 块即「我是谁」的自我描述，由 agent 通过工具改写；改动有旧内容匹配校验。
- 可借鉴：自我描述也应是**有上限、可被自己编辑、可审计**的块，而不是埋在 system prompt 里的一句长文本。
- 坑：自编辑无版本回滚时，一次错误改写会长期污染自我模型。
- 可用性：**借鉴设计**。

> **本模块结论**：最值得抄 **ExpeL 的 insight 增删改** + **Reflexion 的失败→反思→限条数注入**。自我模型不是一个 summary，而是一组**有生命周期（生/改/废）的条目**；Voyager 的"只登记成功"是能力边界的可信锚。

---

### 上下文装配

**MemGPT / Letta 虚拟上下文**｜https://github.com/letta-ai/letta （`cpacker/MemGPT` 已重定向至此；旧 V1 服务在 `archive` 分支）｜Apache-2.0｜Python；当前源码另见 https://github.com/letta-ai/letta-code （**TypeScript**，Apache-2.0，push 2026-11-18）
- 定位与机制 `[文档+源码]`：把上下文窗口当"内存"管——core memory（常驻块）/ recall（全对话）/ archival（向量库）；接近窗口上限时触发 **memory pressure**，agent 必须先换页（摘要落盘）才能继续，**溢出是显式事件而非静默截断**。
- 可借鉴：**三段式预算**（固定区 system+blocks / 可变区近期消息+滚动摘要 / 按需区 archival 检索结果，各带 token 配额）；溢出时注入一条系统消息告知「已压缩，去查档案」；`summarize_messages_inplace()` 的压缩入口。
- 坑：压缩本身要再调一次 LLM（成本+延迟）；摘要必有损；记忆写不写全靠模型自觉。
- 可用性：**借鉴设计**（首选骨架）。`letta-code` 虽有 TS SDK，但自带 prompts/工具的完整 harness，嵌入会被其体系绑架。

**OpenHands Condenser**｜https://github.com/**OpenHands**/OpenHands （`All-Hands-AI/OpenHands` 已重定向）｜MIT｜仓库语言 TS（前端），核心 agent 逻辑在 Python SDK｜`v1.41.0`（2026-11-16），push 2026-11-18
- 定位与机制 `[文档+源码路径]`：事件流驱动（类型化 action/observation 全部写入 event stream，可回放）；上下文 = 从事件流 **condense** 出的视图；`LLMSummarizingCondenser` / `RecentEventsCondenser` / `AmortizedForgettingCondenser` 是可插拔策略。
- 可借鉴：**Condenser 接口 `condense(events) -> View`**——「按预算选策略」而不是 if-else 硬编码；**事件流是唯一事实源**，上下文只是视图（解决 messages 列表漂移）；类型化 action 便于校验与权限控制。
- 坑：极重（Docker + 前后端）；面向软件工程，通用任务要大量改造。
- 可用性：**借鉴设计**（本模块最值得抄的接口形状）。

**Zep context API**｜https://github.com/getzep/zep ｜Apache-2.0｜Python｜`zep-ingest-v0.4.1`（2026-11-17）
- 定位与机制 `[文档]`：**注意该仓库已不是产品本体**（README 明写 "not Zep's product or service"），现为示例 / 集成 / 摄取管线；时序图谱内核在 Graphiti。开源做法：边带 `valid_at / invalid_at`，事实"失效"而非"删除"。
- 可借鉴：**事实四元组 + 失效语义**（`{fact, valid_at, invalid_at, created_at}`，检索按当前时点过滤）——解决「用户偏好变了但旧记忆还在」；分层摄取 `episode → entity/edge → community` 每层可重跑可回溯。
- 坑：把 `getzep/zep` 当自托管服务会踩空；写路径多次 LLM 调用；依赖图数据库。
- 可用性：**借鉴设计**（时序失效模型；图库本身不适用单用户本地）。

**LangMem**｜（同 1.4）｜MIT｜Python｜无 release
- 定位与机制 `[文档]`：hot-path 只做 `search`（快），抽取/合并放异步（贵）；存储抽象是 `(namespace, key) → {content, kind}`，可选向量索引。
- 可借鉴：**两档延迟**把「装配」与「写入」在时间上解耦——本地 agent 控延迟的关键；最简存储模型可直接映射到本地 JSONL/SQLite。
- 坑：无时间有效性模型（不如 Zep）；无 release 会漂移。
- 可用性：**借鉴设计**。

**LLMLingua**｜https://github.com/microsoft/LLMLingua ｜MIT｜Python（PyTorch）｜`v0.2.2`（2024-04-09），实质停滞
- 定位与机制 `[文档+代码]`：用小模型算 per-token 信息量，按预算删低信息 token；LongLLMLingua 支持「按问题条件压缩」。
- 可借鉴：**按 query 条件压缩**（同一上下文对不同 query 压出不同版本）；显式 `rate` / `target_token` 预算参数可做装配器下游接口。
- 坑：**要跑小模型（torch）→ 与本项目硬冲突**；对代码/JSON/结构化上下文会破坏语法；高压缩率下最易丢否定词/数字/边界条件。
- 可用性：**不适用**（Python + torch）。等价替代是「规则化裁剪 + LLM 摘要」。

> **本模块结论**：抄 **OpenHands 的 Condenser 接口** + **MemGPT 的三段预算与显式溢出**。装配器应是 `EventStream → ContextAssembler(固定区|滚动摘要|按需检索) → View`，每个 condenser 是可替换对象。GraphRAG 归世界模型，LLMLingua 是反面教材。

---

### 策略仲裁（Governor）

**LangGraph supervisor**｜https://github.com/langchain-ai/langgraph ｜MIT｜Python｜`1.2.22`（2026-11-14），push 2026-11-18（另有 `langchain-ai/langgraphjs`，**JS 版能力对齐度未确认**）
- 定位与机制 `[文档]`：图式编排 + 共享 state（reducer 如 `add_messages`）；supervisor 是中心节点按需路由到子 agent；**Checkpointer** 每步快照，支持时间旅行、human-in-the-loop interrupt、断点恢复，按 `thread_id` 隔离。
- 可借鉴：**state 是唯一真相**（上下文装配退化为「从 state 选字段渲染」）；Checkpointer 用 **SQLite 单文件**即可实现撤销/回放/人工介入；`Command(goto, update)` 让路由与状态更新**原子完成**（消除两步竞态）；reducer 解决多节点并发写。
- 坑：图复杂后调试依赖可视化；循环边需 `recursion_limit`；强绑 LangChain。
- 可用性：**借鉴设计**（SQLite checkpointer + 条件边 + reducer）。

**Semantic Router**｜https://github.com/aurelio-labs/semantic-router ｜MIT｜Python｜`v1.1.2`（2026-11-09）
- 定位与机制 `[文档]`：声明式 `Route {name, utterances[]}`，启动时编码 utterances，查询时算相似度返回 `RouteChoice{name, similarity_score}`，命中可直接产出 function call（省一次 LLM）。
- 可借鉴：**声明式路由表 + `similarity_score` + `threshold`**（低于阈值走 LLM 兜底，避免硬路由错配——这是生产可用的关键）；本地 embedding 可在 TS 内实现，无服务端依赖。
- 坑：纯语义相似对"要看参数才能决定"的选择不可靠；单句路由无多轮上下文；TS 成熟移植**未确认**。
- 可用性：**借鉴设计 / 可自实现后依赖**（本模块对 local-first + Node/TS 最友好的一项）。

**CrewAI hierarchical**｜https://github.com/crewAIInc/crewAI ｜MIT｜Python｜`1.15.41`（2026-11-12），push 2026-11-18
- 定位与机制 `[文档]`：`Process.hierarchical` 自动创建 manager agent 分派任务、接收回报、判断完成；任务有 `expected_output` 契约与 `context=[taskA,taskB]` 依赖声明；输出过 guardrail 校验后交付。
- 可借鉴：**manager-as-agent**（不写硬编码调度器，委派决策复用同一 LLM 循环 + 两个内置工具：委派/提问）；`expected_output` 作为**可判定的验收契约**；`context` 显式声明数据依赖（轻量 DAG）；guardrail 前置校验 + 重试上限。
- 坑：每步多一次 manager 调用，成本/延迟翻倍；manager 易无限委派（必须 `max_iter` 兜底）；`backstory` 长 prompt 挤占预算。
- 可用性：**借鉴设计**。

**AutoGen speaker selection**｜https://github.com/microsoft/autogen ｜API 报 **CC-BY-4.0**（疑为文档许可，**代码许可未确认**）｜Python｜`python-v0.7.5`（2025-09-30）
- 定位与机制 `[文档]`：`GroupChat` 每轮选下一个发言者；`auto` 模式把历史 + 每个 agent 的 `description` 塞进一次 LLM 调用，只要求输出一个名字；终止条件是独立可组合链。
- 可借鉴：**路由签名极简** `(history, candidates[{name, description}]) → speaker_name`——新增能力只需加一条描述；**终止与选人分离**（终止不要写进路由 prompt）；按类型路由（topic/subscription）比按人名更便宜可靠。
- 坑：每轮多一次 LLM 调用，会选错、会选自己（需白名单）；全量历史塞 prompt → token 近似平方增长；**API 断代**（0.2→0.4 重写）。
- 可用性：**借鉴设计**（20 行 TS 即可复刻路由签名）。

**RouteLLM**｜https://github.com/lm-sys/RouteLLM ｜Apache-2.0｜Python｜**最后提交 2024-08-09，无 release（停滞近两年）**
- 定位与机制 `[文档]`：强/弱模型路由器，按预测的 `strong_win_rate` 与单一 `threshold` 决定路由；自带成本-质量评估 harness。
- 可借鉴：**`calibrated_score ∈ [0,1]` + 单一 threshold 旋钮**（可度量、可 A/B）；可插拔 Router 接口；离线评估先行。
- 坑：偏好数据是通用闲聊，**不适用于工具调用**；停更 + 依赖旧 SDK；只做模型选择、不做能力选择。
- 可用性：**借鉴设计**（阈值校准思路），不直接依赖。

> **本模块结论**：抄 **LangGraph 的 checkpointer/Command 原子路由** + **Semantic Router 的声明式路由表 + 阈值兜底**。Governor 应输出 `{choice, score, threshold, fallback}`，路由与状态更新同一步完成；AutoGen 的「候选+description」签名可直接拿来当 Governor 的接口。

---

### 感知

**Generative Agents Observation Stream**｜https://github.com/joonspk-research/generative_agents ｜Apache-2.0｜Python｜push 2024-08-05（论文代码，停更）
- 定位与机制 `[源码: persona/memory_structures/associative_memory.py]`：记忆原子 `ConceptNode{node_id, type∈{event,thought,chat}, depth, created, expiration, last_accessed, subject·predicate·object, description, embedding_key, poignancy, keywords, filling}`；四类索引并存（时间序、id、关键词倒排、关键词强度）；检索为**相关性 + 显著性(poignancy) + 时效性(last_accessed 衰减)** 三信号复合；`depth = 1 + max(filling.depth)` 表达推演层级。
- 可借鉴：**节点字段集直接照抄**；`depth` 三行算法（推理链深度可追溯）；`last_accessed` 参与检索衰减（访问即刷新，最省事的"常用记忆更强"）；小数据量下四索引齐全就不必上向量库。
- 坑：poignancy / keywords 各需一次 LLM 调用（成本随对话线性涨）；SPO 三元组对中文闲聊常退化；原型无并发考虑。
- 可用性：**借鉴设计**（本模块首选）。

**Cognee ECL**｜https://github.com/topoteretes/cognee ｜Apache-2.0｜Python + **官方 TypeScript SDK** + REST + MCP｜`v1.6.8`（2026-11-12），push 2026-11-18
- 定位与机制 `[文档]`：**ECL = Extract → Cognify → Load**（抽实体关系，无 LLM key 时用本地 GLiNER → 建图并关联 chunk → 入图持久化）；对外四动词 `remember / recall / improve / forget`。
- 可借鉴：ECL **三阶段职责切分**（每段可独立失败重试）；**四动词 API 面**（`improve` 单列——反馈再加工不混进写入）；**「无 LLM 也能跑」的降级路径**（正是 local-first 该有的默认）。
- 坑：默认多租户姿态（单用户须显式关后端访问控制）；Postgres-as-graph 官方自述仍是 demo 特性；整体是平台不是库。
- 可用性：**借鉴设计**；若确需图式感知，它是唯一技术 + 许可都够得着的 TS 重型候选。

**LlamaIndex IngestionPipeline**｜https://github.com/run-llama/llama_index ｜MIT｜Python（TS 版为独立仓库 LlamaIndexTS，**能力对等未确认**）｜`v0.14.26`（2026-11-13）
- 定位与机制 `[文档]`：`Loader → Transformations（切块/抽元数据/嵌入/去重）→ 写入 Index`；变换是可组合数组；支持增量与缓存（文档未变则跳过）。
- 可借鉴：**变换数组的编排签名**（`pipeline.transformations = [split, extractMeta, embed]`）；**摄取与索引阶段分离**（中间产物是可检视的 node，先做规则清洗再落库）；**增量 + 缓存**语义（本地单用户必做）。
- 坑：抽象面极宽，照抄会过度设计；核心是 Python。
- 可用性：**借鉴设计**。

**Unstructured**｜https://github.com/Unstructured-IO/unstructured ｜Apache-2.0｜Python｜`0.27.37`（2026-11-06）
- 定位与机制 `[文档]`：把 PDF/HTML/DOCX 解析成**带类型的 element 序列**：`{type: Title|NarrativeText|Table|ListItem|…, text, metadata}`，下游按类型分别切块。
- 可借鉴：**element 类型化输出**这个中间契约——即便只做 Markdown/HTML 摄取，也应自定 `{type, text, metadata}`。
- 坑：重依赖（OCR/布局模型）；Python-only，对 Node 栈只有 schema 参考价值。
- 可用性：**借鉴设计**（element schema）。

> **本模块结论**：抄 **Generative Agents 的 ConceptNode + depth + 三信号检索**（数据结构与打分），摄取管线抄 **LlamaIndex 的变换数组 + 增量缓存**。感知最小版只消费 DSH 会话信号，所以 element schema 先留接口、不急着实现。

---

### 主动性 / 目标引擎

**Voyager Automatic Curriculum**｜https://github.com/MineDojo/Voyager ｜MIT｜JS bridge + Python｜停更
- 定位与机制 `[文档+源码]`：课程生成器以「当前技能库 + 探索状态」为输入，让 LLM 提议**下一个可完成但有挑战的目标**，难度随能力边界隐式调节。
- 可借鉴：**课程 = 目标生成器** `(已有能力, 当前状态) → next_goal`，难度自适应无需手工梯度；技能库条目的 `usage_count` 可做"最近没练到的能力"优先。
- 坑：强绑 Minecraft；课程可能提出当前环境做不到的目标（需可行性过滤）。
- 可用性：**借鉴设计**（本模块首选接口）。

**AutoGPT（Platform）**｜https://github.com/Significant-Gravitas/AutoGPT ｜**分目录许可**：`autogpt_platform/` 为 **Polyform Shield 1.0.0**（含 noncompete），其余 MIT（已解码 LICENSE 正文）｜Python｜`autogpt-platform-beta-v0.11.3`（2026-11-15）
- 定位与机制 `[文档]`：块编排（`{id, input_schema, output_schema, run(input)→output}`）+ 图执行（节点级状态落库，支持暂停/恢复/单节点重跑）+ triggers（cron/webhook）+ 凭据管理。
- 可借鉴：**Block 抽象 + 图执行器**（能力 = 可组合节点）；**节点级执行状态落库**（暂停/恢复/重跑是目标引擎必备）；**硬预算护栏**（每轮 token 上限 + 最大循环数 + 成本追踪，超限即停）；**触发与执行解耦**（cron/webhook 触发一次图执行，而不是常驻轮询——与 DSH 定时任务最小 60s 的现实契合）。
- 坑：Classic 与 Platform 两套并存、文档混乱；Platform 是完整 SaaS（Postgres+Redis+worker），本地部署极重；**许可陷阱：整仓不能当 MIT 用**。
- 可用性：**借鉴设计**（block 图 + 预算护栏 + 触发器）；代码不建议依赖。

**BabyAGI**｜https://github.com/yoheinakajima/babyagi ｜**许可未确认**（README 称 MIT，但仓库未识别出 LICENSE 文件）｜Python｜push 2026-01-31，无 release
- 定位与机制 `[文档]`：原始循环 = `task_creation → prioritization → execution`（objective + 上次结果生成新任务，重排后执行 top1，结果写库，循环至 max_iterations）；现行 main 转向 functionz（函数依赖图 + 触发器）。
- 可借鉴：**任务队列即目标引擎** `(objective, task_list, last_result) → 新任务 → 重排 → 执行 top1`（200 行 TS 可复刻）；**显式再规划点**（优先级重排是"集中决策"的窄接口，比每步自由思考便宜可审计）；函数 `dependencies` 元数据 → 拓扑加载顺序。
- 坑：原始版是 demo（无状态恢复、无错误处理、任务无限膨胀、objective 漂移）；每轮 2–3 次 LLM 调用；许可未确认。
- 可用性：**借鉴设计**。

**ProactiveAgent（正名）**｜https://github.com/**thunlp**/ProactiveAgent （`aitraveler/ProactiveAgent` **不存在，404**）｜Apache-2.0｜Python｜push 2026-05-12，无 release
- 定位与机制 `[文档]`：把"主动协助"拆成三个可分别建模的决策——**何时介入（时机）/ 介入什么（意图）/ 如何介入（建议 vs 执行）**；用真实活动序列预测下一步任务，并配套数据构造与评估。
- 可借鉴：**「何时」与「做什么」分成两个决策**（不要每轮都评估主动性——先用廉价时机触发器，再决定内容；这是本地 agent 防打扰的核心）；用**行为序列而非单条消息**预测意图（滑窗）；评估要同时看**准确率与打扰成本（误报）**。
- 坑：学术代码、工程化低；依赖特定数据集；论文指标 ≠ 产品可用性（真实场景误报代价极高）。
- 可用性：**借鉴设计**（时机/意图分离）。

**Agent Zero**｜https://github.com/**agent0ai**/agent-zero （`frdel/agent-zero` 已重定向）｜**MIT（LICENSE 正文已解码核实；API 报 NOASSERTION）**｜Python 后端 + Web 前端｜`v2.16`（2026-11-17）
- 定位与机制 `[文档]`：主 agent + 可派生**独立上下文的子 agent**（只回传结论）；**工具即文件**（`tools/*.py` 按文件名注册，可在运行时写新工具并立即调用）；可编辑的长期 memory 文件；前端实时展示 token/上下文占用。
- 可借鉴：**子 agent = 上下文隔离手段**（把脏活外包、只回传结论，比摘要更彻底地保住主上下文）；**工具目录即注册表**（落盘即生效无需重启）；**人类可读可 diff 的 memory**；**预算可视化**。
- 坑：默认依赖 Docker 沙箱，**关掉后代码执行无隔离，风险极高**；行为依赖 system prompt 调参、不稳定。
- 可用性：**借鉴设计**。

> **本模块结论**：抄 **Voyager 的课程生成器**（"下一步做什么"的自适应接口）+ **AutoGPT 的硬预算护栏 + 触发器解耦**。主动性 v1 上限是"提议/预取"，所以 Governor 要在提议前做**时机门槛**（ProactiveAgent 的分离建模），而不是每轮都生成提议。

---

### 情感与关系

**Honcho**｜（同 1.2）｜**AGPL-3.0**｜Python + TS SDK｜`v3.6.1`（2026-11-11）
- 定位与机制 `[文档]`：关系 = 有向 **`(observer, observed)`** 表征——「我眼中的你」与「你眼中的我」分别演化、可冲突可对账；**Peer Cards** 作紧凑身份/关系摘要常驻；低延迟 representation / 高质量 chat 两档读取。
- 可借鉴：**有向关系表征**（比单一无向关系值更接近真实关系）；**Peer Card 作关系常驻压缩表示**；「先快照再深问」的两档读取。
- 坑：AGPL；关系是**推演出的自然语言结论，没有可计算数值状态**（信任/亲密度必须自建）；需 Postgres+pgvector。
- 可用性：**借鉴设计**（有向表征 + 关系卡）。

**SillyTavern 世界书（关系视角）**｜（同 1.5 仓库）｜AGPL-3.0｜JavaScript｜`1.19.0`（2026-09-14）
- 定位与机制 `[文档]`：`constant` 条目常驻关系设定；`insertion_order` / `priority` 在 token 预算内决定"关系设定哪一段先进上下文"——社区跑得最久的长程关系维系实践。
- 可借鉴：**关系状态用「可条件激活的条目集合」表达**，而非不断变长的 summary，并按 `priority` 做预算内淘汰。
- 坑：**没有显式关系数值/阶段机**，关系只是自然语言条目；要可计算的关系必须自建。
- 可用性：**借鉴设计**（只借鉴规范，不拷源码）。

**ESConv**｜数据集：https://huggingface.co/datasets/thu-coai/esconv ｜**CC-BY-NC-4.0（禁商用）**｜数据集，**官方 GitHub 代码仓库无法确认**（`thu-coai/ESConv` 404）｜数据集 lastModified 2023-07-15
- 定位与机制 `[数据集卡片]`：多轮情感支持对话，每轮带 **strategy（支持策略）** 标注（共情 / 安抚 / 建议 / 追问），并带求助者情绪。
- 可借鉴：**「策略」作为一等标签**，与情绪拆成两个维度（`shouldRespondWith(strategy)` 而不是"表达情感"）；策略→轮次对齐可作评测标注模板。
- 坑：**CC-BY-NC 不可商用**；纯英文；无代码可依赖。
- 可用性：**借鉴设计**（仅评测标注参考，不进产品数据）。

**CharacterGLM**｜https://github.com/**thu-coai**/CharacterGLM-6B （`THUDM/CharacterGLM-6B` 404）｜Apache-2.0｜Python（6B 权重）｜push 2025-10-02，release **未确认**
- 定位与机制 `[论文+文档]`：中文角色化对话模型；核心贡献是「人设如何约束对话」的数据与方法——角色属性（性格 / 经历 / 说话风格 / 与用户关系）分开描述并被遵守，配角色一致性评测。
- 可借鉴：**人设字段的拆法**（分开字段而非一段自由文本）+ **角色一致性**评测维度。
- 坑：6B 权重需 GPU + Python 推理栈，与本项目硬冲突；价值在论文与字段设计。
- 可用性：**不适用**（运行时）/ **借鉴设计**（人设字段与一致性评测）。

**EmpatheticDialogues**｜https://github.com/facebookresearch/EmpatheticDialogues ｜**NOASSERTION**（实际条款未确认）｜Python｜**已归档**（archived=true），push 2021-12-03
- 定位与机制 `[文档]`：以带情绪标签的「情境」为起点的共情回应数据集（32 类情绪）。
- 可借鉴：**「情绪标签 × 情境」作为检索键**——查历史时按同类情绪情境召回，比纯语义相似更贴情绪连续性。
- 坑：归档 5 年、脚本基本不可运行；许可 NOASSERTION 有法务风险。
- 可用性：**仅参考**（组织法），不依赖代码或数据。

> **本模块结论**：抄 **Honcho 的有向 `(observer,observed)` 关系表征 + Peer Card 压缩**，配 **SillyTavern 的条件激活 + priority 淘汰** 做常驻关系条目。**必须自建可计算的关系慢变量**（信任/亲密度/禁忌）——所有对标项目的关系都只是自然语言结论。情感数据集（ESConv / EmpatheticDialogues）只能作评测标注，且两者许可都进不了产品。

---

### 世界模型

**Graphiti**｜https://github.com/getzep/graphiti ｜Apache-2.0｜Python｜`v0.31.2`（2026-10-20），push 2026-11-17
- 定位与机制 `[文档]`：时序知识图谱——episode 为原始输入，LLM 抽实体/关系边；边带**双时间** `valid_at / invalid_at`；增量更新无需批处理重算；混合检索（语义 + BM25 + 图）。
- 可借鉴：**双时间有效边**（事实失效而非删除，可答"当时为真"）；**episode → entity/edge 的增量管线**（不是全局重算）；检索三路混合。
- 坑：写路径多次 LLM 调用；依赖图数据库（Neo4j/FalkorDB 类）→ 与单二进制 local-first 冲突；社区聚类是全局算法。
- 可用性：**借鉴设计**（本模块首选：双时间模型）。

**HippoRAG 2**｜https://github.com/OSU-NLP-Group/HippoRAG ｜MIT｜Python｜push 2026-10-06（ICML'25）
- 定位与机制 `[文档+源码]`：KG + **Personalized PageRank** 在 passage-entity 图上做多跳联想检索；离线 OpenIE 比 GraphRAG/LightRAG 便宜得多；提供 **MCP server**：工具 `retrieve / rag_qa / index / delete / index_stats`；索引带 `index_manifest.json`（把向量与抽取状态绑定到模型/端点身份，不匹配则拒绝混用）。
- 可借鉴：**图上游走（PPR）做联想检索**；**MCP 五工具面**（`retrieve/index/delete/index_stats` 是本地记忆服务的干净接口）；**索引身份清单**（换 embedding 模型时就拒绝复用旧索引——防止静默错配，工程价值高）。
- 坑：仍需 Python + torch；OpenIE 质量决定上限；实体消歧在中文上脆弱。
- 可用性：**借鉴设计**（PPR + 工具面 + 索引清单）。

**LightRAG**｜https://github.com/HKUDS/LightRAG ｜MIT｜Python｜`v1.5.8rc1`（2026-11-15），push 2026-11-18
- 定位与机制 `[文档]`：图 + 向量的**双层检索**——低层按具体实体、高层按主题/关系；增量插入无需全量重建。
- 可借鉴：**低层/高层双层检索键**（同一问句同时走实体精确匹配与主题概括）；**增量插入**是单用户高频写入的必要能力。
- 坑：Python；抽取成本仍高。
- 可用性：**借鉴设计**。

**GraphRAG**｜https://github.com/microsoft/graphrag ｜MIT｜Python｜`v3.3.0`（2026-10-08），push 2026-11-16
- 定位与机制 `[文档]`：离线索引（分块 → 抽 entity/relationship/claim，每条带**来源 chunk 引用** → Leiden 层次社区 → 社区 report 摘要）；在线 local / global / DRIFT 三种查询；产物是 parquet 表。
- 可借鉴：**「预计算摘要块」作为上下文单元**（把昂贵理解离线做掉，在线只选取）；**强制溯源 `source_id → text_unit`**（每条事实带 `derived_from`，回答可回指原文——上下文诚实性的工程底座）。
- 坑：索引成本极高（千次 LLM 调用），本地单用户不可行；社区是全局聚类，高频变化上下文每次新增都可能触发重算；重依赖。
- 可用性：**不适用**（实时装配）；仅静态知识库离线预处理场景可借鉴摘要块与溯源字段。

**KAG（OpenSPG）**｜https://github.com/OpenSPG/KAG ｜Apache-2.0｜Python（依赖 OpenSPG 服务）｜`v0.8.0`（2025-06-28），push 2026-01-28
- 定位与机制 `[文档+论文]`：schema 约束的语义增强可编程图（SPG）+ **逻辑形式（logical-form）引导的推理**；文本 chunk 与图谱互索引，按问题类型选推理路径。
- 可借鉴：**schema/本体约束实体类型**（抽出来的图要符合预定义类型，而不是自由抽）；**question-type 驱动的检索路径选择**。
- 坑：需 OpenSPG 服务（Java 栈）+ 本体定义，冷启动门槛高；许可 Apache-2.0 但部署极重。
- 可用性：**不适用**（重）；仅借鉴"schema 约束抽取 + 问题类型路由"两个概念。

> **本模块结论**：抄 **Graphiti 的双时间有效边**（解决"偏好变了旧记忆还在"）+ **HippoRAG 的 PPR 联想检索与索引身份清单**。GraphRAG 的社区摘要/重索引成本与单用户本地场景根本不合，只借它的 `derived_from` 溯源字段。世界模型本质是记忆之上的一层视图（GLOSSARY 的定义），所以存储仍落本地单文件，**不要引入图数据库**。

---

### 持续学习（策略面）

> 注意：本模块与「自我模型」共享 ExpeL / AWM / Reflexion / Voyager 四个项目，此处只写**策略学习**视角的增量结论，不重复条目。

**DSPy**｜https://github.com/stanfordnlp/dspy ｜MIT｜Python｜`3.5.0`（2026-11-10）
- 定位与机制 `[文档]`：把「程序逻辑」与「prompt 文本」解耦——`Signature`（类型化输入输出契约）+ `Module`；`Optimizer/Compiler` 按评测指标自动优化 few-shot 示例、指令甚至权重。
- 可借鉴：**Signatures 作为 I/O 契约**（我们的 prompt 模块也应声明输入输出 schema，而不是一段自由文本）；**编译即优化**——离线拿 eval trace 重写 prompt 模板，正对应我们的「改规则文件」但要有指标驱动。
- 坑：需要相当量标注样例与多次 LLM 调用；Python-only；优化出的 prompt 可解释性差。
- 可用性：**借鉴设计**（理念层：把 prompt 当可编译对象）。

**TextGrad**｜https://github.com/zou-group/textgrad ｜MIT｜Python｜`v0.1.6`（2024-12-15），push 2025-07-25
- 定位与机制 `[文档+论文]`：对文本变量做「反向传播」——LLM 给 `textual gradient`（自然语言批评），optimizer 据此改写变量（指令 / 答案 / 代码）。
- 可借鉴：**feedback-as-gradient 接口** `{variable, feedback, apply()}`；可用于精炼自我模型条目或提示片段，而不是重训模型。
- 坑：文本梯度保真度不稳；多次调用成本高；`v0.1.6` 后基本停更。
- 可用性：**借鉴设计**（接口形状）。

**ExpeL / AWM / Reflexion / Voyager**｜（见「自我模型」条目）
- 策略面增量：ExpeL 的 insight **增删改**是"策略规则库"的维护范式；AWM 的 **workflow 归纳**是"把成功轨迹变成可复用步骤"；Voyager 的 **只登记成功**是能力库准入；Reflexion 是**失败驱动的策略修正**。四者合起来正好覆盖策略的生命周期：发现 → 固化 → 复用 → 淘汰。
- 可用性：**借鉴设计**。

> **本模块结论**：v2 的策略学习应走 **DSPy 式「签名 + 指标驱动编译」** 的理念：我们的 prompt/规则是**可被离线优化器改写的对象**，不是手写常量。落地形态是「成功轨迹 → 归纳 workflow（AWM）→ 达标则替换旧模板（DSPy 指标）→ 失败则回滚」。

---

### 权限边界（横切）

**Open Interpreter 确认机制**｜https://github.com/**openinterpreter**/openinterpreter （`OpenInterpreter/open-interpreter` 已重定向；经典 Python 确认流在 tag `v0.4.2`，主仓现已转 **Rust** 编码代理）｜Apache-2.0｜Python（历史）/ Rust（当前）｜`rust-v0.0.61`（2026-11-11）
- 定位与机制 `[源码: v0.4.2 core.py]`：执行前先 yield 一条 **confirmation 消息**（`{role:'computer', type:'confirmation', format:'execution', content:{type:'code',format:language,content:code}}`），用**生成器阻塞**等待消费方决定；yield 后**重新从 messages 读取代码**（"They may have edited the code!"）——这是门真正生效的关键；拒绝时**向对话注入一条 user 消息**"I have declined to run this code"，让模型拿到反馈自我调整；`safe_mode` 三档可接二次 LLM 扫描。
- 可借鉴：**confirmation 是消息流里的一等消息类型**（不是 UI 层逻辑，任何前端各自实现门）；**生成器阻塞 = 审批暂停点**（原样移植成 TS async generator + await Promise）；**拒绝回注为 user 消息**；确认消息自带 `format`+`content`，审批 UI 无需二次解析。
- 坑：确认逻辑只在终端前端，走 SDK 时默认无门；二次扫描加延迟；`auto_run` 是全局开关，**无工具/命令级细粒度白名单**——粒度太粗，我们必须补。
- 可用性：**借鉴设计**（消息类型 + 阻塞门 + 拒绝回注三件套）。

**HumanLayer**｜https://github.com/humanlayer/humanlayer ｜Apache-2.0｜TypeScript（Bun + Turborepo）｜push 2026-11-16，有持续 nightly release
- 定位与机制 `[文档+目录结构]`：已从"逐动作审批 SDK"转型为**多编码代理驾驭平台**；拓扑是 **daemon-per-host（`apps/riptide-daemon`）+ 中央 API（oRPC + Hono）+ 客户端增量同步（Electric/Postgres）**；用**会话镜像插件**把第三方 agent 的 session 投影进统一平台；人类驾驭面从"每条动作"升维为 **tasks / sessions / plans / reviews**。
- 可借鉴：**daemon-per-host + 中央 API + 客户端同步** 的拓扑（正是单用户 local-first + 界面驾驭的成熟形态）；**会话镜像插件**（不改造内建 agent，用插件投影 session）；**plans/reviews 作为驾驭面**——对高频只读操作，审批粒度提升到"计划与复查"更省人力；**跨 app 的类型化契约层**（oRPC/Effect）防手写 API 类型漂移。
- 坑：已平台化，当年"一行审批装饰器"的轻量库**不是当前重心**（照抄旧博客会错位）；技术栈极重，整体照搬是过度设计。
- 可用性：**借鉴设计**（daemon 拓扑 + 会话镜像 + 契约分层）。

**E2B**｜https://github.com/e2b-dev/E2B ｜Apache-2.0｜TS-first monorepo（`js-sdk` / `cli` 等）+ 并行 Python SDK｜`e2b@2.64.0`（2026-11-13）
- 定位与机制 `[文档]`：云端隔离沙箱执行 AI 代码；**沙箱即 API** `create → commands.run / files.write / runCode → timeout → kill`，命令返回结构化 `{stdout, stderr, exitCode}`；支持 Dockerfile 模板、超时、持久化重连；隔离层为 Firecracker microVM。
- 可借鉴：**沙箱生命周期作为一等接口序列**（可直接映射到本地容器方案）；**自定义模板 = 按角色预装环境**（便于审计"这个沙箱能干什么"）；**结构化命令返回值**天然适合回填给 LLM。
- 坑：**云端服务要 API key，与 local-first 冲突**；自托管 microVM 基建成本极高；每命令有网络往返。
- 可用性：**接口借鉴设计**；运行时**不适用**严格 local-first（若接受云依赖，JS SDK 可直接 `npm i e2b`）。

> **本模块结论**：抄 **Open Interpreter 的"确认消息 + 阻塞门 + 拒绝回注"**（这是agent 与人类之间最干净的审批原语）+ **HumanLayer 的 daemon-per-host 拓扑与计划/复查驾驭面**。必须补 OI 缺的那一半：**工具/命令级细粒度白名单 + 拒绝理由回灌**（`auto_run` 这种全局开关不够用，尤其对会写文件的 agent）。

---

### 评估（横切）

**LongMemEval**｜https://github.com/**xiaowu0162**/LongMemEval （`deforum/LongMemEval` **不存在，404**）｜**MIT**｜Python + HuggingFace JSON `xiaowu0162/longmemeval-cleaned`｜push 2026-05-11
- 定位与机制 `[文档]`：500 题、**五能力维度**（信息抽取 / 多会话推理 / 知识更新 / 时序推理 / **拒答 Abstention**）的长期记忆基准；受控管线合成连贯、可扩展、带时间戳的对话历史；被测系统须**边对话边在线记忆**，全部会话结束再统一作答；三种 haystack（`oracle` 只含证据 / `_s` searchable / `_m` answerable）分离"检索难"与"推理难"。
- 可借鉴：**五能力分类学**直接当测试矩阵；**「先在线记忆、后统一提问」的评估协议**（比离线 QA 更贴真实运行方式）；**oracle/searchable/answerable 三变体做误差归因**；基准自身要**版本化与清洗记录**（2025-09 清洗 + 公开变更日志）。
- 坑：只测端到端 QA，记忆写入器需自建；Abstention 类要求"不确定就拒答"；harness 是 Python，TS 侧需重写。
- 可用性：**数据可直接依赖**（MIT JSON 可被 TS 测试加载）+ **协议借鉴**。本模块对 Node/TS 最友好的一项。

**τ-bench**｜https://github.com/sierra-research/tau-bench ｜MIT（**官方已过期**，README 明示 "tasks not updated"）｜Python｜push 2026-03-18（仅指向继任者）；继任 **τ³ / tau2-bench**：https://github.com/sierra-research/tau2-bench （MIT，Python，push 2026-11-10）
- 定位与机制 `[文档]`：LLM 扮演用户 ↔ 代理 ↔ 领域 API 环境三方多轮对话；每任务 = 用户人设 + 初始环境状态 + 领域 API + **政策文档（policy）**；评估看**终态状态比对**（Reward / Achievable Reward）；可靠性指标 **Pass^k**（同任务重复 k 次全成功才算过）。
- 可借鉴：**Pass^k 评"可依赖性"而非单次能力**（自建代理评估应直接采用）；**环境终态比对做 oracle**（比 LLM-judge 更硬、可复现）；**「政策文档即评估规格」**——把权限边界写成可执行检查，是模块 10 与 11 的交叉点。
- 坑：原版任务有已知 bug、官方过期（引用数字前先确认版本）；LLM 模拟用户高方差；每领域要自建 API mock。
- 可用性：**借鉴设计**（Pass^k / 终态比对 / 政策即规格）。

**MemBench（独立基准，非 MemGPT 的 DMR）**｜https://github.com/import-myself/Membench ｜**许可未确认**（README badge 标 MIT，但仓库检测不到 LICENSE 文件）｜Python（数据集在百度网盘/Google Drive）｜push 2025-11-27，论文 arXiv:2506.21605
- 定位与机制 `[文档]`：按「参与/观察 × 事实/反思」四象限评估；**可控噪声注入**（`MakeNoiseMessage*` 约 1k token/单位把会话从 0–10k 拉到 100k+），从而把"记忆能力"与"长上下文硬塞"解耦。
- 可借鉴：**噪声注入的 token 预算化控制**（自建记忆测试成本最低、见效最快的一招）；**参与者 vs 观察者视角二分**（对自己说过 vs 听来的，写入/检索策略应不同）。
- 坑：数据不在仓库里（外链失效风险）；许可未确认；规模小、无第三方复现。
- 可用性：**借鉴设计**（噪声生成方法 + 四象限）。**MemGPT 自带的基准是 DMR，与 MemBench 无关，勿混用。**

**AgentBench**｜https://github.com/THUDM/AgentBench ｜Apache-2.0｜Python｜push 2026-02-08（2025-10 起主分支改为 AgentBench FC，旧多任务版需回退 `v0.1`/`v0.2` tag）
- 定位与机制 `[文档]`：**环境即测试**——Docker Compose 拉起 controller 容器 + 各任务服务容器（OS / DB / KG / WebShop / ALFWorld），代理以 function-calling 风格交互逐任务打分。
- 可借鉴：**「每能力域一个隔离容器 + 中央 controller」的 harness 模式**（评估不污染宿主机，与权限边界配合）；可按需裁剪只取 1–2 个相关域。
- 坑：Docker 依赖极重；OS 任务以 Linux 为中心；新版与 AgentRL 框架深度耦合。
- 可用性：**借鉴设计**（容器化 harness 编排）。

**RAGAS**｜https://github.com/explodinggradients/ragas （API 现解析为 `vibrantlabsai/ragas`）｜Apache-2.0｜**Python，无官方 TypeScript 版本**｜push 2026-11-14
- 定位与机制 `[文档]`：LLM-judge 的 RAG 指标库（faithfulness / answer relevancy / context precision·recall）+ 合成测试集生成；指标是可组合对象、数据集级 `evaluate()`；faithfulness 思路是把答案拆成断言逐条验证是否被上下文支持。
- 可借鉴：**指标即对象、可组合、数据集级 evaluate()** 的 API 形状；**「拆断言逐条验证」** 的打分法（直接借给记忆评估："该召回的有没有召回、召回的是否真被用上"）。
- 坑：Python-only 不能作依赖；LLM-judge 慢贵方差大；只覆盖 RAG 管线，不覆盖代理行为/多轮；指标名正处 churn 期。
- 可用性：**借鉴设计**（指标定义与 API 形状）；运行依赖**不适用**。

> **本模块结论**：唯一能真正吃进 TS 测试的是 **MIT 的 LongMemEval 数据**；方法论上抄 **Pass^k（可依赖性）+ 终态比对（硬 oracle）+ 五能力矩阵 + 噪声注入**。评估必须同时有**成功指标**和**打扰/误报指标**，否则调优会做出"准确但烦人"的 agent。

---

## 借鉴映射（从哪个项目抄哪个具体设计 → 落到我们的哪个模块）

| # | 抄什么（具体设计） | 来源 | 落到我们 |
|---|---|---|---|
| 1 | `MemoryBlock {label, value, limit, description}` + `replace` 需精确匹配旧内容 | Letta | 用户画像：常驻块数据结构 |
| 2 | schema 定义 + 传 existing + 返回同一 id 的改写对象（update-in-place） | LangMem | 用户画像：抽取写入语义 |
| 3 | `{spec, spec_version, data, extensions}` 外壳 + 「MUST NOT 销毁未知键」 | SillyTavern Card V2 | 用户画像：持久化格式的兼容层 |
| 4 | 世界书 `keys+secondary_keys(双键) / constant / insertion_order / priority` | SillyTavern | 上下文装配：条件层注入与预算内淘汰 |
| 5 | insight 的 add / edit / remove 生命周期 + 任务簇检索 | ExpeL | 自我模型：条目不是 summary 而是可维护集合 |
| 6 | `{task, failure_signal, lesson}` 反思缓冲 + 限 top-3 注入 | Reflexion | 自我模型：失败驱动修正 |
| 7 | `workflow{name, steps[], preconditions, source_task_ids}` 归纳复用 | AWM | 自我模型 / 持续学习：程序性自我模型 |
| 8 | 只登记成功 + `{description, usage_count, last_used}` | Voyager | 自我模型：能力边界可信基线 |
| 9 | `Condenser.condense(events) -> View` 可插拔压缩策略 | OpenHands | 上下文装配：装配器接口形状 |
| 10 | 三段预算（固定/滚动摘要/按需）+ 溢出作为显式系统消息 | MemGPT/Letta | 上下文装配：Governor 的预算模型 |
| 11 | `(namespace, key) → {content, kind}` + hot-path 只搜、异步才写 | LangMem | 上下文装配：LOCAL 存储与两档延迟 |
| 12 | `{fact, valid_at, invalid_at, created_at}` 检索按时点过滤 | Zep / Graphiti | 世界模型：失效而非删除（也供画像复用） |
| 13 | `Checkpointer(thread_id, 每步快照)` + `Command(goto, update)` 原子路由 | LangGraph | 策略仲裁：SQLite 快照 + 原子跳转 |
| 14 | `Route{name, utterances[], threshold}` → `{choice, score}`，低分兜底 LLM | Semantic Router | 策略仲裁：Governor 的意图→能力映射 |
| 15 | `(history, candidates[{name, description}]) → speaker` | AutoGen | 策略仲裁：Governor 路由签名 |
| 16 | `expected_output` 验收契约 + `context=[...]` 依赖声明 + guardrail | CrewAI | 策略仲裁 / 主动性：任务完成判定 |
| 17 | `ConceptNode` 字段集 + `depth = 1 + max(filling.depth)` + 相关性/显著性/时效三信号 | Generative Agents | 感知：观察流节点与检索打分 |
| 18 | `transformations = [split, extractMeta, embed]` + 增量缓存 | LlamaIndex | 感知：摄取管线编排 |
| 19 | `{type, text, metadata}` element 契约 | Unstructured | 感知：解析中间件接口 |
| 20 | `(已有能力, 当前状态) → next_goal`，难度随能力自适应 | Voyager | 主动性：目标生成器 |
| 21 | 每轮 token 上限 + 最大循环数 + 成本追踪，超限即停 | AutoGPT | 主动性：硬预算护栏 |
| 22 | 触发与执行解耦（cron/webhook → 一次执行，不常驻轮询） | AutoGPT | 主动性：与 DSH 定时任务(≥60s)契合 |
| 23 | 「何时介入」与「介入什么」分两个决策 | ProactiveAgent | 主动性：提议前的时机门槛 |
| 24 | 子 agent 独立上下文、只回传结论 | Agent Zero | 主动性：上下文隔离手段 |
| 25 | 有向 `(observer, observed)` 关系表征 + Peer Card 压缩 | Honcho | 情感与关系：关系状态的数据模型 |
| 26 | `strategy` 作为一等标签（与情绪两维分离） | ESConv | 情感与关系：验收标注模板 |
| 27 | 人设字段拆分（性格/经历/说话风格/关系） | CharacterGLM | 情感与关系：角色档案字段 |
| 28 | Passage-entity 图上的 PPR 联想检索 + `index_manifest` 身份绑定 | HippoRAG | 世界模型：联想检索 + 索引防错配 |
| 29 | 低层实体 / 高层主题双层检索键 + 增量插入 | LightRAG | 世界模型：检索键设计 |
| 30 | 每条事实带 `derived_from` 回指原文 chunk | GraphRAG | 世界模型 / 记忆：溯源字段 |
| 31 | schema 约束实体类型 + 问题类型驱动检索路径 | KAG | 世界模型：抽取约束（概念层） |
| 32 | `Signature`（类型化 I/O）+ 指标驱动的 prompt 编译 | DSPy | 持续学习：prompt 是可优化对象 |
| 33 | `{variable, feedback, apply()}` 文本梯度接口 | TextGrad | 持续学习：条目精炼 |
| 34 | confirmation 作为一等消息 + 生成器阻塞门 + 拒绝回注 user 消息 | Open Interpreter | 权限边界：审批原语 |
| 35 | daemon-per-host + 中央 API + 类型化契约 + plans/reviews 驾驭面 | HumanLayer | 权限边界：拓扑与审批粒度 |
| 36 | 沙箱生命周期 `create→run→kill` + 结构化 `{stdout,stderr,exitCode}` | E2B | 权限边界：执行边界接口 |
| 37 | 五能力矩阵 + 「先在线记忆后统一提问」 | LongMemEval | 评估：测试矩阵与协议 |
| 38 | Pass^k（可依赖性）+ 终态比对硬 oracle + 政策即规格 | τ-bench | 评估：指标与 oracle |
| 39 | 噪声注入的 token 预算化 + 参与/观察 × 事实/反思 | MemBench | 评估:记忆测试成本控制 |
| 40 | 每域一个隔离容器 + 中央 controller | AgentBench | 评估：harness 编排 |
| 41 | 「拆断言逐条验证」的 faithfulness 打分法 | RAGAS | 评估：召回是否被用上 |

---

## 不要抄的（每模块至少一条反面模式）

| 模块 | 反面模式（不要抄） | 为什么 |
|---|---|---|
| 用户画像 | LangMem 的 `enable_inserts=True` 式自由 append 事实 | 会产生重复与互相矛盾的画像条目；必须 update-in-place |
| 用户画像 | SillyTavern 的 `first_mes` / `mes_example` 字段 | RP 场景假设，对话式个人 agent 不需要，照搬只会塞满预算 |
| 自我模型 | 把自省做成"每次结束生成一段 summary 塞进 system prompt" | 无生命周期、无淘汰、无回滚；一次错误总结会长期污染 |
| 自我模型 | Voyager 式"技能库只增不减" | 规模上去后检索噪声压过有效信号；必须带淘汰/合并 |
| 上下文装配 | LLMLingua 式 token 级压缩（需 torch 小模型） | 与本栈硬冲突；且会破坏代码/JSON，最易丢否定词与数字 |
| 上下文装配 | GraphRAG 式离线全量社区聚类做实时装配 | 每次新增都可能触发全局重算，个人高频上下文不可行 |
| 策略仲裁 | AutoGen `GroupChat` 把全量历史塞进选人 prompt | token 随轮数近似平方增长；应只给候选 description |
| 策略仲裁 | RouteLLM 式只做模型选择 | 与本项目"能力/工具选择"目标只部分重叠；且项目已停更 |
| 感知 | Unstructured 式默认走 OCR/布局模型重链路 | 单用户本地摄取 Markdown/HTML 不需要，启动成本高 |
| 感知 | Generative Agents 式每轮对每条观察都调 LLM 打 poignancy | 成本随对话线性上涨；应批量化/规则预筛 |
| 主动性 | BabyAGI 式"每轮都重排任务列表" | 无新信息也烧 2–3 次 LLM 调用；再规划必须由事件触发 |
| 主动性 | AutoGPT Classic 式无预算自主循环 | 经典病症：重复命令、目标漂移、空转烧钱 |
| 情感与关系 | 把关系写成一段不断变长的自然语言 summary | 无法计算、无法淘汰、无法对账；应是带 priority 的条目集合 + 数值慢变量 |
| 情感与关系 | 把 ESConv / EmpatheticDialogues 数据当产品语料 | 两者许可分别为 CC-BY-NC-4.0 与 NOASSERTION，不可进产品 |
| 世界模型 | 为单用户引入 Neo4j/FalkorDB 类图数据库 | 与"单二进制 local-first"冲突；单用户量级不需要图库 |
| 世界模型 | GraphRAG 式把摘要当唯一检索入口 | 摘要必有损且不可回指；必须保留 `derived_from` 原文指针 |
| 持续学习 | DSPy/TextGrad 式在线每步做优化 | 需大量样例 + 多次调用；应离线、按指标、可回滚 |
| 持续学习 | 无回滚的策略替换（直接覆盖旧 prompt 模板） | 学坏了无法恢复；规则文件必须版本化 + 一键回退 |
| 权限边界 | Open Interpreter 的 `auto_run` 全局开关 | 粒度太粗；必须有工具/命令级白名单 + 逐次确认 |
| 权限边界 | E2B 式把执行默认放到云端 | 与 local-first、不外发数据直接冲突 |
| 评估 | τ-bench 原版任务数字（官方已明示过期） | 引用前必须确认版本；用 tau2-bench 或标注版本 |
| 评估 | 只报单次成功率 | 会掩盖不稳定；代理评估必须报 Pass^k |
| 评估 | 用 LLM-judge 替代终态比对 | 贵、慢、方差大；能比对状态就别让模型打分 |

---

## 对候选清单的修正（按严重程度）

**1. 仓库坐标/归属错误（硬错，必须改）**
- 用户画像 `Memobase` → **`memodb-io/memobase`**（`memonotrend/memobase` 不存在）。
- 自我模型 `ExpeL` → **`LeapLabTHU/ExpeL`**（`leap-llm/ExpeL` 404）。
- 自我模型 `Agent Workflow Memory` → **`zorazrw/agent-workflow-memory`**（`run-llama/...` 404）。
- 世界模型 `MemGPT` → **`letta-ai/letta`**（`cpacker/MemGPT` 已重定向；旧 V1 在 `archive` 分支）。
- 主动性 `AgentZero` → **`agent0ai/agent-zero`**（`frdel/...` 已重定向）。
- 主动性 `ProactiveAgent` → **`thunlp/ProactiveAgent`**（`aitraveler/ProactiveAgent` 404）。
- 主动性 `OpenHands` → **`OpenHands/OpenHands`**（`All-Hands-AI/...` 已重定向）。
- 情感 `CharacterGLM` → **`thu-coai/CharacterGLM-6B`**（`THUDM/...` 404）。
- 评估 `LoCoMo` → **`snap-research/locomo`**（非 Microsoft；且**许可 CC BY-NC 4.0，禁商用**，只能借 schema 不能借数据）。
- 评估 `LongMemEval` → **`xiaowu0162/LongMemEval`**（`deforum/...` 404）。
- 权限 `Open Interpreter` → **`openinterpreter/openinterpreter`**（已转 Rust；经典确认流只在 `v0.4.2`）。

**2. 情感与关系模块的候选大半不是"参考实现"**
`ESConv`（数据在 HuggingFace，官方 GitHub 代码**无法确认**，许可 CC-BY-NC-4.0 禁商用）、`EmpatheticDialogues`（**已归档**，push 2021，许可 NOASSERTION）、`CharacterGLM`（**6B 权重模型**，需 GPU + Python 推理栈）——三者都不是本项目能借鉴的"可运行工程实现"。建议改为：**Honcho（有向关系表征）+ SillyTavern 世界书（条件注入与淘汰）** 作为实现对标；把 ESConv 的「策略」标签降级为**评测标注参考**，并明确"两数据集不进产品语料"。

**3. Honcho 是"竞品/可选依赖"，不是普通对标项**
Honcho 已同时出现在「用户画像」与「情感与关系」两个候选里，且它**官方公开了 npm 包 `@honcho-ai/dsh-honcho`（DSH 插件）与 `@honcho-ai/sdk`（TypeScript SDK）**，与本项目的画像/关系模块功能正面重叠，许可还是 **AGPL-3.0**。候选清单应把它从"抄它"重新定位为**需要先做的决策题**：划清边界后二选一（自研画像 ← 借它设计；或直接用其 TS 后端 ← 承担 AGPL + Postgres）。这比多列一个对标项目重要。

**次级修正（不改会误导）**
- 「上下文装配」里的 `GraphRAG` 属世界模型，`LLMLingua` 是"不适用"反例——本模块真正的首选应是 **OpenHands Condenser + MemGPT 三段预算**；`GraphRAG` 移到世界模型。
- 「策略仲裁」`RouteLLM` 停更近两年且只做模型选择，首选应换 **LangGraph supervisor + Semantic Router**。
- 「评估」`MemBench` 是与 MemGPT 无关的独立基准（arXiv 2506.21605），MemGPT 自带基准是 **DMR**，勿混称；`τ-bench` 官方已过期，指向 **tau2-bench**。
- 用户画像与情感模块重复列 `SillyTavern` 与 `Honcho`——应按"模块视角"引用同一仓库，不要当成两个不同项目重复计数。

---

## 事实缺口（未确认，不得引用为已核实）

- **AutoGen 代码许可**：GitHub API 仅报 CC-BY-4.0（疑为文档许可），仓内 LICENSE 需人工核对。
- **BabyAGI 许可**：README 称 MIT，但仓库未识别出 LICENSE 文件。
- **MemBench 许可**：README badge 标 MIT，但仓库检测不到 LICENSE 文件。
- **EmpatheticDialogues 许可**：NOASSERTION，实际条款未读原文。
- **CharacterGLM release**：`/releases/latest` 未确认。
- **ESConv 官方 GitHub 代码仓库**：无法确认；可确认的只有 HuggingFace 数据集 `thu-coai/esconv`。
- **LangGraph.js / LlamaIndexTS / Semantic Router 的 TS 移植**：与 Python 版的能力对等度未核实。
- **Letta `letta-code` 的能力边界**：它提供 TS Agent SDK，但是完整 harness；"能否只取记忆块部分嵌入"未核实。
- 本文所有 **star 数一律未写**——为避免过时数字被当结论；需要量级时请以当日 `gh api` 自行取数。

> **下一步**：本清单覆盖 12 个模块、约 50 个对标项，满足「每模块 3–5 个」的规矩。记忆模块另有 `agent-memory-systems.md`。任一模块进入 spec 前，应先就「首选 + 借鉴映射中的 1–2 条」写出接口草图（TS 类型 + zod schema），再落设计文档。
