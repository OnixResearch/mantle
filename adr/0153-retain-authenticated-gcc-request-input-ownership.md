# 0153. Retain authenticated GCC request input ownership

Status: Accepted

## Context

The GCC specs author accepts either static typed operands or a canonical
invocation emitted by an authenticated source producer. Static argv and
environment are deliberately empty in the latter case: the author must not
modify those request bytes.

The input-pruning pass nevertheless inspected only the empty static words.
The current regenerated native recipe's relocated compiler smoke request names
`unit.c` and `target`, but its specs consumer retained neither binding. The
unchanged invocation decoder rejects that request. Dependency-unreachable
status does not avoid it because the recipe scheduler visits every step.

A real decoder regression executed before the fix in task 1612 and failed with
`source recipe: unknown input unit.c`. Successful source preparation and source
availability verification had not exercised this consumer boundary.

## Decision Drivers

- Preserve the authenticated compiler argv/environment exactly.
- Bind every addressed input to its original owner on the consuming specs step.
- Keep unused-input rejection, namespace/path checks and source authority strict.
- Retain static dependency pruning instead of granting every phase-only label.
- Reuse the GCC response author's existing explicit authored-input vocabulary.
- Avoid cloning complete typed words merely to inspect their input names.

## Decision

The authenticated GCC request author receives an explicit source-authored input
read set alongside the canonical request artifact. Every named entry must
already exist in the invocation's input map; this set grants no new artifact or
source authority. The specs step retains those entries in addition to its
executable, request, runtime mounts, cwd and guard dependencies. The same
addressed names remain available to the exact subsequent GCC phase expansion.

The relocated smoke author derives its read set from the actual typed request
arguments and environment before serializing them. The deferred Mantle native
request author declares its source producer's bounded input scope. Static GCC
invocations continue to derive their names from their own words and prune
unrelated labels. Input-name inspection borrows words rather than cloning them.

The decoder is unchanged: removing a required binding must still reject, and
adding an unused declaration must still reject. The regression constructs the
shared author's specs consumer and decodes the actual canonical request through
that production decoder, including both negative authority boundaries. It
replaces incidental map/name-copy assertions with consumer-visible behavior.

## Consequences

- The shared GCC phase author owns this contract; its two authenticated-request
  callers migrate together. No compatibility overload or consumer exception is
  retained.
- The immediate consumer is the original generator-base relocated compiler
  smoke, also used by deferred Mantle native compilation. The durable capability
  is explicit request ownership across an opaque-source invocation boundary.
- The source-built test leg is the repeatable regression rail. Task 1612 retains
  the actual pre-fix failure; subsequent verification evidence must record its
  actual result rather than infer success from this decision.
- Product source and generated program identities change. Preserve the rejected
  `9300895d…` cohort; publish/freeze/qualify and regenerate a replacement before
  any launch acceptance. A local repair does not alter frozen input data.
- No expected source/provider pin, original request bytes, resource bound,
  hermeticity rule, fallback policy or consumer admission is relaxed. Neither
  this decision nor a developer regression establishes compiler correctness,
  protected provider execution, fixed-point equality or release eligibility.
