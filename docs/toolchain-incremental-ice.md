# Toolchain incremental-compilation panic in the pinned nightly

Status: open, reproducible on the pinned cohort, unrelated to the source under
test.

## Symptom

With the pinned toolchain `rustc 1.96.0-nightly (c75612477 2026-04-07)` on
`x86_64-unknown-linux-gnu`, an incremental build of `crates/crunch-delta`
panics inside the compiler:

```
error: the compiler unexpectedly panicked. This is a bug
note: please attach the file at `.../rustc-ice-<timestamp>.txt`
note: compiler flags: --crate-type lib -C embed-bitcode=no -C debuginfo=... -C incremental=...
```

The recorded panic frames name the incremental dependency graph:

```
query stack during panic:
#0 [analysis] running analysis passes on crate `crunch_delta`
there was a panic while trying to force a dep node
try_mark_green dep node stack:
#0 check_mod_deathness(crunch_delta[c6f5]::fixtures)
```

## Where it was observed

- The ordinary first-party gate (`./scripts/check-first-party-quality.sh`)
  reached the strict Clippy phase in a fresh target with the default
  incremental profile and failed with this panic for `crunch-delta`.
- The same panic was observed earlier in the same campaign when a build
  directory had been edited underneath an existing incremental cache.

## Workaround

Run the gate with incremental compilation disabled:

```bash
CARGO_INCREMENTAL=0 ./scripts/check-first-party-quality.sh
```

With incremental compilation off, the Clippy phase compiles `crunch-delta`
without the panic and the gate proceeds. The same setting is what the guarded
verification harness uses for the source-bound runs.

## Non-claims

- This is not a defect in `crates/crunch-delta`; the panic is inside the
  compiler's incremental dependency graph, and the crate compiles normally
  without incremental state.
- The workaround changes build strategy, not the checked source, the toolchain
  version, the crate features, or the lint set. It does not relax any gate
  condition.
- Upstream issue reporting is not part of this repository's scope; the panic
  artifacts are retained with the gate transcripts.
