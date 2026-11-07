#!/usr/bin/env bash
# Same-machine A/B of one benchmark on two commits. Each side is built in its own temporary worktree, and
# the rounds alternate (base, candidate, base, ...) so machine noise hits both sides equally. The whole run
# holds the heavy-job lock, so no build or other benchmark disturbs it. Paste the summary into docs/experiments.md.
# Usage: scripts/bench-ab.sh <base-commit> <candidate-commit> [rounds]
set -euo pipefail

[ $# -ge 2 ] || { echo "usage: $0 <base-commit> <candidate-commit> [rounds]" >&2; exit 2; }
base=$1 cand=$2 rounds=${3:-3}
root=$(git rev-parse --show-toplevel)

# Replace both functions for your project; until then the script fails.
build() { echo "scripts/bench-ab.sh: build() is not configured (it builds the tree in $1)" >&2; exit 1; }
# e.g. build() { (cd "$1" && cmake -S . -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build -j16); }
bench() { echo "scripts/bench-ab.sh: bench() is not configured (it prints 'metric value' lines for $1)" >&2; exit 1; }
# e.g. bench() { (cd "$1" && ./build/app-bench --rows 20000) | awk '/_ms|_sec/ {print $1, $2}'; }

# Optional binary archive: set ARTIFACT to the build output's path inside the tree (e.g. build/app) and each
# commit is built once; later runs against the same baseline or candidate reuse the archived output.
ARTIFACT=${ARTIFACT:-}
archive=${BENCH_ARCHIVE:-$HOME/.cache/$(basename "$root")-builds}
cached_build() {
  local tree=$1 sha name
  sha=$(git -C "$tree" rev-parse HEAD)
  name=$(basename "${ARTIFACT:-none}")
  if [ -n "$ARTIFACT" ] && [ -e "$archive/$sha/$name" ]; then
    mkdir -p "$tree/$(dirname "$ARTIFACT")"
    cp -a "$archive/$sha/$name" "$tree/$ARTIFACT"
    echo "reused the archived build of ${sha:0:12}"
    return
  fi
  build "$tree"
  if [ -n "$ARTIFACT" ]; then
    mkdir -p "$archive/$sha"
    cp -a "$tree/$ARTIFACT" "$archive/$sha/"
  fi
}

exec 9>"${HEAVY_LOCK:-/tmp/$(basename "$root")-heavy.lock}"
flock 9

work=$(mktemp -d)
cleanup() {
  git -C "$root" worktree remove --force "$work/base" 2>/dev/null || true
  git -C "$root" worktree remove --force "$work/cand" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT
git -C "$root" worktree add --quiet --detach "$work/base" "$base"
git -C "$root" worktree add --quiet --detach "$work/cand" "$cand"

cached_build "$work/base"
cached_build "$work/cand"

for i in $(seq 1 "$rounds"); do
  bench "$work/base" | sed "s/^/base $i /"
  bench "$work/cand" | sed "s/^/cand $i /"
done > "$work/results.txt"

# Median per metric and side; lower is better for timings.
echo "base=$base candidate=$cand rounds=$rounds"
for metric in $(awk '{print $3}' "$work/results.txt" | sort -u); do
  line="$metric"
  for side in base cand; do
    vals=$(awk -v s="$side" -v m="$metric" '$1 == s && $3 == m {print $4}' "$work/results.txt" | sort -g)
    n=$(printf '%s\n' "$vals" | wc -l)
    med=$(printf '%s\n' "$vals" | sed -n "$(( (n + 1) / 2 ))p")
    line="$line $side=$med"
  done
  echo "$line"
done
