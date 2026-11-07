#!/usr/bin/env bash
# Keep a stacked branch (made with new-task.sh --stack-on) on top of its parent:
#   - parent still open and moved on  -> replay this branch's own commits onto the parent's new tip;
#   - parent squash-merged and deleted -> replay them onto origin/main and retarget the pull request to main.
# Either way the declared base is updated, so scripts/branches.sh keeps telling the truth.
# Usage: scripts/restack.sh        (run inside the stacked branch's worktree)
set -euo pipefail

branch=$(git symbolic-ref --short HEAD)
description=$(git config "branch.$branch.description" || true)
declared=$(sed -n 's/.*base \([^;]*\);.*/\1/p' <<<"$description")
parent=${declared%@*}
old_base=${declared#*@}
if [ -z "$declared" ] || [ "$parent" = "main" ]; then
  echo "$branch is not a declared stack; for a branch on main use: git fetch && git rebase origin/main" >&2
  exit 1
fi

git fetch --quiet --prune origin
if git rev-parse --verify --quiet "refs/remotes/origin/$parent" >/dev/null; then
  new_parent=$parent
else
  new_parent=main
fi
target=$(git rev-parse "origin/$new_parent")

git rebase --onto "$target" "$old_base"
git config "branch.$branch.description" \
  "$(sed "s|base [^;]*;|base $new_parent@${target:0:12};|" <<<"$description")"
git push --quiet --force-with-lease

if [ "$new_parent" = "main" ] && command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  gh pr edit "$branch" --base main >/dev/null && echo "pull request retargeted to main"
fi
echo "$branch now sits on $new_parent@${target:0:12}"
