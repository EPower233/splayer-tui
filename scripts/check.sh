#!/usr/bin/env bash
# 人、agent、CI 跑的同一条验证命令。用法：bash scripts/check.sh
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

step() { printf '\n== %s\n' "$1"; }

# 架构检查放最前：它不用编译，几毫秒就能指出依赖方向错在哪，不用等 Cargo 报循环依赖。
step "architecture"; bash scripts/arch-check.sh
step "format";       cargo fmt --all --check
step "lint";         cargo clippy --workspace --all-targets --locked -- -D warnings
step "unit tests";   cargo test --workspace --locked

printf '\nall checks passed\n'
