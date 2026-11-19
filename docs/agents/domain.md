# 领域文档

工程技能在探索代码库时，应当如何消费本仓库的领域文档。

## 探索之前先读这些

- 仓库根目录的 **`GLOSSARY.md`**，或
- 仓库根目录的 **`GLOSSARY-MAP.md`**（若存在）：它指向每个上下文的 `GLOSSARY.md`，按主题读取相关的那几份。
- **`docs/adr/`**：读取涉及你即将改动区域的 ADR。多上下文仓库里，还要看 `src/<context>/docs/adr/` 下的上下文级决策。

这些文件若不存在，**静默继续**。不要提示缺失，也不要提前建议创建。`/domain-modeling` 技能（经由 `/grill-with-docs` 与 `/improve-codebase-architecture` 到达）会在术语或决策真正定下来时按需创建它们。

## 文件结构

单上下文仓库（多数仓库）：

```
/
├── GLOSSARY.md
├── docs/adr/
│   ├── 0001-event-sourced-orders.md
│   └── 0002-postgres-for-write-model.md
└── src/
```

多上下文仓库（根目录存在 `GLOSSARY-MAP.md`）：

```
/
├── GLOSSARY-MAP.md
├── docs/adr/                          ← 系统级决策
└── src/
    ├── ordering/
    │   ├── GLOSSARY.md
    │   └── docs/adr/                  ← 上下文级决策
    └── billing/
        ├── GLOSSARY.md
        └── docs/adr/
```

## 使用术语表里的词汇

当你的输出提到某个领域概念（issue 标题、重构提案、假设、测试用例名）时，使用 `GLOSSARY.md` 里定义的术语。不要漂移到术语表明确回避的同义词。

如果需要用的概念还不在术语表里，这是个信号：要么你在自造项目不用的语言（应重新考虑），要么存在真实缺口（记下来交给 `/domain-modeling`）。

## 标出 ADR 冲突

如果你的输出与已有 ADR 矛盾，要显式指出，而不是悄悄覆盖：

> _与 ADR-0007（事件溯源订单）矛盾，但值得重新讨论，因为……_
