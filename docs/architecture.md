# Architecture

## Purpose

<What the system does, in two sentences. Requirements: `docs/spec/product.md`.>

## Context

<Who uses the system and which external systems it talks to. A small diagram is enough.>

## Dependency rule

Source dependencies point inward: `main → adapters → application → domain`. `main` wires everything together and is the only place that names concrete adapters. The architecture step of `./scripts/check.sh` enforces this.

## Layout

```
src/
  <feature>/
    domain/          entities, value objects, domain services (pure code)
    application/     use cases, one per file
      ports/         interfaces the use cases need from the outside world
    adapters/
      inbound/       http, cli, jobs → use cases
      outbound/      database, external APIs, files → implement ports
  shared/kernel/     the few value objects several features share
  main/              configuration, wiring, process entry points
tests/               mirrors src/: unit (domain, application), integration (adapters), e2e (smoke)
```

## Features

| Feature | Use cases | Ports | Adapters |
|---|---|---|---|
| <reminders> | <create, list, cancel> | <ReminderRepository, Notifier, Clock> | <SqlReminderRepository, SmtpNotifier, http routes> |

## Where new code goes

| You are writing | Put it in |
|---|---|
| A business rule, calculation or validation (no I/O) | `<feature>/domain` |
| The steps of one user action | `<feature>/application/<use_case>` |
| Something the use case needs from outside (storage, clock, API) | an interface in `<feature>/application/ports` |
| The real implementation of that interface | `<feature>/adapters/outbound` |
| Turning a request or command into a use-case call | `<feature>/adapters/inbound` |
| Configuration, connections, choosing implementations | `main/` |

## Cross-cutting concerns

- Errors: <domain errors are plain values; adapters translate infrastructure errors; the inbound edge maps both to responses>.
- Logging: <in adapters and main only>.
- Configuration: <read only in main and passed in>.
- Transactions: <owned by the use case through a unit-of-work port>.

## Testing

| Layer | What | How |
|---|---|---|
| domain | business rules | plain unit tests |
| application | use-case steps | unit tests with in-memory fakes of the ports |
| adapters/outbound | real reads and writes | integration tests against real dependencies (containers) |
| adapters/inbound + main | requests flow end to end | a few request-level tests and e2e smoke tests |

## Fitness functions

Checks that keep the architecture true, all run by `./scripts/check.sh` or CI:

- Layer dependencies (architecture step).
- <No cycles between features.>
- <Build and unit-test time budget: under 2 minutes for one feature.>

## Decisions and open questions

Decisions: `docs/adr/`. Open questions: <list>.
