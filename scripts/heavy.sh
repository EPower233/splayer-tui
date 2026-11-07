#!/usr/bin/env bash
# 重任务排队：release 构建、带 FFmpeg 的构建、基准测试一次只跑一个。
# 用法：bash scripts/heavy.sh <命令...>
# Git Bash 没有 flock，所以用 mkdir 做锁：mkdir 是原子操作，目录存在就说明别人在用。
set -euo pipefail
[ $# -gt 0 ] || { echo "usage: $0 <command...>" >&2; exit 2; }

lock="${HEAVY_LOCK:-${TMPDIR:-/tmp}/splayer-tui-heavy.lock}"
waited=0
until mkdir "$lock" 2>/dev/null; do
  if [ $waited -eq 0 ]; then
    echo "heavy.sh: 等别的重任务结束（锁：$lock，持有者：$(cat "$lock/owner" 2>/dev/null || echo 未知)）" >&2
  fi
  waited=1
  sleep 5
done
trap 'rm -rf "$lock"' EXIT
echo "pid=$$ cwd=$PWD cmd=$*" > "$lock/owner"
"$@"
