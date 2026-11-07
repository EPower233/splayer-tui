#!/usr/bin/env bash
# Lists the processes that run from this worktree (their working directory is inside it) and the TCP ports
# they listen on, so a session can stop what it started before it ends. Linux only. It only lists;
# stop a process yourself with: kill <pid>
# Usage: scripts/procs.sh
set -euo pipefail

root=$(git rev-parse --show-toplevel)
found=0
for dir in /proc/[0-9]*; do
  pid=${dir#/proc/}
  [ "$pid" = "$$" ] || [ "$pid" = "$PPID" ] && continue
  cwd=$(readlink "$dir/cwd" 2>/dev/null) || continue
  case "$cwd/" in
    "$root"/*) ;;
    *) continue ;;
  esac
  cmd=$(tr '\0' ' ' < "$dir/cmdline" 2>/dev/null | cut -c1-100) || continue
  [ -n "$cmd" ] || continue
  ports=$(ss -ltnpH 2>/dev/null | awk -v p="pid=$pid," 'index($0, p) { n = split($4, a, ":"); print a[n] }' | sort -un | paste -sd, - || true)
  printf '%-8s %-12s %s\n' "$pid" "${ports:--}" "$cmd"
  found=1
done

if [ "$found" = 0 ]; then
  echo "nothing is running from $root"
else
  echo "(pid, listening ports, command) — stop what you started with: kill <pid>"
fi
