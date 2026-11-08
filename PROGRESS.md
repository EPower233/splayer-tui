# PROGRESS.md

项目现在在哪一步。**每次会话结束时重写这份文件**，不往末尾追加流水账；每个会话做了什么，记在各自任务卡的进度日志里，加上 git 历史。

最后更新：2026-11-08 · claude-opus · 会话 `3da92460-4818-4aed-8564-b4f87a41b79d`

## 现在

**第 1 期 · M0 第 0 天：三个摸底实验。** 仓库、CI、`main` 保护都已就绪（https://github.com/EPower233/splayer-tui ，公开）。T-1（音频引擎）结论是"能"，用户已亲耳试听。接下来是 T-2（网易云接口）。

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
| 骨架：工具包、设计文档、规则文件、workspace、CI | 已在 `main` | 完成，CI 两个平台全绿 |
| T-1 摸底：音频引擎去掉 NAPI | `exp/1-audio-engine`（永不合并） | 完成，结论见 `docs/experiments.md`；Linux 部分等 X220 装依赖 |
| T-5 把 T-1 的结果写进 main | `docs/5-record-t1-audio-engine` | 等审查 |
| T-2 摸底：ncm-api-rs 扫码登录 → 喜欢列表 → 播放地址 → 逐字歌词 | `exp/2-netease-api` | 未开始；扫码要用户来 |
| T-3 摸底：ratatui 画样稿骨架，对齐、彩色下划线、封面 | `exp/3-terminal` | 未开始 |
| T-4 X220 性能基线 | 跟 T-3 一起 | 等 T-3 |

## 已经定了的

| 日期 | 决定 | 出处 |
|---|---|---|
| 2026-11-08 | "SPlayer" 一律指 SPlayer-Next；界面、默认值、功能都照它 | `docs/ui-direction.md` |
| 2026-11-08 | 构法 D：Rust 独立实现，模块和类型对齐 SPlayer-Next | `docs/scope.md`、`docs/adr/0002-approach-d.md` |
| 2026-11-08 | 目标是 SPlayer-Next 的终端版，多来源；第 1 期先做本地文件夹和网易云 | `docs/scope.md` §3–4 |
| 2026-11-08 | 仓库建在 EPower233 下，公开 | 用户 |
| 2026-11-08 | 终端档位先只做"现代"，"兼容"以后补 | `docs/ui-direction.md` |
| 2026-11-08 | 默认值全部照 SPlayer-Next | `docs/settings.md` |
| 2026-11-08 | 逐字歌词默认开、可关；不做流动背景；没有底部提示栏 | `docs/ui-direction.md` |
| 2026-11-08 | 听歌打卡、Last.fm、Discord、窗口标题歌词、状态栏歌词：先不做 | `docs/settings.md` |

## 等用户定的

| 事项 | 卡住什么 |
|---|---|
| 接受 ADR 0003（技术栈）和 ADR 0004（音频引擎） | M1 的 `splayer-audio` |
| 能不能在 X220 上装 `libpipewire-0.3-dev`、`libpulse-dev` | 音频引擎的 Linux 验证、T-4 |
| 键位 v2（`docs/keymap.md`）定稿 | M2 |
| 项目正式名称（现在是占位的 `splayer-tui`） | 发 v0.1 |
| 审查者用哪个 agent | 现在由用户审查 |
| 要不要、什么时候联系 SPlayer-Next 作者 imsyy | 不卡，越早越好 |

## 踩过的坑

- 本机全局 `CC`/`CXX` 指向 msys2 clang，构建带 FFmpeg、aws-lc 的代码要用 `env -u CC -u CXX LIBCLANG_PATH="C:\Program Files\LLVM\bin" cargo ...`。CI 没有这个问题。
- 用户的 SPlayer-Next 开着独占输出时，别的程序打不开声卡（`0x8889000A`）。试听或测试前要先让出来。

## 下一步

1. 开 `exp/2-netease-api` 做 T-2（不依赖 T-5 合并），到扫码那一步请用户来。
2. 然后 T-3、T-4，交 M0 验收包。
