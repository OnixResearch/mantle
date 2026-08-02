# Design: Thin receipt-bound foreign realization adapter

## Context

The compilation change emits resolved native units and exact target paths. It does not prove that source payloads exist or that builders ran.

Mantle already owns lazy scheduling, substitution, fetch dispatch, sandbox execution, PathInfo persistence, castore export, and artifact attestations. The adapter should reuse those paths.

Current build-request construction adds Nix-style environment variables, a fixed work directory, and `ProvideBinSh`. Those defaults are not valid for every foreign derivation family.

## Decisions

### Decision 1: Reuse the ordinary scheduler and store

**Choice:** The adapter will register resolved native units in `DerivationRegistry`. It will use the ordinary `Builder` worker path for selected target roots.

The adapter will not add a foreign worker, recursive executor, or second PathInfo service.

**Rationale:** Scheduling and store admission are Mantle responsibilities. Foreign semantics end at concrete unit lowering.

### Decision 2: Keep realization orchestration in a thin shell

**Choice:** Pure functions will validate executable plans, source records, profile bindings, and receipt inputs. The shell will read files, open stores, materialize sources, configure the builder, run goals, and write reports.

**Rationale:** Realization has I/O and mutation. Policy decisions remain testable without store or process mocks.

### Decision 3: Require materialized source records

**Choice:** Every non-derivation source input will reference a verified Mantle source-bundle record or an admitted fixed-output fetch unit.

The adapter will ingest source bytes into castore, persist signed PathInfo, and record the exact target source path before parent registration. It will not read an ambient foreign store path.

**Rationale:** A foreign logical path does not supply bytes on a consumption host.

### Decision 4: Use typed per-derivation execution profiles

**Choice:** Human-authored execution policy will use a typed Nickel contract with deterministic runtime export.

Each profile will define shell provisioning, work directory, environment mode, protected variables, network policy, setid chmod behavior, syscall exceptions, writable prefixes, resource limits, and unsupported capabilities.

**Rationale:** Global compatibility switches hide authority and make mixed graphs unsafe.

### Decision 5: Bind profile identity into target derivation identity

**Choice:** The compiler will add a reserved internal profile-digest field to each target derivation before target path computation.

The field name will be versioned and reserved. The builder will verify the BLAKE3 digest against the supplied profile, then remove the internal field from the child environment.

A foreign derivation that already uses the reserved field will fail.

**Rationale:** Side metadata alone could build one target path under different execution semantics.

### Decision 6: Define a Guix-compatible profile without `/bin/sh`

**Choice:** The initial Guix profile will set `provide_bin_sh = false`. It will expose only declared builder and source paths.

A derivation that names `/bin/sh` must provide an explicitly mapped builder path or fail before execution.

**Rationale:** Ambient `/bin/sh` would hide a real Guix-to-Mantle compatibility gap.

### Decision 7: Extend build requests with explicit profiles

**Choice:** `derivation_to_build_request` will accept an explicit profile. Existing native callers will pass the current Mantle compatibility profile.

The profile will control normalized environment, work directory, constraints, network, writable paths, and supported exceptions. Remote foreign realization will fail as unsupported until profile transport is specified.

**Rationale:** Explicit parameters preserve existing behavior while removing implicit state.

### Decision 8: Keep source fallback deterministic and bounded

**Choice:** Fetch units will accept a bounded ordered candidate list. The first version will continue after transport or declared transient failures only.

A fixed-output mismatch will fail closed. It will not silently accept a later candidate under the same attempt.

**Rationale:** Content verification remains authoritative. Availability policy must not weaken integrity.

### Decision 9: Emit realization evidence separately

**Choice:** Emit `mantle-foreign-realization-receipt-v1` beside the ordinary build report.

The receipt will bind the import receipt, executable plan, source records, profile policy, selected roots, build-report digest, PathInfo identities, action dispositions, failures, and non-claims.

**Rationale:** Import admission and execution observations are different evidence domains.

### Decision 10: Keep system assembly outside Mantle

**Choice:** Mantle will stop at built and stored artifacts. OnixOS remains responsible for initrd, activation, account, Shepherd, VM, deployment, and boot evidence.

**Rationale:** This preserves ADR 0010 and avoids foreign package import becoming an OS module layer.

## Failure Semantics

- A stale import receipt or executable plan fails before store mutation.
- A missing, changed, unsigned, or incomplete source record fails before parent registration.
- A missing or mismatched execution profile fails before dispatch.
- A forbidden `/bin/sh`, network, syscall, writable path, or environment request fails closed.
- A fixed-output mismatch fails and does not enter PathInfo.
- Sibling root outcomes remain visible, but the overall command fails when any selected root fails.
- Cancellation uses the existing worker teardown path and cannot emit a success receipt.
- Unsupported remote execution fails before a remote request is sent.

## Risks / Trade-offs

- Profile-aware build requests touch the normal builder path and require regression tests.
- Source materialization can require new source-bundle record classes.
- Adding a profile digest changes all translated target paths when profile policy changes.
- Initial local-only realization limits distributed use.
