# {{ID}}: <title>

- Branch: `{{BRANCH}}`, from `{{BASE}}`, created {{CREATED}} by {{AGENT}}
- Worktree: `{{WORKTREE}}`
- Pull request: <link to the draft opened by scripts/new-task.sh>
- Status: todo | doing | review | done
- Owner: <the person or agent responsible> · Reviewer: <agent or person>
- Handed over: <date · from agent → to agent, one line per handover>

## Goal

<One sentence describing the user-visible behaviour this task delivers.>

## Acceptance

Commands that must pass on the final commit:

- `./scripts/check.sh`
- `<the command that shows the behaviour, e.g. an end-to-end test>`

## Scope

- May change: <directories or files>
- Must not change: <directories, files, frozen assets>
- Decisions that need the user: <list, or "none">

## Plan

1. <step>

## Verification budget

- While coding: <fast loop command>
- Once at the end: `./scripts/check.sh`, then the acceptance commands
- Heavy jobs and their lock: <command, or "none">

## Progress log

<!-- Append one line per batch: date time · agent · what changed (commit) · what was verified -->

## Handoff

<!-- Rewrite at the end of every session. -->

- Branch: `{{BRANCH}}` from `{{BASE}}`. The next agent keeps this branch: no new branches, no merges, catch up with `git rebase origin/main`.
- State: <what works, the first failing acceptance command, any uncommitted work (there should be none)>
- Next step: <the next bounded batch and its acceptance>
- Pitfalls: <what wasted time>
- Resume: <exact commands to get back into this state>
- Session: <agent, session ID, where its transcript is>
- Rules read: AGENTS.md at <commit>; processes stopped (`scripts/procs.sh` shows none): <yes/no>
