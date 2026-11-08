# 0004. 音频内核直接用 SPlayer-Next 的引擎，靠替身 crate 去掉 Node.js 依赖

- 状态：提议中（等用户接受）
- 日期：2026-11-08
- 决定人：E_Power（用户），claude-opus 提出

## 背景

ADR 0003 把音频列为待摸底项。SPlayer-Next 的 `native/audio-engine` 是 Rust 写的，功能齐全（FFmpeg 解码、HTTP 流、交叉淡入、WASAPI 独占），但它是给 Node.js 用的：`src/bindings/` 约 2100 行，加载、预载、seek、换曲的编排都写在里面，到处是 `#[napi]` 标注和 NAPI 类型，不是一层薄绑定。

T-1 验证了一个办法：另写三个 crate，名字就叫 `napi`、`napi-derive`、`napi-build`，用普通 Rust 实现引擎用到的那一小部分接口。引擎的 `src/` 和 `build.rs` 一个字节不改，只把 `crate-type` 从 cdylib 改成 rlib，就能当普通 Rust 库用。数字和复现命令见 `docs/experiments.md`。

## 决定

- `splayer-audio` 的内核用 SPlayer-Next 的 `native/audio-engine`。
- 引擎源码原样放进仓库，放在 `third_party/audio-engine/`，登记到 `THIRD_PARTY.md`，不手改。同步上游就是覆盖这个目录，再跑一遍测试。
- 三个替身 crate 放在 `third_party/` 下，由我们维护。实验版只求够用，正式版要收紧接口、补测试。
- `third_party/` 下的 crate 不参加本仓库的 fmt 和 clippy 检查，只有 `splayer-audio` 能依赖它们。
- `splayer-audio` 实现 `app::ports` 里的音频输出 trait，并负责两件引擎不管的事：
  - `stop()` 这类同步方法可能销毁 HTTP 流内部的 tokio 运行时，必须放到阻塞线程上调用，否则 panic。
  - 引擎报的设备错误是一串 OS 错误码，要翻译成用户看得懂的提示。比如 `0x8889000A` 是"输出设备被其他程序独占"。

## 考虑过的其他做法

- **改写 `src/bindings/`，把 NAPI 换成普通 Rust 接口**：T-1 原计划这么做。要改约 2100 行编排代码，而且以后每次同步上游都要手工合并，违背构法 D 向上游靠拢的目的。
- **rodio + symphonia**：依赖轻，但要自己重写加载、预载、seek、换曲的编排，交叉淡入和独占输出也要重做。只在 SPlayer-Next 的引擎走不通时才用。

## 后果

- 同步上游的成本很低，新功能和修复能直接拿到。
- 程序体积约 10 MB，含静态 FFmpeg。
- 替身 crate 依赖引擎用到的 NAPI 接口，上游用了新接口时要补替身；编译会直接报错，不会悄悄出问题。
- 上游改了 `crate-type` 以外的构建方式时，覆盖目录可能不够，要看 diff。
- 还没验证 Linux：X220 上缺 `libpipewire-0.3-dev` 和 `libpulse-dev`，要用户同意后才能装。如果 Linux 上编不过或者性能达不到 `docs/plan.md` 的预算，这个决定要重新考虑。
