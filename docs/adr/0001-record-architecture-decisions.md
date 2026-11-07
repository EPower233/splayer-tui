# 0001. Record architecture decisions

- Status: accepted
- Date: <YYYY-MM-DD>
- Deciders: <user>

## Context

Agents and people work on this project across many sessions. Without a record, each session re-derives earlier choices or silently reverses them.

## Decision

Every decision that changes the structure (layers, top-level directories, frameworks, storage, external services, cross-feature calls) gets a numbered ADR in `docs/adr/`, written from `0000-template.md`. The ADR is proposed before or together with the change, and the user accepts it. Accepted ADRs are not edited; a new ADR supersedes them.

## Alternatives considered

- Decisions only in chat or commit messages: lost when sessions or agents change.
- One growing design document: hard to see what was decided when, and why.

## Consequences

Decisions can be reviewed and searched, and changing direction costs one short document. Agents must check `docs/adr/` before changing structure.
