# ADR 0144: Keep native script reservations inside shared source controllers

## Status

Accepted (2026-11-06) for source-program authoring. This does not accept source
preparation, an executed provider, a fixed point, or release evidence.

## Context

Real Rust source-program preparation passed consuming-workspace resolution,
sealed target classification and producer-label admission, then exhausted the
recipe declaration capacity before constructing the original LLVM graph. Native
Rust build-script scheduling expanded its source-owned suspension reservation
into repeated script, observation, selection, native-controller and reply-merge
declarations. Reservation hints are not observations of actual requests.

The shared C source controller already preserves original command requests,
descriptors, workspace timestamps, normal statuses and protected executable
selection. Compact macro scheduling established that runtime reservations need
not multiply immutable recipe declarations (ADR 0140).

## Decision Drivers

- Retain source-owned request and native-command reservations and overrides.
- Retain recipe, protected-event, time, disk and byte ceilings.
- Execute original script and native images only as separately observed protected
  leaves, never as children spawned by a coordinator.
- Reuse the shared request/result format, controller engine and exact Produced
  image or authenticated generated-output authority.
- Preserve whole temporary state, readonly reply history and original descriptor
  semantics; require a genuine successful script completion.

## Decision

Author a constant-size initial workspace, script environment/cwd frame, shared
source controller and completion projection. The shared C source-entry hook
alternates original script execution with its genuine exit-75 native request,
acknowledges the actual native status and descriptor/workspace result, and
replays the original script. A native nonzero status is a result for the script,
not an invented successful command or automatic script completion.

The environment/cwd frame is a Produced Tree containing the exact script image.
It supplies the installed readonly image path, while the separate Produced
executable inventory remains the image authority. Environment-only text inputs
are consumed into the frame, not misclassified as readonly Tree roots.

For n reserved requests and m reserved native commands per request, the source
command upper bound is `(n + 1) + n * m`. Checked arithmetic rejects overflow.
The existing global controller command ceiling is an independent intersection
with that bound. Neither the source request reservation nor its per-request
native reservation is lowered; reaching any applicable bound fails closed.
Declaration capacity is constant instead of dividing remaining recipe space by
a per-request expansion factor.

Logical `/tmp` is the workspace's `tmp` subtree. Coordinator history is outside
that subtree and is exposed to the script through the readonly `workspace-seed`
input, never a mutable output operand. Captured request and acknowledgement
snapshots contain temporary state, not recursively nested coordinator history.
The writable workspace also contains a duplicate of that private history. Before
each script leaf the coordinator records exact names, kinds, modes and bytes
outside the workspace, then compares the restored result before accepting a
request or final zero. A readonly environment path alone is insufficient:
otherwise a script can alter the duplicate and forge a later replay result.
Final publication selects genuine completed checkpoint state, projects the
script's original output directory and all generated temporary state, removes
only private request/random protocol members, and forwards actual final script
stdout. Failed or unfinished scripts cannot publish completion products.

The native command inventory continues to compare exact Produced executable
bytes. Compiler/linker output callbacks remain the only generated-image
promotion mechanism. Finding an ELF file in a workspace is not executable
authority. Original command operand and environment typing remains fail-closed.
No new runtime operation, controller schema, host interpreter or subprocess
launcher is introduced. The expanded classifier/reply-merger path is removed.
Shared evaluator fatal errors use status 2, preserving status 1 exclusively for
the controller's genuine-completion predicate. An original script exit 1 is
still an actual script failure, never a successful controller completion.

## Consequences

The immediate consumer is Rust source-program preparation, including native
requests emitted by original Rust build scripts before the LLVM/successor
handoff. Mantle's Rust source-authoring and source-controller owners maintain
the adapter. Repeatability evidence must include helper state-transition checks,
a real protected controller smoke, and a separate unchanged-pin preparation
attempt. A bounded protocol fixture is not an original compiler or provider
construction proof; successful preparation still does not execute a provider,
compare stage binaries, reconcile a full proof or establish release eligibility.
