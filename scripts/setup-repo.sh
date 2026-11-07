#!/usr/bin/env bash
# One-time setup for every clone: provenance hooks and local git settings. Safe to run again.
# Run it as `bash scripts/setup-repo.sh`, which works even before the files are executable.
# In a brand-new repository, record the executable bits once in the first commit:
#   git add --chmod=+x .githooks/* scripts/*.sh
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

chmod +x .githooks/* scripts/*.sh
git config core.hooksPath .githooks
git config pull.rebase true
git config rebase.autoStash true
git config fetch.prune true
git config push.autoSetupRemote true
git config branch.autoSetupMerge simple   # never record a differently named branch as upstream
git config rerere.enabled true

echo "hooks installed from .githooks"
echo "start every task with: scripts/new-task.sh <type> <id> <slug> --agent <name>"
