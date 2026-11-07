# splayer-tui

> 占位名。**这是一个非官方项目，和 SPlayer-Next 及其作者没有关联。**

[SPlayer-Next](https://github.com/SPlayer-Dev/SPlayer-Next) 的终端版：一个在终端里跑的音乐播放器，布局和功能照 SPlayer-Next，长相和交互是 TUI 的。用 Rust 独立实现，模块和类型对齐 SPlayer-Next。

**现状：刚开工，还不能用。** 进度见 [PROGRESS.md](PROGRESS.md)。

## 文档

- 范围和构法：[docs/scope.md](docs/scope.md)
- 产品一页纸：[docs/spec/product.md](docs/spec/product.md)
- 界面原则：[docs/ui-direction.md](docs/ui-direction.md)，样稿：[docs/mockup.html](docs/mockup.html)（下载后用浏览器打开）
- 开发计划：[docs/plan.md](docs/plan.md)；验收：[docs/acceptance.md](docs/acceptance.md)
- 架构：[docs/architecture.md](docs/architecture.md)；决定记录：[docs/adr/](docs/adr/)
- 给 agent 的规则：[AGENTS.md](AGENTS.md)

## 开发

需要 Rust（版本由 `rust-toolchain.toml` 指定）。在 Windows 上用 Git Bash。

```bash
bash scripts/setup-repo.sh   # 每个 clone 一次
bash scripts/check.sh        # 格式、lint、依赖方向、测试
cargo run -p splayer-tui-bin
```

## 许可证

AGPL-3.0-only，见 [LICENSE](LICENSE)。本项目会使用 SPlayer-Next（AGPL-3.0）等项目的代码，来源登记在 [THIRD_PARTY.md](THIRD_PARTY.md)。

本项目仅供个人学习与研究使用。部分功能依赖第三方服务的接口，使用者须自行确保使用方式符合相关法律法规和服务协议。
