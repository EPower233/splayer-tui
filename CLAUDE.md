@AGENTS.md

<!-- 项目规则都在 AGENTS.md，所有 agent 读同一份。这里只放 Claude Code 专用的说明。机器相关的笔记（本地路径、占用的端口）放用户记忆，不进仓库。 -->

## Claude Code 专用

- 在 Windows 上，`scripts/*.sh` 用 Bash 工具（Git Bash）跑，不要用 PowerShell 跑。
- 长时间的构建用后台运行，完成时会收到通知；不要循环 sleep 轮询。
- 会话 ID 写进 worktree 根目录 `.agent` 文件的 `session=` 一行，钩子会把它加进提交的 `Session:` 尾注。
