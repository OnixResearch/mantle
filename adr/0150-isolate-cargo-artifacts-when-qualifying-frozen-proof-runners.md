# 0150. Isolate Cargo artifacts when qualifying frozen proof runners

Status: Accepted

## Context

The source-built fixed-point workflow freezes complete source trees with
normalized archive timestamps. Qualification of `71c2bd4d` ran successful Cargo
commands in a previously warmed target directory and preserved the frozen
source's pre/post BLAKE3. Nevertheless, its installed regular runner had the
same BLAKE3 as the rejected e9 runner. The shared release binary's timestamp
predated the new qualification while source members had epoch timestamps.

Those observations establish old artifact reuse, not which changed source the
installed runner compiled. A successful Cargo invocation, passing test binary,
source checksum or transfer checksum alone cannot discharge that distinction.
The preparation using that runner was stopped and its partial state retained.

## Decision Drivers

- Bind the operator runner to the declared frozen source, lock/vendor cohort
  and checked-in compiler environment before authoring a proof cohort.
- Preserve frozen input bytes and timestamps rather than retouching source to
  influence an external build tool's freshness decision.
- Keep ordinary developer caches outside proof-runner qualification authority.
- Retain exact commands and failed/interrupted observations without promoting a
  prior successful binary or a source-only profile into current proof evidence.

## Decision

Each frozen-source runner qualification uses a distinct Cargo target directory
that is absent before the run starts. Canonical CLI/contract/lifecycle gates use
a separate absent-before-start target. Both use the checked-in development
shell, the frozen lock/vendor inputs and bounded recorded build settings.
Disable `RUSTC_WRAPPER` and `RUSTC_WORKSPACE_WRAPPER` inside that shell so its
initialization cannot silently restore a compiler-cache wrapper.

Retain the target-directory precondition, compiler versions, complete commands,
source/archive identities, unchanged pre/post source identity and installed
runner BLAKE3. Do not reuse a partially populated target after a failed attempt;
select another fresh target and retain the failed attempt's evidence. Package
registry/source caches and the admitted Nix toolchain are not Cargo target
artifacts and retain their existing declared authority boundaries.

Install the newly qualified runner at its own immutable run path. Do not replace
an older recorded executable in place or reuse that runner's partially authored
cohort. Only the newly bound runner may author the replacement source-program
cohort, which still requires current physical/graph review before a cold launch.
Do not require every source revision to produce different binary bytes: an
unchanged digest is an observation to reconcile, not a universal failure rule.

## Consequences

- Mantle's operator proof workflow owns the fresh-target assertions and runner
  binding evidence. Ordinary local development may retain its existing caches.
- This can rebuild dependency artifacts and use more disk than a warm build;
  resource admission and proof bounds remain unchanged.
- The `71c2bd4d` fresh-target qualification scripts and exact task results are
  retained in the active change's evidence. A completed command is accepted only
  at its actual observed scope; interrupted preparation is not current intake.
- Fresh runner compilation establishes a bounded source/build observation, not
  compiler correctness, full-source bootstrap, fixed-point equality, protected
  action reconciliation or release eligibility.
