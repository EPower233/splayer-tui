# 0003. 技术栈

- 状态：提议中（等用户接受；标了"待摸底"的项由 M0 的实验决定，结果另写 ADR）
- 日期：2026-11-08
- 决定人：E_Power（用户），claude-opus 提出

## 背景

构法 D（ADR 0002）要求用 Rust，并且尽量复用 SPlayer-Next 已有的 Rust 代码，以及其他项目里许可证兼容的现成部件。候选和许可证见 `docs/plan.md` §3 的拼装清单。

## 决定

- **语言和工具链**：Rust，用 `rust-toolchain.toml` 钉死在 1.95.0（本机已装的稳定版），edition 2024。
- **异步运行时**：只用 tokio 一个。CNMPlayer 那种基于 compio 的代码只借思路，不原样搬。
- **终端界面**：ratatui + crossterm；封面用 ratatui-image；宽度计算用 unicode-width。
- **网络**：reqwest 0.13 + rustls，全程只用 aws-lc-rs 一套加密后端。音频引擎依赖的 reqwest 0.13 默认用 aws-lc-rs（T-1 发现），"只用 ring"做不到；fork ncm-api-rs 时把它的 reqwest 升到 0.13。
- **网易云接口**：fork SPlayer-Dev/ncm-api-rs（WTFPL），补上 SPlayer-Next 后来加的 xeapi 和易盾 v3 token（待摸底：T-2）。
- **音频**：SPlayer-Next 的 `native/audio-engine`，T-1 已验证可行，具体做法见 ADR 0004；备选 rodio + symphonia。
- **歌词**：amll-lyric（GPL-3.0），不行就退到 ypm 的 `yrc.rs`（待摸底：T-2）。
- **封面取色**：material-colors。
- **系统媒体控制**：souvlaki。
- **序列化和配置**：serde、TOML。
- **错误和日志**：thiserror、anyhow（只在 bin 里用）、tracing。
- **测试**：Rust 自带的测试，加上 insta 做界面快照。
- **依赖版本**：钉死；新加依赖先查许可证，必须能并进 AGPL-3.0。

## 考虑过的其他做法

- GPUI、Flutter：是以后做图形界面版时的选项，和这个 TUI 不冲突；内核库照 ADR 0002 写，那时可以直接复用。
- 自己写 TUI 框架：没必要，ratatui 的增量渲染已经够用，性能风险主要在我们自己的重画策略。

## 后果

- 单一语言，单个可执行文件。
- FFmpeg 静态构建不慢：T-1 量到干净 debug 构建约 40 s，干净 release 构建（LTO）1 分 21 秒，数字见 `docs/experiments.md`。
- aws-lc-rs 要编译 C 代码，构建机上要有 C 编译器；Windows 上本机的 `CC`/`CXX` 如果指向 msys2，要先去掉。
