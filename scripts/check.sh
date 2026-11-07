#!/usr/bin/env bash
# The one verification command for people, agents and CI.
# Replace every `todo` with the real command for your stack (see arch-checks.md in the kit).
# Until then the script fails, so nobody mistakes an unconfigured check for a passing one.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

step() { printf '\n== %s\n' "$1"; }
todo() { echo "scripts/check.sh: step '$1' is not configured yet" >&2; exit 1; }

step "format";       todo format        # e.g. ruff format --check . | npx prettier --check . | cargo fmt --check | test -z "$(gofmt -l .)"
step "lint";         todo lint          # e.g. ruff check . | npx eslint . | cargo clippy --all-targets -- -D warnings | golangci-lint run
step "types";        todo types         # e.g. mypy src | npx tsc --noEmit   (compiled languages: delete this step)
step "architecture"; todo architecture  # e.g. lint-imports | npx depcruise src --config .dependency-cruiser.cjs | scripts/arch-check.sh
step "unit tests";   todo unit-tests    # e.g. pytest -q tests/unit | npx vitest run | cargo test | go test ./...

printf '\nall checks passed\n'
