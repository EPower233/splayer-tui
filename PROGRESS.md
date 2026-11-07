# PROGRESS.md

项目现在在哪一步。**每次会话结束时重写这份文件**，不往末尾追加流水账；每个会话做了什么，记在各自任务卡的进度日志里，加上 git 历史。

最后更新：2026-11-08 · claude-opus · 会话 `3da92460-4818-4aed-8564-b4f87a41b79d`

## 现在

**第 1 期 · M0 第 0 天：建骨架。** 本地仓库已建好，初始化提交还在本地 `main` 上，还没有推送到任何远端。

## 里程碑

验收标准见 [docs/acceptance.md](docs/acceptance.md)。

| 里程碑 | 状态 | 验收 |
|---|---|---|
| M0 第 0 天：骨架和三个摸底实验 | 进行中 | 未交 |
| M1 能听歌（本地文件夹 → 网易云） | 未开始 | — |
| M2 像 SPlayer | 未开始 | — |
| M3 发 v0.1.0 | 未开始 | — |

## M0 任务队列

| 任务 | 类型 | 状态 |
|---|---|---|
| 初始化提交：工具包骨架、设计文档、规则文件、产品和架构文档、workspace 骨架 | 直接在 `main` 上（首推之前） | 进行中 |
| T-1 摸底：音频引擎去掉 NAPI 后独立编译，放本地 FLAC 和网络地址，量构建时间 | `exp/1-audio-engine` | 等远端 |
| T-2 摸底：ncm-api-rs 扫码登录 → 喜欢列表 → 播放地址 → 逐字歌词 | `exp/2-netease-api` | 等远端；扫码要用户来 |
| T-3 摸底：ratatui 画样稿骨架，对齐、彩色下划线、封面 | `exp/3-terminal` | 等远端 |
| T-4 X220 性能基线 | 跟 T-3 一起 | 等 T-3 |

`scripts/new-task.sh` 要先从 `origin` 拉取，所以有了远端才能开任务分支。

## 已经定了的

| 日期 | 决定 | 出处 |
|---|---|---|
| 2026-11-08 | "SPlayer" 一律指 SPlayer-Next；界面、默认值、功能都照它 | `docs/ui-direction.md` |
| 2026-11-08 | 构法 D：Rust 独立实现，模块和类型对齐 SPlayer-Next | `docs/scope.md`、`docs/adr/0002-approach-d.md` |
| 2026-11-08 | 目标是 SPlayer-Next 的终端版，多来源；第 1 期先做本地文件夹和网易云 | `docs/scope.md` §3–4 |
| 2026-11-08 | 终端档位先只做"现代"，"兼容"以后补 | `docs/ui-direction.md` |
| 2026-11-08 | 默认值全部照 SPlayer-Next | `docs/settings.md` |
| 2026-11-08 | 逐字歌词默认开、可关；不做流动背景；没有底部提示栏 | `docs/ui-direction.md` |
| 2026-11-08 | 听歌打卡、Last.fm、Discord、窗口标题歌词、状态栏歌词：先不做 | `docs/settings.md` |

## 等用户定的

| 事项 | 卡住什么 |
|---|---|
| GitHub 远端：建在哪个账号、仓库名、公开还是私有 | M0 的所有任务分支和 CI |
| 键位 v2（`docs/keymap.md`）定稿 | M2 |
| 项目正式名称（现在是占位的 `splayer-tui`） | 发 v0.1 |
| 审查者用哪个 agent | 第一个 PR |
| 要不要、什么时候联系 SPlayer-Next 作者 imsyy | 不卡，越早越好 |

## 下一步

1. 完成剩下的初始化提交，`bash scripts/check.sh` 全绿。
2. 等用户定远端；建好后首推 `main`，再按工具包设置 main 保护。
3. 用 `scripts/new-task.sh` 开 T-1 到 T-3，先做 T-1（风险最大，FFmpeg 构建时间未知）。
