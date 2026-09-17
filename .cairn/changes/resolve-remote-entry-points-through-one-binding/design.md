# Design: Resolve remote entry points through one binding step

## Goal and scope

Concentrate remote name admission into one resolve step per entry point, with a
binding table and revocation. Change nothing about payload trust.

## Current behavior

Remote builder access uses one-use tickets with quotas. Transfer sessions bind
a canonical manifest to the job, attempt, fence generation, policy digest, and
store prefix. Substituters are configured by URL with trust keys. Base stores
carry descriptors and generations. Each path validates its own name shape.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Keep per-path admission | Existing ticket, URL, and layer checks | Rejected as sufficient: revocation and visibility stay split | Revocation fixture |
| One resolve step per entry point | Binding table with generation fences | Selected direction | Resolve and revocation fixtures |
| Route everything through one service | Single admission daemon for all paths | Deferred: larger operational change than this slice needs | Not blocking |
| Kerberos-style ticket exchange | Two-party ticket protocol | Rejected: more machinery than name resolution needs | Not blocking |

## Contract and component ownership

- Pure core: name shape validation, binding selection, generation comparison,
  revocation state, and resolve decision over in-memory facts.
- Shell: the resolve endpoint per entry point, binding persistence, and
  revocation commands.
- Boundary: a resolved handle admits a session only. Payload trust stays with
  signed PathInfo, content checks, and existing admission.

## Decisions

### Decision: Resolution binds a generation

**Choice:** A resolved handle records the binding generation at resolve time.

**Rationale:** Generation fencing already exists for worker and layer
freshness. Binding it at resolve time makes a stale handle detectable without a
new mechanism.

### Decision: Revocation is a retraction, not a deletion

**Choice:** Revocation retracts the binding. Admitted content keeps its
validity.

**Rationale:** Revocation removes future authority. Rewriting past admission
would break content identity and make evidence depend on current policy.

### Decision: Fail closed before payload work

**Choice:** Name validation, binding selection, and generation checks run
before any manifest, checkpoint, or payload handling.

**Rationale:** This preserves the existing bounded-transfer rule that no
payload work happens before session admission.

## Risks / Trade-offs

- One step adds indirection. It returns the same session facts the current
  paths already produce.
- Bindings need persistence. They use the existing durable state path with
  explicit generation.
- A revoked-then-recreated name is a new binding with a new generation.

## Non-Claims

- A resolved handle proves admission at resolve time.
- It does not prove later content identity, output trust, or execution success.
