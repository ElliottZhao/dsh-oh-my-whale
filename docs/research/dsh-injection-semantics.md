# DSH 上下文装配与注入语义（一手源码核实）

> 范围：为设计"宿主内 Adapter 每轮注入什么、何时注入、如何避免乱插话"而做的一手核查。
> 证据：DSH 0.2.2-rc.3 官方文档 `docs/subsystems/` 与对应 `src`。本文只记录已核实事实；未确认一律标「未确认」。

## 1. 会话真相源：append-only 事件日志，模型历史是从中派生的

`docs/subsystems/session.md:5`：「A `Session` is an **append-only log** of typed `SessionEvent`s — the single source of truth. The LLM message history is *derived* from the log, never stored separately。」

- 派生函数 `Session.deriveMessages()`（session.md:685）：按 surface 事件序派生模型看到的 `Message[]`。**缓存**：纯尾部增长 O(新节点)；surface 重写才重建。
- **surface** 是派生的唯一事实源（session.md:669）：「every message-producing append records its `surfaceOp` … a compaction `replace` deletes the shadowed nodes from the derivation」。
- 关键推论：**插件事后摄入应该读 `session/event` 日志，而不是读模型的 messages 数组**。模型看到的是派生视图，日志才是全量无损真相。

## 2. 哪些事件投影成模型消息（SurfaceEventType）

`SurfaceEventType`（session.md:311-317）= `system/message`、`developer/message`、`user/message`、`assistant/message`、`tool/result`。**只有这五种产生模型消息**。

- `user/message`（session.md:49-55）：user-role 消息。三种来源：直接人类输入、**合成 `agent.inject()` 上下文**（文件变更通知、AGENTS.md、技能内容、cron 通知等）、goal 续跑。三者都逐字投影 `content`，靠 `source` 区分。
- 非 user source 的 `user/message` 注入（session.md:703）：投影为 user-role 消息、在其时间位置逐字呈现，`source` 命名生产方。
- `assistant/message`：完整模型流；`tool/result`：tool-role 结果。
- 其余（`turn/*`、`step/*`、`assistant/attempt`、`request/header`、`request/context`、`compaction/*`、`hook/*`、`session/end-seed`）都是**log-only**，不投影成消息。

## 3. 系统提示是 surface node 0，不是 header 的一部分

session.md:185：「The rendered system prompt is not part of the header: it is derived history, the `system/message` event at surface node 0」；`EpochHeader.system` 字段被 `@persistenceReserved` 禁止（session.md:201-204）。

- 系统提示变更走「替换或追加一个 system node」。
- `request/header` 的 `systemPromptUpdate`（`'in-history'` 等，session.md:223-224）决定：**变更后的系统提示是替换最新 system node，还是在缓存历史之后追加**。这是"每轮注入"是否累积的关键机制。

## 4. 压缩（compaction）——模型视野何时被裁剪

`docs/subsystems/compaction.md`：

- 压缩是**可选能力**，不是 loop 主干（compaction.md:5）。
- 三个 log-only 事件 `compaction/start|summary|end`（compaction.md:13-17），**不投影成消息**。
- 摘要本身通过**一个带 `surfaceOp: {op:'replace', startSeq, endSeq}` 的 `user/message`** 进入模型历史（compaction.md:11）——这是摘要压缩执行的唯一 surface 变更。**替换即删除被遮蔽节点**。
- 触发：`agent/pre-step`（压力）与 `agent/request`（改目的地的 context overflow）（compaction.md:101）。
- 推论：**被压缩掉的节点仍在日志里（shadowedSeqs 记录），但不再出现在派生视图**。插件若在压缩**之后**读 `deriveMessages()`，会读不到已压缩内容；要拿全量历史必须读日志原文。

## 5. 客户端 Assembly（conversation.md）与模型视野无关

`conversation.md` 讲的是 **Web Client 渲染层**（`ui-conversation` → Chat/Trajectory 视图），是"人类看到的对话"如何装配，不是"模型看到的输入"。不要把它误当模型上下文装配。

## 6. 注入纪律（由以上事实推出的可执行规则）

1. **注入走 `user/message`（带非 user `source`）或 system section，绝不自己伪造 assistant 消息**。只有五种 surface 事件能进模型历史。
2. **事后摄入读 `session/event` 日志，不读 `deriveMessages()`**——后者是经压缩裁剪的派生视图，前者才是无损真相。
3. **在压缩后读历史要意识到 `replace` 已删除被遮蔽节点**；要"压缩前的完整历史"只能回日志。
4. **每轮注入是否累积，取决于宿主的 `systemPromptUpdate` 路由**——不是我们自由追加后自己清理；要先查清楚这条路由再决定注入位置，否则会无限累积。
5. **主动"插话"只有经 `agent.inject()` 成为 `user/message` 一条路进入模型视野**；任何其它途径（写 UI、改日志）都不会被模型看见。

## 未确认

- `systemPromptUpdate` 各路由（`'in-history'` 之外还有哪些）的确切行为与默认值。
- `agent/inject` 是否有节流/去重/每 turn 条数上限（文档未提，需看 `dsh-agent/src/types.ts` 的 inbox 事件实现）。
- loop 驱动对"turn 内多次 inject"是追加还是替换（`agent-lifecycle.md` 显示经「下一次 claim」进入 pre-step，但多次 inject 的合并规则未见明述）。