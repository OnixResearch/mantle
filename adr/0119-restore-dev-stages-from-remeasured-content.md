# ADR 0119: Restore dev stages from remeasured content

## Status

Accepted

## Context

Mantle development runs can reuse a persistent store and receipt-validated provider outputs. Attempt-local markers cannot restore stage state into a new staging directory.

Promoted checkpoints already have bounded payload I/O. Their authority is restricted to cold proof workflows and must not become a dev-cache shortcut.

## Decision

Mantle will use a separate dev-resume contract and cache namespace.

A `no_std + alloc` core will validate bounded resume candidates. Each candidate binds the current source, plan, policy, stage, producer, output, payload, and bundle identities.

The core will select only a contiguous stage prefix. Its plan will list restored stages, executed stages, and the first incomplete stage.

The shell will publish one content-addressed manifest for each completed stage. Shared payload objects will use BLAKE3 identities. The shell will remeasure each selected object before restore.

The shell reuses bounded checkpoint observation and copy mechanisms. Dev prefixes use `mantle-dev-provider-prefix-v1` and a separate namespace. The promoted four-stage format remains unchanged.

Each provider prefix requires its exact payload set: 1, 2, 14, or 17 payloads. The native boundary includes the Rust host tools and their evidence. The Rust provider, toolchain closure, and Rust action evidence enter only at the fourth boundary.

Shared objects use the payload kind and BLAKE3 digest as their key. Prefix manifests refer to those objects instead of copying earlier trees again. Restore selects the exact checkpoint reference from the chosen resume manifest.

Mantle stage 1 has an explicit publication callback. The callback must finish before stage 2 starts. Publication failure prevents continuation. Each attempt records only the manifests that it publishes.

Dev native evidence can retain missing action events from cached derivations. Deterministic replay must reproduce those observations exactly. Promoted admission still requires complete reconciliation.

Binding relocation modifies only payloads that the restore created. Identical pre-existing payloads remain unchanged. Native-prefix host paths refer to restored tools and evidence, not an earlier attempt.

A promoted run will reject dev resume options before it reads any dev cache path.

## Consequences

Fresh dev attempts can resume from validated stage state. A missing, stale, partial, modified, unknown, or conflicting candidate falls back to execution.

Reports distinguish restored stages from stages executed in the current attempt. Restored work is not relabeled as current execution.

The cache uses extra disk for immutable payload objects and manifests. Existing proof disk limits remain authoritative.

## Rejected alternatives

### Trust stage markers

Markers do not contain the state needed by later stages. They are not execution evidence.

### Trust persistent store presence

Content presence does not prove stage, producer, or policy linkage.

### Read promoted checkpoints as dev cache

This would mix dev reuse with promoted proof authority.

### Duplicate tree-copy logic

A second copy implementation would create a new race and symlink surface. The dev shell reuses the bounded checkpoint mechanisms instead.
