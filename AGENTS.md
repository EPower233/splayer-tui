# AGENTS.md

本仓库里所有 coding agent 都要遵守的规则。Claude Code 通过 `CLAUDE.md` 读这份文件，其他 agent 直接读。这里的规则和 agent 工具的默认行为冲突时，以这里为准。

## 项目

SPlayer-Next 的终端版（TUI）：在终端里跑的音乐播放器，布局、按钮、默认值、功能都照 [SPlayer-Next](https://github.com/SPlayer-Dev/SPlayer-Next)，长相和交互是 TUI 的。长期目标是功能和 SPlayer-Next 对齐，包括多个在线平台、流媒体服务器、本地曲库和插件。用 Rust 独立实现，模块和类型对齐 SPlayer-Next，内核库写成 SPlayer-Next 也能调用的样子（构法 D，见 `docs/scope.md`、`docs/adr/0002-approach-d.md`）。

本文件里说的 "SPlayer" 一律指 SPlayer-Next。产品细节见 `docs/spec/product.md`。

## 开工前先读

1. 你的任务卡 `docs/tasks/T-<编号>.md`（用户会点名）：目标、验收、范围、上一次交接。
2. `PROGRESS.md`：项目现在在哪一步、已经定了什么、还在等谁定什么。
3. `docs/architecture.md`：代码放哪。
4. 任务碰到哪块，就读哪份设计文档：界面看 `docs/ui-direction.md`，键位看 `docs/keymap.md`，设置看 `docs/settings.md`，性能看 `docs/plan.md` 4.5 节。
5. `git status`、`git log --oneline -10`，以及你所在分支的 CI 状态。

只读任务碰到的部分，不要通读整个仓库。

## 产品上的硬规矩

这些都是用户在设计阶段明确定下的，违反就是返工：

- **不加用户没要过的功能。** 想加什么，先在汇报里提出来，等用户点头。（前车之鉴：设计阶段我自作主张加过流动背景，被否掉了。）
- **界面照 SPlayer-Next。** 改任何一个按钮栏之前，先读 SPlayer-Next 对应的组件源码（`NavHeader`、`PlayerBar`、`FullPlayer` 下的 `PlayerTopBar` / `PlayerBottomBar` / `LyricActions` 等），按钮的位置和顺序照它来。
- **默认值照 SPlayer-Next。** 终端版新增的设置除外，见 `docs/settings.md`。
- **界面结构不靠底色，也不用框。** 线只用来分组，线和线不相交；按键提示只写在它控制的按钮旁边，没有底部提示栏，也不为了放按键另造按钮栏（见 `docs/ui-direction.md`）。
- **键位按 `docs/keymap.md`**，每个窗口都要过按键冲突测试。

## 架构（`scripts/check.sh` 会检查）

Cargo workspace，一层一个 crate。依赖只能往里指，由 `scripts/arch-check.sh` 检查每个 crate 的 `Cargo.toml`：

| crate | 放什么 | 只能依赖 |
|---|---|---|
| `splayer-domain` | 数据类型（对齐 SPlayer-Next 的 `shared/types`）和纯规则：队列、播放模式、歌词解析、当前行 / 当前字计算、封面取色。没有 I/O、没有异步运行时、不读时钟、不用随机数（需要的话由调用方传进来） | 无 |
| `splayer-app` | 播放器大脑：接收命令、发出事件、编排用例。对外部世界的需求都写成 `ports` 里的 trait：音乐来源、音频输出、系统媒体控制、本地存储 | `domain` |
| `splayer-audio`、`splayer-source-*`、`splayer-mediactl`、`splayer-store` | 适配器：实现 `app::ports` 里的 trait | `domain`、`app` |
| `splayer-tui` | 终端界面：只读大脑给的快照来画，把按键翻译成命令；`终端能力` 结构也在这里 | `domain`、`app` |
| `splayer-tui-bin`（可执行文件 `splayer-tui`） | 唯一知道具体用哪些适配器的地方：读配置，把适配器接进大脑，再接上界面 | 全部 |

- 只在边界上写 trait（`app::ports` 里那几个，或者测试必须替换的地方）。一层内部只有一个实现的东西，不写 trait。
- 一个文件一件事。函数里混了不同层次的逻辑，或者一屏放不下，就拆。
- 新加一个 crate、一个重量级依赖、或者跨层调用，先写 ADR（`docs/adr/`），用户同意后再动手。
- **对齐 SPlayer-Next（构法 D）**：
  - 照着 SPlayer-Next 写的文件，开头注释写明 `对应 SPlayer-Next：<路径> @ <提交号>`。上游改了这个文件，就知道要回来同步。
  - `domain` 里和 SPlayer-Next 对应的类型，序列化后的字段名和 SPlayer-Next 一致（`serde(rename_all = "camelCase")`）。
  - `domain`、`app` 和各适配器 crate 不依赖任何终端库（ratatui、crossterm），接口只用普通的结构体、枚举和函数，这样 SPlayer-Next 将来能加一层 NAPI 直接调用。
- **复制别的项目的代码**：只复制许可证能并进 AGPL-3.0 的；在 `THIRD_PARTY.md` 登记来源仓库、提交号、许可证；文件开头注明来源。没有许可证文件的仓库只能看，不能复制。

## 命令

- 初次设置（每个 clone 一次）：`bash scripts/setup-repo.sh`
- 一条命令验证全部（CI 跑的就是它）：`bash scripts/check.sh`
- 写代码时的快速循环：`cargo test -p <crate>`
- 本地运行：`cargo run -p splayer-tui-bin`
- 重任务（release 构建、带 FFmpeg 的构建、基准测试）一次只跑一个：`bash scripts/heavy.sh <命令>`，它会排队拿锁。

在 Windows 上，以上脚本都用 Git Bash 跑。

## 分支和提交（`.githooks/` 里的钩子会强制其中大部分）

- `main` 永远可以发布。不要往 `main` 提交或推送，它只通过 squash 合并的 PR 改动，CI 要绿，由用户合并。（例外：仓库刚建时、第一次推送之前的几个初始化提交，是直接提交在 `main` 上的，照 `git-setup.md` 的流程。）
- 不要自己开分支（`git branch`、`git switch -c`、`git checkout -b`），只用 `scripts/new-task.sh`。它会记下分支从哪来、为哪个任务、谁开的，并开一个草稿 PR。需要一个还不存在的分支，问用户。
- 只在任务卡指定的 worktree 和分支上干活。一个 worktree 只有一个写入者。
- 每个提交都带 `Task:` 和 `Agent:` 尾注，钩子会从分支名和 worktree 里的 `.agent` 文件自动加上，不要删。任务换人时，由用户改 `.agent`。
- **原子化提交：做完一个子任务就提交一个**，提交里包含这个子任务的代码、它的测试，以及在这个提交上实际跑出来的验证结果（写在提交正文里）。不要先全部做完再拆成几个提交，那等于伪造开发历史。
- 提交标题：Conventional Commits 的类型加中文摘要，例如 `feat(audio): 播放本地 FLAC`，和 SPlayer-Next 一样。正文写清做了什么、跑了哪些检查、哪些没跑。
- 每次提交后推送。PR 合并时会 squash 成一个提交。
- 分支保持线性。要跟上 main，就 `git fetch origin && git rebase origin/main`，然后 `git push --force-with-lease`。不要把 main 或别的分支 merge 进任务分支（钩子会拒绝）。不要强推别的 agent 推过的提交。
- 需要一个回退点就打 `checkpoint/<编号>-<n>` tag；需要构建别的提交做 A/B，用 `scripts/bench-ab.sh` 或 `git worktree add --detach`。这两种情况都不开分支。
- 交 PR 审查：`scripts/pr.sh "<type(scope): 摘要>" <描述文件>`。标题会成为 main 上的提交。
- PR 保持小：改动大约 400 行以内（不算测试和生成文件）。大的拆成堆叠 PR。
- 摸底实验放在 `exp/<编号>-<slug>` 分支上，同样用 `scripts/new-task.sh` 开，永不合并；实验成功了，在正常任务分支上重做一遍。
- 只推送到 `origin`。不要改名、删除或强推不是你开的分支，不要改写 `main` 的历史。
- 每次会话结束，跑 `scripts/branches.sh`，报告你分支上的任何标记。

## 完成的定义（每个 PR）

- `bash scripts/check.sh` 本地和 CI 都通过。
- 每个改变了的行为都有测试，放在能抓住它的最低一层：`domain`、`app` 用单元测试和假的 port；适配器用集成测试；界面用快照测试（insta）。
- 按键冲突测试通过：每个窗口"全局键 + 本窗口键"没有重复。
- 任务卡里的验收命令在最终提交上跑过，结果贴在 PR 描述里。
- 动了渲染、音频或网络的热路径，就在 `docs/experiments.md` 里记一行性能数字，和上一个基线比。
- 结构变了就更新 `docs/architecture.md`；做了取舍就写 ADR；复制了代码就更新 `THIRD_PARTY.md`。
- 任务卡的进度日志和交接是最新的；`PROGRESS.md` 反映了当前状态。
- PR 描述里写明哪些部分没有验证。
- 里程碑结束时，按 `docs/acceptance.md` 对应那一节交验收包。

## 验证预算

- 写代码时，只跑你碰到的那个 crate 的测试。
- 一批（一个用户看得见的小功能）结束时，跑一次 `bash scripts/check.sh`，再跑任务的验收命令。
- 任务卡没要求，就不要去搭测试框架、夹具库或工具。
- 检查失败时，留下第一次失败的输出，修根因，只重跑受影响的部分。
- 等长命令时用一次阻塞调用加足够的超时，不要每几秒轮询一次。

## 资源

- 同一时间最多 2 个 agent 写代码，其余只读。重任务一律走 `scripts/heavy.sh`。
- 每个 worktree 用自己的配置和缓存目录（`SPLAYER_TUI_HOME` 指向 worktree 内的目录），不要碰别的 worktree、进程或端口。
- 没有用户同意，不装编辑器插件、语言服务器或全局工具。
- 性能准绳机器是 X220（Debian，通过 SSH）。同一时间只有一个会话在 X220 上测量。

## 会话

- 开始：读任务卡（包括上一次交接）和 `PROGRESS.md`，检查分支状态。
- 按批次朝任务的验收推进。只有任务卡列出的决定、或者结构性的阻塞，才停下来问用户。
- 结束时，或者上下文快满时：
  1. 提交并推送；
  2. 在任务卡的进度日志里加一行；
  3. 重写任务卡的交接一节：分支、状态、下一步、踩过的坑、怎么接着做、你的会话 ID；
  4. 更新 `PROGRESS.md` 的当前状态（重写，不往末尾追加流水账）；
  5. 停掉你启动的进程（`scripts/procs.sh` 会列出）；
  6. 然后汇报。

## 汇报

先说用户看得见的结果：哪条验收命令通过了，第一条失败的是哪条。然后是证据（确切的命令和结果）。然后是没验证的部分和下一步。命令证明不了的进度，不要说。

## 永远不要

- 往 `main` 提交或推送、自己合并自己的 PR、为了变绿而放松检查。
- 在 `scripts/new-task.sh` 之外开分支、把分支互相 merge、推送到 `origin` 以外的远端。
- 手改生成文件、复制来的第三方代码（要改就先在 `THIRD_PARTY.md` 里登记"已修改"）、冻结的测试资产。
- 根据测试名、测试数据或期望输出写分支逻辑。
- 把凭据放进代码、提交说明、URL 或日志里，尤其是网易云的登录 Cookie（`MUSIC_U`、`__csrf`）。
