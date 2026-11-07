#!/usr/bin/env bash
# 依赖方向检查：读每个 crate 的 Cargo.toml，内部依赖（splayer-*）只能指向表里允许的 crate。
# 规则和理由见 docs/architecture.md。新加 crate 要先写 ADR，再在下面登记。
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

declare -A allowed=(
  [splayer-domain]=""
  [splayer-app]="splayer-domain"
  [splayer-audio]="splayer-domain splayer-app"
  [splayer-source-local]="splayer-domain splayer-app"
  [splayer-source-netease]="splayer-domain splayer-app"
  [splayer-mediactl]="splayer-domain splayer-app"
  [splayer-store]="splayer-domain splayer-app"
  [splayer-tui]="splayer-domain splayer-app"
  [splayer-tui-bin]="*"
)

status=0
for manifest in crates/*/Cargo.toml; do
  name=$(awk -F'"' '/^\[package\]/{p=1;next} /^\[/{p=0} p && /^name *=/{print $2; exit}' "$manifest")
  if [ -z "${allowed[$name]+x}" ]; then
    echo "arch-check: $manifest 里的 crate '$name' 没有登记；新 crate 先写 ADR，再登记到 scripts/arch-check.sh 和 docs/architecture.md" >&2
    status=1
    continue
  fi
  [ "${allowed[$name]}" = "*" ] && continue
  deps=$(awk '
    /^\[/ { section = ($0 ~ /^\[(target\..*\.)?(dev-|build-)?dependencies\]$/) ; next }
    section && match($0, /^splayer-[a-z-]+/) { print substr($0, RSTART, RLENGTH) }
  ' "$manifest")
  for dep in $deps; do
    if ! grep -qw -- "$dep" <<<"${allowed[$name]}"; then
      echo "arch-check: $name 不能依赖 $dep（$manifest）。依赖只能往里指，见 docs/architecture.md" >&2
      status=1
    fi
  done
done

[ $status -eq 0 ] && echo "dependency directions ok"
exit $status
