# ADR 0121: Bind cold command continuations to actual producer outcomes

- **Status:** Proposed
- **Date:** 2026-10-20

## Context

The cold source route must admit all six stage action inventories before its first
source command. Original configure, CMake and build-script algorithms choose later
commands from actual compiler and executable-probe results. An absent object after
an accepted negative compiler probe is not a successful generated object. Imported
configuration, recorded success codes, opaque shell/make children and replaying a
compiler through another executable do not satisfy the action authority boundary.

The preparation executable's host is also not necessarily the source-produced
compiler's host. A GNU-hosted preparer can construct a musl-hosted compiler. Host
build scripts, procedural macros and their dependency libraries must be planned
for the declared compiler host before graph construction, rather than retagged
following planning.

## Decision drivers

- Preserve complete predeclared action, executable, dependency and resource bounds.
- Execute each real original command under its own protected executable identity.
- Preserve negative probes, conditional non-execution and immutable workspaces.
- Keep ordinary successful output/event coverage exact.
- Bind compiler-host decisions to authenticated source policy, not ambient state.

## Decision

An execution leaf can consume a canonical `RecipeInvocation` from an exact prior
Produced Text input. It supplies only typed arguments and environment words.
Executable, working directory, standard input, runtime mounts, output names,
producer dependencies and guards remain in the immutable action plan. Static
arguments/environment are empty when this mode is selected. Before launch the
executor authenticates the request bytes, checks bounded canonical decoding and
existing typed-word/path rules, and requires the fixed and actual dynamic operands
to consume every declared input name. An unused label cannot establish a
source dependency. This is a runtime obligation; the static expansion alone does
not establish that a dynamic operand was used.

Literal words retain the ban on untyped path separators. A `Runtime` word names
an exact declared runtime mount and a normalized relative member. Read-only,
producer-authenticated bounded runtime Trees can supply genuine original loader
and compiler search directories; they do not grant directory-prefix executable
authority. Destinations are restricted to the existing library roots, descendants
of the proof's admitted logical store prefix and `/mantle-rust-runtime`, without
overlapping mounts or shadowing reserved sandbox namespaces.

`TextInput` supplies actual source-program/data bytes from an exact prior ordinary
Produced Text artifact. `TextSlice` selects a checked UTF-8 byte range from that
same authenticated text, allowing original dynamic environment values without
inventing output names or discarding the environment. The whole producer text is
read and authenticated once per invocation, shared across its slices. Actual
filesystem operands still use Input, Runtime or Output words. A data-text label
does not prove a filesystem dependency or the semantics of a program interpreting
those bytes; fixed mounts and exact protected executable authority remain the
enforcement boundary.

`DirectorySeparator` must immediately follow a typed Tree operand and be final
or followed by the exact literal `:` path-list delimiter. This preserves original
compiler directory-prefix and path-list spelling, including empty components,
without permitting arbitrary slash concatenation or weakening the literal/member
path rules. `ExtractArchive` keeps raw pinned archives as source authority,
reuses the existing in-process decoder, and preserves the original root under the
entry and expanded-output bounds. The existing XZ decoder's whole-buffer
allocation is not a bounded-memory claim.

Ordered guards consume actual producer exit-status bytes. Their canonical normal
codes and hashes participate in reconciliation. A skipped phase emits an explicit
control observation, not a protected launch, completion or invented compiler
output. Direct conditional artifact use requires a dominating guard. An
`ObservePhase` operation binds the actual executed/skipped outcome and can select
either the actual produced state or the declared prior state. It cannot synthesize
the missing output of a skipped phase.

A source-owned classifier may select one exact member of a finite inventory of
individually Produced executable inputs and publish that byte-identical ELF as a
new Produced executable. The next action still binds one immutable producer and
output identity and launches that actual ELF directly. No interpreter, wrapper,
child launcher, path-prefix authority or imported binary substitutes for the
selected image. Actual output and protected-image digests, source request identity,
upstream producer ancestry and unchanged resource bounds remain required.

A source-owned normal exit policy explicitly accepts exact or bounded alternative
codes. Accepted nonzero probes can omit ordinary outputs, but must retain their
actual standard streams and exit status. Observed inventory, not the allowed code
alone, establishes which products exist. Normal success keeps complete output
coverage. Output seeds copy exact prior Produced artifacts into a fresh bounded
output workspace without mutating the producer. Merged capture uses one shared
open-file description for both descriptors; concatenating separately captured
streams is not equivalent to the original write ordering.

Original timeout wrappers lower to inclusive immutable `timeout_groups` over
their complete controller, command and observation spans. Nested/disjoint groups
share a monotonic deadline across all resumed leaves; each worker receives the
minimum remaining duration, never a fresh per-leaf allowance. Any actual expiry
fails the strict proof, including an optional wrapper's timeout. This conservative
policy does not claim the original algorithm's timeout recovery, fabricate normal
code 124 or authorize outputs from an expired group. Successful acceptance still
requires the complete selected run within its original source-derived limits.

The authenticated Nickel source `bootstrap/source-built-continuations.ncl`
declares finite macro and native-script reservations. Source site/file counts are
resource-sizing hints, not proved expansion/command counts. Exact phase overrides
precede checked arithmetic; zero, overflow and capacity excess are errors rather
than clamping or silent graph pruning. Exhaustion fails the attempt. The existing
recipe/action/event/time/disk ceilings remain authoritative, and only a complete
actual run establishes that a selected reservation was sufficient.

`RustPlanOptions.host_triple` is required. Source-only authoring obtains it from
the original typed Rust source route; cold admission carries the same host from
that admitted route into Mantle topology validation. Host and host-dependency
units are created for that host. Target units retain their independently selected
target. Ordinary CLI planning explicitly supplies its existing default host; this
decision does not claim new native Darwin execution or arbitrary compiler-host
discovery.

Rust successor and Mantle compiler actions preserve the original target-wrapper
argv, environment, cwd and retained link workspace. The native controller
interprets that source-owned wrapper and executes its individually authenticated
native leaves; it does not reconstruct a substitute GCC command from a response
file. The Rust provider exports the actual assembled native package, controller
and source boundary helper consumed by the Mantle stages. Cargo package sources
remain separate capabilities, and `CARGO_MANIFEST_DIR` resolves to those genuine
source directories rather than the native SDK package.

Native command data can include admitted Source or ManifestSource references;
ordinary recipe admission still authenticates their role, member and kind.
Read-only directory roots are explicit Trees. Controllers, executable inventory,
captured requests and mutable workspaces retain their existing Produced authority
requirements. Source-data admission never grants executable inventory authority.

## Consequences

Source controllers must still faithfully implement the selected original shell,
Make, CMake and build-script algorithms. A typed invocation mechanism, helper
source, inactive slot or well-formed plan is not a completed source author or an
executed bootstrap proof. No action/resource limit or original graph is relaxed by
this decision. Fixed-point acceptance still requires the real complete cold run,
producer-linked execution reconciliation, stage1-owned stage2 and binary equality.

Mantle bootstrap/protected-execution maintainers own these contracts. Deterministic
host/graph regressions and the real static-leaf/bubblewrap guard, request,
workspace and stream scenarios provide repeatable bounded checks. Only captured
execution results establish that those checks passed; this ADR is not evidence
of six-stage completion, compiler correctness or release eligibility.
