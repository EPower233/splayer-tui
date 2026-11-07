# AGENTS.md

Standing instructions for every coding agent in this repository. Claude Code reads this file through `CLAUDE.md`; other agents read it directly. Where these rules differ from an agent tool's built-in defaults, these rules win.

## Project

<One paragraph: what the product does, for whom, and what it deliberately does not do. Details: `docs/spec/product.md`.>

## Read before working

1. Your task card in `docs/tasks/` (the user names it): goal, acceptance, scope, and the last handoff.
2. `docs/architecture.md` for where code goes.
3. `git status`, `git log --oneline -10`, and the CI status of your branch.

Read what your task touches, not the whole repository.

## Architecture (checked by `./scripts/check.sh`)

- Code is grouped by feature under `src/<feature>/`, with three layers in each feature:
  - `domain/`: business rules and data. Imports nothing outside `domain/` and `src/shared/kernel/`. No I/O, no framework, no clock, no randomness.
  - `application/`: one use case per file. Orchestrates domain objects. Declares the ports (interfaces) it needs in `application/ports/`.
  - `adapters/`: `outbound/` implements ports (database, external APIs, files); `inbound/` turns requests (HTTP, CLI, jobs) into use-case calls.
- `src/main/` is the only place that names concrete adapters: it reads configuration and wires adapters into use cases.
- Dependencies point inward only: `main → adapters → application → domain`.
- Add an interface only at a boundary (a port the use case needs from the outside world, or a seam a test needs). No interface with a single implementation inside a layer.
- One responsibility per file. Split a function when it mixes levels of abstraction or no longer fits on a screen.
- A new layer, top-level directory, heavyweight dependency, or cross-feature call needs an ADR in `docs/adr/` accepted by the user first.

## Commands

- Setup: `<command>`
- Verify everything, the same command CI runs: `./scripts/check.sh`
- Fast loop while coding: `<unit tests for one feature>`
- Integration and end-to-end tests: `<command>`
- Run locally: `<command>`
- Heavy jobs (full builds, integration tests, benchmarks) always take the shared lock: `flock "${HEAVY_LOCK:-/tmp/<repo>-heavy.lock}" <command>`

## Branches and commits (the git hooks in `.githooks/` enforce most of this)

- `main` is always releasable. Never commit to `main` or push to it. It changes only through squash-merged pull requests with green CI, merged by the user.
- Never create a branch yourself (`git branch`, `git switch -c`, `git checkout -b`). Only `scripts/new-task.sh` creates branches. It records where the branch starts, which task it serves and which agent opened it, and it opens a draft pull request. If you need a branch that does not exist, ask the user.
- Work only in the worktree, and on the branch, that your task card names. One writer per worktree.
- Every commit carries `Task:` and `Agent:` trailers, which the hooks add from the branch name and from the worktree's `.agent` file. Do not remove them. When a task changes hands, the user updates `.agent`.
- Commit early and often, and push after each commit. Checkpoint commits are fine: the pull request is squashed into one commit at merge.
- Keep the branch linear. To catch up with main, run `git fetch origin && git rebase origin/main`, then `git push --force-with-lease`. Never merge main or any other branch into a task branch; the hooks refuse it. Never force-push over commits another agent pushed.
- Need a point to come back to? Tag it `checkpoint/<id>-<n>`. Need to build some other commit for an A/B? Use `scripts/bench-ab.sh` or `git worktree add --detach`. Neither is a reason to open a branch.
- A candidate that is not on `main` yet is submitted to a platform or target environment as a `cand/<id>-<n>` tag, and its score goes into `docs/experiments.md`. Final submissions are `submit-` tags on `main` only.
- A stacked branch (opened with `--stack-on`) follows its parent with `scripts/restack.sh`, both when the parent moves and after the parent is merged.
- Hand a pull request in for review with `scripts/pr.sh "<type(scope): summary>" <description-file>`. The title becomes the commit on `main`: Conventional Commits, lower case, at most 72 characters. Types: feat, fix, perf, refactor, test, docs, build, ci, chore, revert.
- Keep a pull request small: about 400 changed lines or fewer, excluding tests and generated files. Split larger work into stacked pull requests, or hide unfinished behaviour behind a feature flag.
- Experiments go on `exp/<id>-<slug>` branches, also opened with `scripts/new-task.sh`, and are never merged; a winning experiment is redone on a normal task branch. Work that is not on a task branch and must be saved goes to `backup/<agent>/<yyyymmdd>-<slug>`, and is deleted when the task ends.
- Push only to `origin`. Never rename, delete or force-push a branch you did not open, and never rewrite `main`.
- At the end of every session, run `scripts/branches.sh` and report any flag on your branch.

## Definition of done (every pull request)

- `./scripts/check.sh` passes locally and in CI.
- Every changed behaviour has a test at the lowest layer that can catch it: unit tests with fake ports for domain and application, integration tests for outbound adapters.
- The task card's acceptance commands were run on the final commit, and their results are in the pull request description.
- `docs/architecture.md` and the ADRs reflect any structural change. The task card's progress log and handoff are up to date.
- The pull request description says what was not verified.

## Verification budget

- While coding, run only the fast loop for the code you touch.
- At the end of a batch (one user-visible slice), run `./scripts/check.sh` once, then the task's acceptance commands.
- Do not build test frameworks, fixture libraries or tooling unless the task card asks for it.
- When a check fails, keep the first failing output, fix the root cause, and rerun only what is affected.
- Wait for long-running commands with one blocking call and a generous timeout; do not poll every few seconds.

## Resources

- At most <N> agents write code at the same time; the rest are read-only. Heavy jobs always take the lock.
- Each worktree uses its own ports and data directories (from its `.env.local`). Never touch another worktree, process or port.
- Do not install editor plugins, language servers or global tools without the user's approval.

## Sessions

- Start: read the task card, including the last handoff, and check the branch state.
- Work in bounded batches toward the task's acceptance. Stop to ask only for the decisions the task card lists, or for a structural blocker.
- End, or when your context is getting long: commit and push; append one line to the progress log; rewrite the handoff section (branch, state, next step, pitfalls, how to resume, your session ID); stop the processes you started (`scripts/procs.sh` lists them); then report.

## Reporting

Lead with the user-visible result: which acceptance command passes and which fails first. Then the evidence (exact commands and results). Then what is not verified, and the next step. Claim no progress that a command cannot show.

## Never

- Commit to or push `main`, merge your own pull request, or weaken a check to get green.
- Create branches outside `scripts/new-task.sh`, merge branches into each other, or push to any remote other than `origin`.
- Hand-edit generated files, vendored code or frozen test assets.
- Branch on test names, test data or expected outputs.
- Put secrets in code, commit messages, URLs or logs.
