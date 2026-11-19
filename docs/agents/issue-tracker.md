# 议题跟踪：GitHub

本仓库的议题（issue）与规格（spec）都以 GitHub issue 形式存放。所有操作统一使用 `gh` CLI。

## 约定

- **创建 issue**：`gh issue create --title "..." --body "..."`。多行正文用 heredoc。
- **读取 issue**：`gh issue view <number> --json number,title,body,labels,comments`。
- **列出 issue**：`gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`，按需加 `--label`、`--state` 过滤。
- **把 issue 设为父 issue 的子 issue**：`gh issue create --parent <parent> ...`，或事后执行 `gh issue edit <parent> --add-sub-issue <child>`（`gh` 2.94+）。较老版本的 `gh`：`gh api --method POST repos/<owner>/<repo>/issues/<parent>/sub_issues -F sub_issue_id=<child-db-id>`（传数据库 id，见下方**阻塞关系**）。不支持子 issue 时，在子 issue 正文顶部写 `Part of #<parent>`。
- **阻塞关系**：使用 GitHub 的**原生 issue 依赖**——规范且 UI 可见的表达方式。加边用 `gh issue create --blocked-by <n>,<n> ...` 或 `gh issue edit <child> --add-blocked-by <blocker>`（`gh` 2.94+）。较老版本的 `gh`：`gh api --method POST repos/<owner>/<repo>/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-db-id>`，其中 `<blocker-db-id>` 是阻塞方 issue 的数字**数据库 id**（`gh api repos/<owner>/<repo>/issues/<n> --jq .id`，**不是** `#number`，也不是 `node_id`）。依赖功能不可用时，退回在子 issue 正文顶部写 `Blocked by: #<n>, #<n>`。所有阻塞项都关闭，票才算解阻。
- **统计未关闭的阻塞项**：`gh api repos/<owner>/<repo>/issues/<n> --jq .issue_dependencies_summary.blocked_by`（只算未关闭阻塞项；为 0 表示可以开工）。`gh issue view <n> --json blockedBy` 也会列出已关闭的阻塞项，需过滤：`--jq '[.blockedBy.nodes[] | select(.state == "OPEN")] | length'`。
- **评论 issue**：`gh issue comment <number> --body "..."`
- **添加 / 移除标签**：`gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **关闭**：先用 `gh issue comment <number> --body "..."` 写明原因，再 `gh issue close <number>`。对已关闭的 issue 使用 `gh issue close --comment` 会丢掉评论，但仍返回 0。

在 clone 内运行 `gh` 时，它会自动从 `git remote -v` 推断仓库。

只往用户自己的仓库提 issue。除非用户拥有 `mattpocock/skills`（这些技能来源的仓库），否则绝不往那里提。在 fork 中 `gh` 可能默认指向 upstream 父仓库，因此每次会话首次 `gh issue create` 之前，先用 `gh repo view --json nameWithOwner` 确认目标；若不属于用户本人，先向用户确认。

## 把 PR 作为 triage 入口

**PRs as a request surface: no.**（PR 是否作为请求入口：否。）_(若本仓库把外部 PR 当作功能请求，改成 `yes`；`/triage` 会读取这个开关。)_

设为 `yes` 时，PR 走与 issue 相同的标签与状态，使用对应的 `gh pr` 命令：

- **读取 PR**：`gh pr view <number>`，再 `gh pr view <number> --comments`（非 TTY 下 `--comments` 只打印评论，不打印正文），diff 用 `gh pr diff <number>`。
- **列出待 triage 的外部 PR**：`gh api --paginate 'repos/{owner}/{repo}/pulls?state=open' --jq '.[] | select(.author_association | IN("OWNER","MEMBER","COLLABORATOR") | not) | {number, title, author: .user.login, author_association, labels: [.labels[].name]}'`。
- **评论 / 打标签 / 关闭**：`gh pr comment`、`gh pr edit --add-label`/`--remove-label`、`gh pr close`。

GitHub 的 issue 与 PR 共享同一编号空间，所以裸 `#42` 可能是两者之一：先用 `gh pr view 42` 解析，失败再退回 `gh issue view 42`。

## 当技能说 "publish to the issue tracker" 时

创建一条 GitHub issue。

## 当技能说 "fetch the relevant ticket" 时

按上文**读取 issue** 的方式读取。

## Wayfinding 操作

供 `/wayfinder` 使用。**地图（map）** 是一条 issue，**子 issue** 是票（ticket）。

- **地图**：单条 issue，打 `wayfinder:map` 标签，正文存放 Notes / Decisions-so-far / Fog 三段。`gh issue create --label wayfinder:map`。
- **子票**：作为地图的子 issue 关联（见**把 issue 设为父 issue 的子 issue**）。子 issue 功能不可用时，把子票加进地图正文的任务清单，并在子票正文顶部写 `Part of #<map>`。标签用 `wayfinder:<type>`（`research`/`prototype`/`grilling`/`task`）。被认领后指派给执行者。
- **阻塞**：见上文**阻塞关系**。
- **前沿查询（frontier query）**：列出地图所有未关闭子 issue（`gh issue list --state open`，范围限于地图的子 issue / 任务清单），剔除存在未关闭阻塞项（见**统计未关闭的阻塞项**，或 `Blocked by` 行里仍有未关闭 issue）的，以及已有指派的；地图顺序最靠前者胜出。
- **认领**：`gh issue edit <n> --add-assignee @me`，这是本会话的第一次写操作。
- **解决**：`gh issue comment <n> --body "<answer>"`，然后 `gh issue close <n>`，再把上下文指针（gist + 链接）追加到地图的 Decisions-so-far 段。
