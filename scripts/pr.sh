#!/usr/bin/env bash
# Make the task's draft pull request reviewable: final Conventional Commit title, the description the agent
# wrote, and provenance trailers collected from the branch. Squash-merging with "PR title + PR body" carries
# the trailers into the commit on main, so `git log main` keeps saying which task and which agents made it.
# Usage: scripts/pr.sh "<type(scope): summary>" <description-file>
set -euo pipefail

title=${1:-}
desc=${2:-}
[ -n "$title" ] && [ -f "$desc" ] || { echo "usage: $0 \"<type(scope): summary>\" <description-file>" >&2; exit 2; }

branch=$(git symbolic-ref --short HEAD)
[[ "$branch" =~ ^[a-z]+/([0-9]+)- ]] || { echo "not on a task branch: $branch" >&2; exit 1; }
task="T-${BASH_REMATCH[1]}"
base=$(git config "branch.$branch.description" 2>/dev/null | sed -n 's/.*base \([^;]*\);.*/\1/p' || true)
base_name=${base%@*}
agents=$(git log --format='%(trailers:key=Agent,valueonly)' "origin/${base_name:-main}..HEAD" | sed '/^$/d' | sort -u | paste -sd ',' - | sed 's/,/, /g')

body=$(mktemp)
trap 'rm -f "$body"' EXIT
{
  cat "$desc"
  printf '\nTask: %s\nBase: %s\nAgents: %s\n' "$task" "${base:-unknown}" "${agents:-unknown}"
} > "$body"

git push --quiet
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  gh pr edit "$branch" --title "$title" --body-file "$body"
  gh pr ready "$branch"
  echo "pull request ready for review: $title"
else
  echo "gh is not available here. Set this title and description on the pull request by hand:"
  echo "title: $title"
  cat "$body"
fi
