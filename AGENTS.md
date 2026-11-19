# oh-my-whale 的 Agent 指南

本项目的工程流程以 **Matt Pocock Skills** 为唯一事实源（SSOT）。流程怎么走、何时要用户确认、产物放在哪，由技能本体规定。本文件负责四件事：把常见场景路由到对应技能、列出技能未覆盖的本地补充、说明没有技能时的做法、记录本仓库约定。

技能已规定的内容只在技能本体维护；本文件与 `docs/rules/` 发现重复就删本地条款，发现冲突以技能为准。

## 技能路由

开始任务前先按场景选技能。同一任务可以串联多个技能，先后以技能本体为准。「用户运行」的技能由用户输入命令；「模型可调用」的技能在需要时直接调用。

| 场景 | 技能 | 调用方式 |
|---|---|---|
| 不确定该走哪个流程 | `ask-matt` | 用户运行 |
| 压测想法、计划或设计树 | `grill-with-docs`（有工作目录）；`grill-me`（无工作目录） | 用户运行 |
| 只要访谈本身，不要外层流程 | `grilling` | 模型可调用 |
| 需要跑起来才能回答的设计问题 | `prototype` | 模型可调用 |
| 多会话构建 | `to-spec` → `to-tickets` → `implement`（逐票）或 `implement-spec`（整份） | 用户运行 |
| 单会话实现 | `implement` | 用户运行 |
| 测试驱动或回归测试 | `tdd` | 模型可调用 |
| 合并前评审 | `code-review` | 模型可调用 |
| 会话复盘、改进代理环境 | `retro` | 用户运行 |
| 议题分诊 | `triage` | 用户运行 |
| 难缺陷、间歇性失败、性能回归 | `diagnosing-bugs` | 模型可调用 |
| 大型模糊工程、路径看不清 | `wayfinder` | 用户运行 |
| 架构摩擦、模块深化 | `improve-codebase-architecture`（用户运行）+ `codebase-design`（模型可调用） | 混合 |
| 领域术语、`GLOSSARY.md`、ADR | `domain-modeling` | 模型可调用 |
| 需要一手来源的调研 | `research` | 模型可调用 |
| 写 PR 正文 | `pr` | 模型可调用 |
| 跨宿主、跨目录或交给同事 | `handoff` | 用户运行 |
| 消息没讲清、要重讲 | `wait-what` | 用户运行 |
| 写或改技能、`AGENTS.md` | `writing-for-agents` | 模型可调用 |

流程取舍（要不要先 grill、要不要出规格与票据、走 triage 还是 wayfinder）由 `ask-matt` 判断。议题跟踪、分诊标签、领域文档布局已经按技能的前置要求配置好，见下文。

## 本仓库补充

技能没有覆盖的细节放在 `docs/rules/`，只在对应场景加载。

| 场景 | 规则 |
|---|---|
| 没有规格或票据的小任务 | `docs/rules/plan.md` |
| 模块接口已定、要落地实现 | `docs/rules/design-first.md` |
| 非缺陷的同一修复反复失败 | `docs/rules/unstick.md` |
| 长任务的决策账本 | `docs/rules/ledger.md` |
| 可运行产物的多版本取优 | `docs/rules/best-of-n.md` |
| 宽而浅的并行审计 | `docs/rules/swarm.md` |
| 高风险改动的影响半径取证 | `docs/rules/blast-radius.md` |
| 正式界面的视觉与验收标准 | `docs/rules/design.md` |
| 现有界面的证据化审查 | `docs/rules/design-audit.md` |
| 解释一块代码 | `docs/rules/explain.md` |
| 用户离开很久后回来 | `docs/rules/catchup.md` |
| PR 生命周期：标题、草稿、基线、脱敏、gh 兜底 | `docs/rules/pr.md` |
| PR 评论归属格式 | `docs/rules/pr-comments.md` |
| 盯 PR 的持续集成与评审 | `docs/rules/watch-pr.md` |
| 从代码到合并 | `docs/rules/ship.md` |
| 停在合并前 | `docs/rules/ready.md` |
| 接手他人的 PR、分支或线程 | `docs/rules/adopt.md` |
| 复活陈旧 PR | `docs/rules/revive.md` |
| 中文写作风格 | `docs/rules/unslop.md` |
| 技能未覆盖的工程判断 | `docs/rules/principles.md` |

## 没有启用技能时

单文件、小改动、没有流程可依时，按四条走：

- 一行复述目标与最小改动。
- 先读要改的代码与调用方。
- 做最小改动，每改一次跑一条真实命令。
- 报告给命令与输出，标明实测、推断或猜测。

## 项目约定

### 议题与标签

议题以 GitHub Issues 形式存放在 `ElliottZhao/dsh-oh-my-whale`，通过 `gh` 操作。标签沿用五角色名：`needs-triage`、`needs-info`、`ready-for-agent`、`ready-for-human`、`wontfix`。详见 `docs/agents/issue-tracker.md` 与 `docs/agents/triage-labels.md`。

### 领域文档

单上下文布局：仓库根目录 `GLOSSARY.md` 与 `docs/adr/`。详见 `docs/agents/domain.md`。术语与 ADR 由 `domain-modeling` 维护。

### 文档语言

文档、代码注释、提交信息与 PR 正文使用简体中文。代码标识符、命令、标签名与外部模板的段名保持原样。

### 文档隐私边界

本仓库公开在 GitHub，面向外部开发者。文档、代码与提交信息中：

- **不得**提及维护者本机的环境细节：本机安装的其它插件与工具包、内部试验性仓库、本机配置路径、个人目录或凭据。
- 工程事实一律引用公开来源（如 DSH 官方文档），或写成与具体环境无关的结论。
- 只服务于本机、不应进入公开仓库的工作笔记，放在仓库**之外**的目录（不提交）。

### 设计与调研

每个模块在进入设计前应列出 3–5 个开源对标项目（附仓库 URL），并在 `docs/research/` 留下调研结论。流程要求先出规格时，把对标调研列进规格的前置任务或补充说明，不阻塞流程。
