# oh-my-whale

一个一对一 Personal Agent：宿主内的薄适配器，加一个本机常驻内核。本文件定义这个项目的领域语言；代码标识符、issue 标题与设计文档一律使用这里的规范名。

## Language

### 系统形态

**Core**:
本机常驻、与宿主无关的内核进程，持有 Memory、Profile、Soul、Relationship 等状态并提供检索。
_Avoid_: 服务端、后端、server、daemon

**Adapter**:
运行在宿主进程内、把宿主接缝与 Core 连起来的薄适配层。
_Avoid_: 客户端、插件、client

**Host**:
运行 Agent 会话的外部程序（如 DeepSeek Harness），Adapter 依附于它。
_Avoid_: 平台、运行时

**Protocol**:
Adapter 与 Core 之间的 wire 契约，带主版本号；主版本不同即拒绝沟通。契约类型只在 Core 的 Rust 代码里定义一次，Adapter 的类型与校验器由它生成。
_Avoid_: API、接口层、RPC

### 模块

**Perception**:
把宿主与外部信号转成可存储的 Observation。Memory 的唯一入口。
_Avoid_: 摄取、采集、Sensor、ingestion

**Memory**:
跨会话保存的、带时间有效性的记忆条目集合。
_Avoid_: 存储、知识库、向量库、database

**Profile**:
对用户的可计算描述，由有上限、可原地更新的具名块组成。
_Avoid_: 用户档案、user model、用户画像

**Soul**:
Agent 对自身的可编辑、可审计描述，由稳定的身份块与从经验提炼的 Insight 组成。
_Avoid_: 人格、人设、persona、self-model

**Consolidation**:
睡眠期对记忆条目的合并、失效与淘汰。
_Avoid_: 整理、清理、GC、sleep

**Relationship**:
Agent 与用户之间的有向关系状态与情感建模。
_Avoid_: Humanization、情感模块、亲和度

**Composer**:
系统里唯一决定「这一轮注入什么」的地方：按预算把 Memory、Profile、Soul、Relationship 组装成 View。不存在第二个模块参与这个决定。
_Avoid_: 上下文管理、prompt 拼接、context assembler、Governor、调度器、仲裁者

**Guard**:
对会改变外部世界或自身状态的动作做审批与拦截。
_Avoid_: 沙箱、权限、安全模块、sandbox

**Ledger**:
记录每条结论与每次变更的来源与版本，使改动可追溯、可回滚。
_Avoid_: 日志、审计、历史、log

**Evaluation**:
对模块行为做可判定回归的验收集合。
_Avoid_: 测试、评测、benchmark、eval

**Onboarding**:
在没有任何历史时建立初始 Profile 与 Soul 的过程。
_Avoid_: 冷启动、初始化、setup

### 模型与配置

**Provider**:
Core 配置里登记的一个模型服务端点，带地址、密钥引用与它声明的能力集合。Core 的出站白名单就是这张表。
_Avoid_: 模型供应商、服务商配置

**Credential Ref**:
凭据的引用方式：环境变量名或 Core 自己的密钥文件路径。配置里只存引用，不存密钥值。
_Avoid_: 密钥、token、api key

**Model Capability**:
provider 能提供的模型能力类别，按能力而非业务划分：LLM、Embedding、CrossEncoder、Decision。同一个 provider 可声明多种能力。

**Capability Role**:
Core 内部对某类工作的具名需求（抽取、查询改写、排序、弃权判定等），绑定到某 provider 的某条能力。能力角色不是模型，也不等于业务场景。

### 核心概念

**Observation**:
一条带时间与来源的单次信号，例如一句话、一次工具调用、一次文件变更。

**Memory Item**:
一条独立生命周期的事实或情节，带有效区间与自证的证据摘录。原文可能已不存在，所以证据必须内嵌，不能只存指针。

**Insight**:
从经验中提炼、可增删改的一条规则，属 Soul。

**View**:
Composer 产出的、受预算上限约束的上下文快照。

**World Model**:
记忆之上的一层世界状态视图；其 v1 用法尚未确定，暂不作为模块。
