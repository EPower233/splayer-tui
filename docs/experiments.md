# Experiments

One row per experiment branch. Experiment branches (`exp/*`) are never merged; a winner is redone on a normal task branch and goes through a pull request.

Before the first row, run the unmodified code through the oracle once and record it: an oracle that already gives the baseline a full score, or whose run-to-run noise exceeds the gain you hope for, cannot judge a change.

| Date | Branch | Hypothesis | Oracle (command, environment, baseline) | Result vs baseline | Submitted as (`cand/` tag) and score | Decision |
|---|---|---|---|---|---|---|
| 2026-11-08 | `exp/1-audio-engine` | SPlayer-Next 的音频引擎去掉 Node.js 依赖后，能在普通 Rust 程序里放歌和 seek，构建时间可以接受 | 见下面"T-1"一节的复现命令；Windows，Rust 1.95.0，MSVC；基线是原样带 NAPI 的干净 debug 构建 43.6 s | 替身版干净 debug 37.2 s，增量 3.1～3.8 s，干净 release 1 分 21 秒，程序 10.2 MB；本地、本机 HTTP、HTTPS 都能放、能 seek；用户试听正常 | 不提交（摸底实验） | 采用 → ADR 0004，M1 的 `splayer-audio` 重做 |

## T-1：音频引擎去掉 NAPI

**结论：能。** SPlayer-Next 的 `native/audio-engine`（@ a8716df）不改源码就能当普通 Rust 库用，能放本地 FLAC 和网络地址，能 seek，构建时间不是问题。用户 2026-11-08 亲耳试听本地 FLAC 和 seek，回复"声音正常"。

做法：另写三个替身 crate（`napi`、`napi-derive`、`napi-build`），用普通 Rust 实现引擎用到的那部分接口。引擎的 `src/` 和 `build.rs` 与上游逐字节一致，只把 `crate-type` 从 cdylib 改成 rlib。

实验代码在分支 `exp/1-audio-engine` 的 `spikes/audio-engine/`（提交 `4a31397`），完整过程见该分支的 `docs/tasks/T-1.md`。

### 构建时间

Windows 11，Rust 1.95.0，MSVC。

| 项目 | 结果 |
|---|---|
| 原样带 NAPI，干净 debug 构建（基线） | 43.6 s，229 个 crate |
| 替身版，干净 debug 构建 | 37.2 s，218 个 crate |
| 替身版，改一行后增量构建 | 3.1 s（改引擎）、3.8 s（改 `play`） |
| 替身版，干净 release 构建（LTO） | 1 分 21 秒；在新的 detached worktree 里用下面的复现命令再跑一次是 1 分 22 秒 |
| release 版程序体积（含静态 FFmpeg） | 10.2 MB |

### 播放

用 `PLAY_VOLUME=0` 静音跑，seek 后位置都正确，退出码 0。

| 来源 | 加载 | seek |
|---|---|---|
| 本地 FLAC | 38～40 ms | 20～28 ms |
| 本机 HTTP + Range | 50 ms | 42～48 ms |
| HTTPS（GitHub 上的 MP3） | 3.7～6.7 s | 110 ms～2 s |

### 复现

```bash
git worktree add --detach ../t1-repro origin/exp/1-audio-engine
cd ../t1-repro/spikes/audio-engine
cargo run --release -p play -- <本地文件或网络地址> 10 30
```

`10` 是每段播放的秒数，`30` 是 seek 到第几秒。

- 想静音跑，在命令前加 `PLAY_VOLUME=0`。
- Windows 上如果全局 `CC`/`CXX` 指向 msys2，`cargo` 前要加 `env -u CC -u CXX LIBCLANG_PATH="C:\Program Files\LLVM\bin"`。
- 声卡被别的程序独占时（比如 SPlayer-Next 开着独占输出）会报 `0x8889000A`，要先让出来。

### 没验证

- Linux / X220：缺 `libpipewire-0.3-dev` 和 `libpulse-dev`。
- 网易云 CDN 的真实地址：留给 T-2。

对正式实现的影响见 `docs/adr/0004-audio-engine.md`。
