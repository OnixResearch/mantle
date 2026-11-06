# ADR 0140: Keep macro reservations inside compact source controllers

## Status

Accepted (2026-11-06) for source-program authoring and bounded controller
execution. This does not accept a provider, a fixed point, or release evidence.

## Context

The mrustc source author previously expanded every reserved proc-macro slot into
recipe steps. Preparation rejected `rustc_data_structures` because its authored
reservation needed 5612 slots while only 2624 slots fit the remaining expanded
layout. Source-site counts are reservation hints, not observations of actual
macro expansion. Lowering a hint or raising the recipe ceiling to obtain success
would hide the capacity problem rather than preserve the reviewed contract.

Mantle already has a compact immutable controller catalog, lazy instantiation of
protected command turns, measured phase observations, and controller-aware action
reconciliation for native continuations.

## Decision Drivers

- Keep source-owned reservation policy and all recipe, event, time and disk ceilings.
- Keep helpers I/O-only: original compiler and plugin images execute as separate
  protected, observed actions, never children spawned by a source helper.
- Retain exact plugin basename, source-package path and Produced image-byte admission.
- Preserve binary request/response history, environment, cwd, argv[0] and mutable tmp.
- Require genuine compiler exit zero for completion, including early completion.
- Reuse the existing controller engine and action evidence rather than add a
  macro-specific runtime or a new unreviewed execution authority.

## Decision

Author one `ControllerSpec` with a fixed original compiler slot and an inventory
slot containing the exact Produced plugin images. A small source-only Rust
controller implements the existing `--mantle-controller-*` protocol. It alternates
compiler and plugin requests, validates the exact executed phase inventory and
image, carries genuine workspace/timestamp observations and contiguous binary
history, and stops at the first actual compiler-zero result.

For a reservation of n plugin invocations, admit a checked command ceiling of
`2 * n + 1`, including the final compiler invocation. It must fit the unchanged
`CONTROLLER_COMMANDS_MAX`. The macro author needs five recipe declarations at
most: an optional empty tmp seed, compiler configuration, compact controller, and
two extraction leaves for translation and tmp. Supplied tmp needs no empty seed.
The reservation-capacity calculation uses the controller command ceiling, not
expanded declaration count. The source policy and its overrides are unchanged.

Compiler configuration consists of one compact serialized `RecipeInvocation`
JSON line followed by a newline and the original compiler argv[0] bytes. A typed
TextSlice consumes those real bytes from the standard controller `data` output.
The command therefore retains its original argv[0] without inventing a dummy
input use or weakening unused-input admission. Only typed translation output
operands are remapped to the controller workspace. Original arguments, metadata,
environment and cwd retain their meaning.
The deferred native decoder receives that declared original namespace as
comparison data: `/outputs/translation` for a direct compiler leaf or
`/outputs/workspace/translation` for a compact controller. Response source/output
operands must match the declared namespace. Response and captured environment
path projection use only that namespace, never generated-text autodetection.

Workspace timestamp frames use the executor's sorted-child-first traversal,
including the root last. Lexical ordering of complete paths is not equivalent.
History remains readonly protocol state; changes made by a command are rejected.
A plugin stdout response is appended only after one actual successful plugin
leaf. Failed, skipped, double, wrong-image or unrequested execution cannot mark
completion. Exhausted command or macro reservations fail closed.

Controller CLI errors exit 2 explicitly. Start/resume exit 0 for a pause and 1
only for genuine completion; prepare exit 1 means unselected. Rust main's default
error exit 1 must never be allowed to stand in for these control predicates.

## Consequences

The current consumer is Rust source-program preparation, first at the mrustc
Rust 1.90 compiler graph. Mantle's Rust source-authoring and source-controller
owners maintain the adapter and protocol. Existing lazy execution, producer
identities, guards, per-turn output facts, action reconciliation and resource
limits remain authoritative; no new ControllerSpec fields or runtime operation
are introduced.

Regression evidence combines author boundary tests, emitted-helper protocol
validation, and an opt-in real protected smoke. The smoke uses a parent-built
static fixture compiler/plugin implementing the request protocol, exercises the
real author and controller engine in the seed-stage test lane, and feeds actual
events through `ActionObservationState`. It is not a Rust compiler bootstrap or
provider-construction receipt. Real source preparation remains a separate
integration check, and a prepared program is still not an executed provider or
fixed-point proof.
