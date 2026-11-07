#!/usr/bin/env bash
# The only sanctioned way to create a branch. In one step it:
#   1. branches from origin/main (or, for a declared stack, from the parent task branch),
#   2. creates the worktree next to the main checkout and writes .agent there,
#   3. writes the task card with the branch's provenance and commits it,
#   4. pushes the branch and opens a draft pull request, so the branch is visible from minute one.
# Usage: scripts/new-task.sh <type> <id> <slug> --agent <name> [--stack-on <parent-branch>]
set -euo pipefail

usage() { echo "usage: $0 <type> <id> <slug> --agent <name> [--stack-on <parent-branch>]" >&2; exit 2; }
[ $# -ge 3 ] || usage
type=$1 id=$2 slug=$3
shift 3
agent=${AGENT:-}
parent=""
while [ $# -gt 0 ]; do
  case "$1" in
    --agent) agent=${2:-}; shift 2 ;;
    --stack-on) parent=${2:-}; shift 2 ;;
    *) usage ;;
  esac
done

case "$type" in
  feat|fix|perf|refactor|test|docs|build|ci|chore|exp) ;;
  *) echo "unknown type '$type' (feat fix perf refactor test docs build ci chore exp)" >&2; exit 2 ;;
esac
[[ "$id" =~ ^[0-9]+$ ]] || { echo "id must be a number" >&2; exit 2; }
[[ "$slug" =~ ^[a-z0-9][a-z0-9-]*$ ]] || { echo "slug must be lower-case words joined by '-'" >&2; exit 2; }
[ -n "$agent" ] || { echo "say who works on this branch: --agent <name> (for example claude-opus, codex, human-alex)" >&2; exit 2; }

root=$(git rev-parse --show-toplevel)
branch="$type/$id-$slug"
worktree="$(dirname "$root")/$(basename "$root")-$id-$slug"

git -C "$root" fetch --quiet origin
base_name=${parent:-main}
base_sha=$(git -C "$root" rev-parse --verify --quiet "origin/$base_name") || { echo "no such base: origin/$base_name" >&2; exit 1; }
if git -C "$root" rev-parse --verify --quiet "refs/heads/$branch" >/dev/null ||
   git -C "$root" rev-parse --verify --quiet "refs/remotes/origin/$branch" >/dev/null; then
  echo "branch $branch already exists" >&2; exit 1
fi
created=$(date -u +%Y-%m-%dT%H:%MZ)
provenance="task T-$id; base $base_name@${base_sha:0:12}; created $created by $agent"

git -C "$root" worktree add --quiet --no-track -b "$branch" "$worktree" "$base_sha"
git -C "$root" config "branch.$branch.description" "$provenance"
printf 'agent=%s\nsession=\n' "$agent" > "$worktree/.agent"

card="docs/tasks/T-$id.md"
if [ ! -e "$worktree/$card" ]; then
  sed -e "s|{{ID}}|T-$id|g" -e "s|{{BRANCH}}|$branch|g" -e "s|{{WORKTREE}}|$worktree|g" \
      -e "s|{{BASE}}|$base_name@${base_sha:0:12}|g" -e "s|{{CREATED}}|$created|g" -e "s|{{AGENT}}|$agent|g" \
      "$worktree/docs/tasks/TEMPLATE.md" > "$worktree/$card"
fi

cd "$worktree"
git add "$card"
git commit --quiet -m "docs(tasks): open T-$id $slug" -m "Provenance: $provenance"
git push --quiet -u origin HEAD

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  gh pr create --draft --base "$base_name" --head "$branch" \
    --title "$type: $slug (T-$id)" \
    --body "Task card: \`$card\`. Provenance: $provenance." >/dev/null
  echo "draft pull request opened"
else
  echo "gh is not available here: open the draft pull request for $branch by hand now"
fi

echo "branch:    $branch (from $base_name @ ${base_sha:0:12})"
echo "worktree:  $worktree"
echo "task card: $worktree/$card"
