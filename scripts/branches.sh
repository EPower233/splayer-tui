#!/usr/bin/env bash
# Where did every branch come from, and where is it going? One line per branch, then the worktrees,
# with problems flagged.
# Usage: scripts/branches.sh            report
#        scripts/branches.sh --graph    compact graph of main and every branch
#        scripts/branches.sh --prune    report, then delete local branches whose remote branch is gone
#                                       (merged or closed), removing their worktree first if it is clean
#
# Branch flags:
#   BAD-NAME            not <type>/<id>-<slug>, backup/... or release/...: probably made by hand
#   LOCAL-ONLY          never pushed                 UNPUSHED-n / BEHIND-ORIGIN-n / DIVERGED-a/b vs origin
#   UPSTREAM-MISMATCH   tracks a differently named remote branch
#   NO-PR               no pull request (needs gh)   MERGED-NOT-DELETED  PR merged, remote branch still there
#   HAS-MERGES          merge commits on the branch
#   SHARES-n-WITH:x     shares n commits with x without a declared stack
#   STACKED-ON:x / STACK-PARENT-OF:x  declared with new-task.sh --stack-on
#   PARENT-GONE         its parent was merged: run scripts/restack.sh in its worktree
#   GONE                remote branch deleted (usually merged): --prune removes it
#   STALE-nd / EMPTY    untouched for n days / nothing beyond main
# Worktree flags: MAIN-NOT-CHECKED-OUT, DETACHED, WORKTREE-MISMATCH
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
git fetch --quiet --prune origin
main=origin/main

if [ "${1:-}" = "--graph" ]; then
  git log --graph --oneline --decorate --simplify-by-decoration "$main" --branches --remotes=origin
  exit 0
fi

have_gh=0
declare -A pr=()
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  have_gh=1
  while IFS=$'\t' read -r head number state; do
    pr[$head]="#$number-$state"
  done < <(gh pr list --state all --limit 300 --json headRefName,number,state \
             --jq '.[] | [.headRefName, (.number | tostring), .state] | @tsv')
fi

has_local()  { git rev-parse --verify --quiet "refs/heads/$1" >/dev/null; }
has_remote() { git rev-parse --verify --quiet "refs/remotes/origin/$1" >/dev/null; }
ref_of()     { if has_local "$1"; then echo "refs/heads/$1"; else echo "refs/remotes/origin/$1"; fi; }
task_re='^(feat|fix|perf|refactor|test|docs|build|ci|chore|exp)/([0-9]+)-([a-z0-9-]+)$'

mapfile -t branches < <( { git for-each-ref --format='%(refname:short)' refs/heads;
                           git for-each-ref --format='%(refname:lstrip=3)' refs/remotes/origin; } |
                         grep -vxE 'main|HEAD' | sort -u)

# Declared base of every branch, from the description new-task.sh records ("...; base <name>@<sha>; ...").
declare -A base_of=()
for b in "${branches[@]}"; do
  base_of[$b]=$(git config "branch.$b.description" 2>/dev/null | sed -n 's/.*base \([^;]*\);.*/\1/p' || true)
done

printf '%-34s %-6s %-30s %-9s %-12s %-14s %s\n' BRANCH TASK 'BASE (DECLARED)' AHEAD/BEH LAST AGENT 'PR AND FLAGS'
for b in "${branches[@]}"; do
  ref=$(ref_of "$b")
  flags=()
  if [[ "$b" =~ $task_re ]]; then
    task="T-${BASH_REMATCH[2]}"
  else
    task="-"
    [[ "$b" =~ ^(backup|release)/ ]] || flags+=(BAD-NAME)
  fi

  merge_base=$(git merge-base "$ref" "$main")
  ahead=$(git rev-list --count "$main..$ref")
  behind=$(git rev-list --count "$ref..$main")
  last=$(git log -1 --format='%cd' --date=format:'%m-%d %H:%M' "$ref")
  age_days=$(( ( $(date +%s) - $(git log -1 --format=%ct "$ref") ) / 86400 ))
  agent=$(git log -1 --format='%(trailers:key=Agent,valueonly)' "$ref" | sed -n '1p')

  declared=${base_of[$b]}
  parent=${declared%@*}
  if [ -n "$declared" ]; then base="${parent}@${declared#*@}"; base=${base:0:30}; else base="? (main@$(git rev-parse --short "$merge_base"))"; fi

  if has_local "$b" && has_remote "$b"; then
    mine=$(git rev-list --count "refs/remotes/origin/$b..refs/heads/$b")
    theirs=$(git rev-list --count "refs/heads/$b..refs/remotes/origin/$b")
    if [ "$mine" -gt 0 ] && [ "$theirs" -gt 0 ]; then flags+=("DIVERGED-$mine/$theirs")
    elif [ "$mine" -gt 0 ]; then flags+=("UNPUSHED-$mine")
    elif [ "$theirs" -gt 0 ]; then flags+=("BEHIND-ORIGIN-$theirs"); fi
  fi
  if has_local "$b"; then
    upstream=$(git for-each-ref --format='%(upstream:short)' "refs/heads/$b")
    if [ -n "$upstream" ] && [ "$upstream" != "origin/$b" ]; then flags+=("UPSTREAM-MISMATCH:$upstream"); fi
    if [ "$(git for-each-ref --format='%(upstream:track)' "refs/heads/$b")" = "[gone]" ]; then flags+=(GONE); fi
  fi
  has_remote "$b" || flags+=(LOCAL-ONLY)

  [ -z "$(git rev-list --merges "$merge_base..$ref")" ] || flags+=(HAS-MERGES)
  if [ -n "$parent" ] && [ "$parent" != "main" ] && ! has_local "$parent" && ! has_remote "$parent"; then flags+=(PARENT-GONE); fi
  for other in "${branches[@]}"; do
    [ "$other" = "$b" ] && continue
    shared=$(( ahead - $(git rev-list --count "$main..$ref" "^$(ref_of "$other")") ))
    [ "$shared" -gt 0 ] || continue
    other_parent=${base_of[$other]%@*}
    if [ "$other" = "$parent" ]; then flags+=("STACKED-ON:$other")
    elif [ "$other_parent" = "$b" ]; then flags+=("STACK-PARENT-OF:$other")
    else flags+=("SHARES-$shared-WITH:$other"); fi
  done
  [ "$age_days" -le 2 ] || flags+=("STALE-${age_days}d")
  [ "$ahead" -gt 0 ] || flags+=(EMPTY)

  state=${pr[$b]:-}
  if [ "$have_gh" = 1 ]; then
    if [ -z "$state" ] && [ "$task" != "-" ]; then flags+=(NO-PR); fi
    if [[ "$state" == *-MERGED ]] && has_remote "$b"; then flags+=(MERGED-NOT-DELETED); fi
  fi

  printf '%-34s %-6s %-30s %-9s %-12s %-14s %s %s\n' "$b" "$task" "$base" "$ahead/$behind" "$last" "${agent:--}" "${state:--}" "${flags[*]:-}"
done

echo
echo "worktrees:"
first=1
while IFS=$'\t' read -r path head; do
  wflags=()
  if [ "$first" = 1 ]; then
    [ "$head" = "refs/heads/main" ] || wflags+=(MAIN-NOT-CHECKED-OUT)
    first=0
  elif [ "$head" = "(detached)" ]; then
    wflags+=("DETACHED: remove when done (git worktree remove $path)")
  else
    name=${head#refs/heads/}
    if [[ "$name" =~ $task_re ]] && [[ "$(basename "$path")" != *"-${BASH_REMATCH[2]}-${BASH_REMATCH[3]}" ]]; then
      wflags+=(WORKTREE-MISMATCH)
    fi
  fi
  printf '  %-60s %-34s %s\n' "$path" "${head#refs/heads/}" "${wflags[*]:-}"
done < <(git worktree list --porcelain | awk '
  $1 == "worktree" { path = substr($0, 10); head = "" }
  $1 == "branch"   { head = $2 }
  $1 == "detached" { head = "(detached)" }
  $0 == ""         { if (path != "") print path "\t" head; path = "" }
  END              { if (path != "") print path "\t" head }')
[ "$have_gh" = 1 ] || echo "(gh not available: PR state, NO-PR and MERGED-NOT-DELETED not checked)"

if [ "${1:-}" = "--prune" ]; then
  echo
  for b in $(git for-each-ref --format='%(refname:short) %(upstream:track)' refs/heads | awk '$2 == "[gone]" && $1 != "main" {print $1}'); do
    wt=$(git worktree list --porcelain | awk -v want="refs/heads/$b" '$1 == "worktree" {w = substr($0, 10)} $1 == "branch" && $2 == want {print w}')
    if [ -n "$wt" ] && ! git worktree remove "$wt"; then
      echo "kept $b: its worktree $wt still has uncommitted work"
      continue
    fi
    git branch -D "$b"
  done
  git worktree prune
  echo "Remote branches flagged MERGED-NOT-DELETED are not deleted automatically: git push origin --delete <branch>"
fi
