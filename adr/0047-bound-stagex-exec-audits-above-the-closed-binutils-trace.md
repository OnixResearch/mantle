# ADR 0047: Bound StageX exec audits above the closed binutils trace

## Status

Accepted (2026-07-30)

## Context

The protected StageX seccomp supervisor stores one audit event for each `execve` or `execveat` request. The diagnostic observer originally limited this audit to 65,536 events.

Complete authenticated binutils 2.30 diagnostic runs used 74,002, 74,027, 74,041, or 74,056 events after adopted descendants were reaped and the audit became quiescent. The bounded difference follows the existing sed closure of 4,891 or 4,892 invocations and bounded contention in the sed bridge invocation lock. All 68 executable identities, generated-child counts, producer order, and output identities stayed unchanged.

A pre-quiescence diagnostic snapshot contained only 73,996 events. Mantle rejects incomplete snapshots and waits for a bounded stable audit count after reaping descendants.

A later complete, quiescent provider-role rerun recorded 73,991 events. Compared with the passing 74,022-event v81 suffix, only the declared coreutils `mkdir` authorization changed: 5,411 events became 5,380. All other authorization counts, outputs, sed calls, identities, and fallback facts matched. The lower accepted bound includes this observed lock-and-directory contention variant.

The old limit denied later `ld` generator children after the audit reached its bound. The denial was correct, but the limit was smaller than the authenticated execution graph.

An unbounded audit is not acceptable. A limit that is equal to the observed count also leaves no space for a fail-closed diagnostic event or a reviewed extension.

## Decision Drivers

- Keep memory and audit growth bounded.
- Keep limit exhaustion fail-closed.
- Admit the complete authenticated binutils execution graph.
- Use one explicit limit for diagnostic and protected supervision.
- Keep the limit independent from host memory or ambient configuration.

## Decision

Mantle limits each diagnostic or protected StageX exec audit to 131,072 events.

The supervisor denies later execution requests when the audit reaches this limit. It does not allow an unaudited request. The limit is two times the old power-of-two bound and is greater than the maximum accepted binutils trace of 74,056 events.

The binutils audit validator accepts only the overall bounds `[73,991, 74,057]` and the narrower bounds for the observed sed invocation count. It checks exact counts for every stable identity and narrow bounds for Bash, `chmod`, `cp`, and lock-related `mkdir` events. It also requires the exact 68-identity set, exact generated-child and helper counts, producer order, audit quiescence, and no denied or fallback events. The larger supervisor limit is capacity, not permission to change the accepted binutils graph.

## Alternatives Considered

### Keep the 65,536-event limit

Rejected because it denies the complete authenticated binutils graph before `ld` finishes.

### Remove the event limit

Rejected because audit memory growth would depend on child behavior.

### Set the limit to one observed event count

Rejected because the authenticated sed, lock, and directory-creation graph has a small bounded count range. One observed value also gives no bounded margin for a fail-closed diagnostic event or a separately reviewed successor stage.

## Consequences

- Diagnostic and protected audits remain bounded.
- Limit exhaustion denies execution.
- The binutils transition must stay inside `[73,991, 74,057]`, match its sed-specific bounds, and pass every exact identity, bounded contention, and order check.
- A successor graph above 131,072 events requires a new reviewed limit or a smaller execution graph.
- This decision does not prove that a full protected binutils transition passed. That claim requires current protected evidence.
