# 架构

## 目的

SPlayer-Next 的终端版。播放器的规则和流程都在内核里，终端界面只是其中一个前端；内核按 SPlayer-Next 的模块和类型组织，将来 SPlayer-Next 也能调用。需求见 `docs/spec/product.md`，构法见 `docs/adr/0002-approach-d.md`。

## 外部关系

```
            按键 / 鼠标                    画面
  用户 ───────────────► 终端 ◄──────────────────┐
                         │                      │
                         ▼                      │
                 splayer-tui（界面）─── 快照 ────┘
                         │ 命令 ▲ 事件
                         ▼      │
                 splayer-app（播放器大脑）
                         │ 通过 ports 里的 trait
     ┌───────────┬───────┴───────┬──────────────┬─────────────┐
     ▼           ▼               ▼              ▼             ▼
 音频引擎     本地文件夹        网易云接口      系统媒体控制    本地存储
 （声卡）     （磁盘）          （HTTPS）       （SMTC/MPRIS）  （配置、缓存）
```

## 依赖规则

依赖只能往里指：`可执行文件 → 适配器 / 界面 → app → domain`。可执行文件是唯一知道具体用哪些适配器的地方。

规则写在 `scripts/arch-check.sh` 里，`scripts/check.sh` 会跑它：它读每个 crate 的 `Cargo.toml`，凡是出现了表里不允许的内部依赖，就报错。Cargo 自己保证没写进 `Cargo.toml` 的依赖引用不到，所以只要查 `Cargo.toml` 就够了。

**违规示例**（验收 M0 时用来确认检查真的会变红）：在 `crates/domain/Cargo.toml` 的 `[dependencies]` 里加一行 `splayer-tui = { path = "../tui" }`，跑 `bash scripts/check.sh`，架构那一步应该失败，并指出 `splayer-domain` 不能依赖 `splayer-tui`。

## 目录

```
crates/
  domain/           splayer-domain：类型（对齐 SPlayer-Next 的 shared/types）和纯规则
  app/              splayer-app：播放器大脑；ports/ 里是对外部世界的 trait
  audio/            splayer-audio：音频输出适配器（T-1 摸底后决定用 SPlayer-Next 的引擎还是 rodio）
  source-local/     splayer-source-local：本地文件夹来源
  source-netease/   splayer-source-netease：网易云来源
  mediactl/         splayer-mediactl：系统媒体控制
  store/            splayer-store：配置、登录会话、缓存目录
  tui/              splayer-tui：终端界面
  bin/              splayer-tui-bin：可执行文件 splayer-tui，负责组装
docs/               设计、规格、ADR、任务卡、实验、发布记录
scripts/            check.sh、arch-check.sh、heavy.sh 和工具包的分支脚本
tools/              调试用的小工具（终端画线实测等）
```

以后的来源（QQ 音乐、酷狗、流媒体服务器、插件）各自一个 `source-*` crate，和 SPlayer-Next 的 `apis/<平台>`、`services/streaming/adapters` 一一对应。

## crate 一览

| crate | 用例 / 内容 | 实现的 port | 允许依赖的内部 crate |
|---|---|---|---|
| `splayer-domain` | Track、Album、Artist、Playlist、歌词类型；队列、播放模式 | — | 无 |
| `splayer-app` | 命令、事件、播放流程；ports：`MusicSource`、`AudioOutput`、`MediaControls`、`Store` | — | domain |
| `splayer-audio` | 音频输出 | `AudioOutput` | domain、app |
| `splayer-source-local` | 本地文件夹 | `MusicSource` | domain、app |
| `splayer-source-netease` | 网易云 | `MusicSource` | domain、app |
| `splayer-mediactl` | 系统媒体控制 | `MediaControls` | domain、app |
| `splayer-store` | 配置、会话、缓存 | `Store` | domain、app |
| `splayer-tui` | 终端界面、终端能力探测、键位表 | — | domain、app |
| `splayer-tui-bin` | 组装和程序入口 | — | 全部 |

ports 的具体方法，等 M1 的第一条打通路径用到时再定，不提前设计。

## 新代码放哪

| 你在写的东西 | 放在 |
|---|---|
| 不碰 I/O 的规则、计算、校验 | `domain` |
| 一个用户动作的流程步骤 | `app` |
| 流程需要外部世界提供的东西（音频、网络、存储、系统接口） | `app/src/ports/` 里的一个 trait |
| 那个 trait 的真实实现 | 对应的适配器 crate |
| 画面、按键、终端能力 | `tui` |
| 读配置、建连接、选用哪个实现 | `bin` |

## 横切关注点

- **错误**：`domain` 和 `app` 用 `thiserror` 定义具体的错误类型；适配器把底层错误翻译成这些类型；只有 `bin` 用 `anyhow`。
- **日志**：用 `tracing`，只在适配器和 `bin` 里打日志。日志里不能出现登录凭据。
- **配置**：只在 `bin` 里读，传给需要的地方。
- **异步**：tokio 运行时由 `bin` 创建。`app` 对外只暴露命令和事件两条通道，界面只往命令通道里发、从事件通道里收。
- **时间和随机数**：`domain` 不直接取，需要时由调用方传进来，方便测试。

## 测试

| 层 | 测什么 | 怎么测 |
|---|---|---|
| domain | 规则 | 普通单元测试 |
| app | 流程步骤 | 单元测试，ports 用内存里的假实现 |
| 适配器 | 真实读写：文件、网络（录下来的响应）、音频 | 集成测试；真联网的测试默认跳过，手动跑 |
| tui | 布局、对齐、按键 | ratatui 的 TestBackend 渲染成文字，和 insta 快照比；按键冲突测试 |
| bin | 整条路径能走通 | 少量端到端冒烟测试（假来源 + 假音频） |

## 适应度检查

`scripts/check.sh` 或 CI 会跑的检查，保证架构一直成立：

- 依赖方向（`scripts/arch-check.sh`）。
- 按键冲突测试（`tui` 里有键位表以后加上）。
- 界面快照测试（同上）。
- 时间预算：改一处后相关检查 2 分钟内跑完，CI 全量 10 分钟内。
- 性能预算：`docs/plan.md` 4.5 节，由 `scripts/bench.sh` 测，结果记进 `docs/experiments.md`。

## 决定和待定问题

决定记在 `docs/adr/`。待定问题见 `PROGRESS.md` 的"等用户定的"。
