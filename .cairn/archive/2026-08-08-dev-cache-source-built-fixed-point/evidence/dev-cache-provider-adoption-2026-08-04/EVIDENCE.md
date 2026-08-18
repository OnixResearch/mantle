# Dev cache provider-adoption increment evidence (2026-08-04)

Second increment for `dev-cache-source-built-fixed-point`. Builds on the
foundation increment. This records what changed and what validated.

## What is implemented and committed

Follow-on commits on `pi/dev-cache-source-built-fixed-point` (after the
foundation commit and a push).

- Provider cache now has a **writer**: after a successful cold dev run,
  `write_dev_provider_cache` publishes the StageX and native provider store
  subtrees plus a receipt-validated entry keyed by the current source-authority
  + policy digests.
- Provider cache now has a real **runtime adoption path**: on a validated hit,
  `run_attempt` skips the StageX transition, StageX provider publication, and
  native-provider build, copies the cached provider store subtrees into the
  fresh staging store (`adopt_cached_provider_subtrees`), and continues with a
  fresh host-tools + Rust provider + Cargo-free fixed point built against the
  adopted providers.
- Adopted runs are routed to a **dev-labeled outcome**: they record a
  dev-cache-hit transcript and a `dev-adopted-report.json`, and explicitly do
  NOT write the promoted deterministic receipt and do NOT publish to the
  `latest` / `latest-source-built-fixed-point` success aliases. A cached
  adoption can therefore never be mistaken for a fresh promoted proof.

## Validation evidence (same-turn)

Working env uses a gcc/mold linker override because the documented clang/mold
store paths were GC'd on this host.

- `cargo test -p mantle --bin mantle source_built_fixed_point` -> `46 passed;
  0 failed; 1 ignored` (includes the new adoption-copy test)
- First-party clippy reports no findings in
  `source_built_fixed_point_shell.rs` or `source_built_fixed_point_dev_cache.rs`.
  `-D warnings` still fails on three pre-existing violations in
  `full_source_rust_binding_shell.rs`, `realization_routing.rs`, and
  `stagex_transition.rs`.
- `rustfmt --check` on both touched leaf files -> clean; `git diff --check` ->
  clean.
- `cairn validate` and the proposal/design/tasks gates were already PASS and
  remain so.

## Not re-run in this increment

- The multi-hour full-source fixed-point proof and a full cold -> cached ->
  adopt round trip are not executed here; the adoption writer/lookup/copy logic
  is exercised at the unit level only.
- A fresh emitted promoted bundle is not reproduced.
- Cross-run multi-stage resume beyond the same-staging-dir transition resume is
  still partial (per-stage markers are persisted in the staging dir; the
  transition execution-tree snapshot needed for a fresh-dir resume is not yet
  stored in the cache).
- `bootstrap_eval` `gcc_native_diagnostic_uses_runtime_tcc_without_autotools_claims`
  still fails on this tree (asserts only on `bootstrap/gcc-4.0-native.ncl`
  content, untouched here; pre-existing).

## Contained real-store dogfood (same turn)

A full multi-hour source-built proof cannot run on this host: bubblewrap is not
installed and the handoff source closures (`native-source-closure.json`,
`stagex-source-closure.json`, `rust-source-archives`) are absent on this
machine. So I ran a small, contained real-store test instead of the full proof.

- Dogfooded the built binary end to end with the new flags: `self-build
  --source-built-fixed-point ... --dev-provider-cache ... --dev-resume
  --dev-fast-fail` accepted the flags, passed option validation, hit the
  fail-closed disk preflight (default 1 TiB bound vs ~16 GB on /tmp), then with
  `--proof-disk-bytes-max` passed preflight and failed closed cleanly at the
  missing source profile (`attempt_not_started=true`). Writing it: the new entry
  path and fail-closed gates work with the real binary.
- Added and ran a contained REAL-store test
  `store_service_registration_adopts_real_provider_and_is_observable`: it
  creates a real state/output store, generates a real signing key, and registers
  a provider through the dev-cache store path
  (`register_adopted_provider` -> `adopt_verified_local_provider_path_strict`
  -> `StoreHandle::adopt_verified_local_output`: ingest, NAR, sign, persist,
  plus `import_constructed_store_path_source`), then asserts the logical store
  path and that `pathinfo.redb` persisted. This proves the store-service
  registration is real and observable, not a plain file copy.
- `source_built_fixed_point` tests now 46 passed (includes the real-store test);
  touched files clippy/fmt-clean; `git diff --check` clean.

This remains prior to a full cold->cached->adopt round trip, which still needs
bubblewrap and the handoff source closures on the run host.

## Remaining work for full completion

- Store a transition execution-tree snapshot so `--dev-resume` can restart from
  a fresh staging dir, not only the same interrupted dir.
- Confirm the adopted path with a real emitted run and a fresh promoted cold
  bundle.

## Store-service rework (same day)

Addresses the review point that the snapshot/seeding were raw tree copies
instead of content-addressed store reuse.

- `seed_dev_store_snapshot` now adopts the cached provider store subtrees
  through the store service: `adopt_verified_local_provider_path_strict`
  (-> `StoreHandle::adopt_verified_local_output`) plus
  `import_constructed_store_path_source`, the exact seam the cold path uses for
  freshly constructed transition/provider paths.
- The coarse whole-tree snapshot of `native-store` + `native-state`
  (`write_dev_store_snapshot` / `dev_store_snapshot_root`) was removed; the
  provider cache (`write_dev_provider_cache`) is now the content-addressed
  capture of the expensive store content, and `prepare_attempt` always imports
  source records via the content-addressed `import_source_bundle` first.
- Adoption copies (idempotent) and observes the seeded providers; store
  registration happens once at prepare-time seeding.

Validation: `source_built_fixed_point` tests 45 passed; touched files
clippy/fmt-clean; `git diff --check` clean. The store-service registration itself
is NOT unit-tested (it requires a real persisted store with a signing keypair);
only the no-cache-disabled path and the adoption-copy path are unit-tested.
Note: `import_constructed_store_path_source` asserts one record import per
fresh state dir, which holds for the intended fresh-staging flow.

## Full-proof round trip attempt (same turn)

The full cold->cached->adopt round trip is not yet demonstrable end to end on
this host: prior full proofs failed, and this machine lacks a real bubblewrap on
PATH (though `nix shell nixpkgs#bubblewrap` provides it) and the handoff source
closures are not under `/media/handoff`.

Latest prior run `~/.cargo-target/mantle-source-built-fixed-point-runs-v19a`
got through StageX -> provider -> native-provider admission (native provider
`82b08dcd...`) and then FAILED constructing the Rust provider:

```
constructing full-source Rust provider: launch
 .../rust-provider-scratch/run-mrustc-first-stage.sh: Exec format error (os error 8)
```

Root cause: the generated `.sh` script's shebang embeds the absolute
native-store busybox path, which is 266 bytes long. Linux rejects shebang lines
over ~256 bytes with ENOEXEC. The busybox interpreter itself is fine (running it
explicitly builds the first stage successfully).

Fix applied in `src/rust_source_provider.rs`: `run_generated_script_with_log`
now launches generated scripts through an explicit absolute interpreter in
argv[0] (the full-source busybox `sh`) instead of relying on the shebang, for
all three build stages (first-stage, rustc stage1, rustc final). `rust_source_provider`
tests: 94 passed; touched file clippy/fmt-clean.

This fix is a pre-existing parent (`prove-source-built-mantle-fixed-point`)
blocker surfaced while dogfooding; it sits in the dev-cache worktree to unblock
the run. A full successful cold proof, and therefore a populated cache and a
cached->adopt round trip, is not yet achieved; that still needs a valid source
profile + independent expected digests and a full successful multi-hour run.

## Cold proof run on the parent graph (2026-08-04, task 8569 / detached 115495)

The dev-cache change was rebased onto the parent branch
(`agent/finish-source-built-fixed-point-20260804`, which carries the equivalent
interpreter fix `33d04663`), so the branch has the 59/66-record graph plus the
dev-cache code and the fix together. The cold proof was launched with the v33
profile (`dcf1b356...`), StageX lineage `e477ab39...`, native provider
`82b08dcd...`, real bubblewrap, busybox-static, and a 200 GB disk bound.

The first attempt (pueue task 8569) was killed by a pueue daemon restart during
prepare (the known shared-daemon hazard); it was relaunched detached via setsid
(pid 115495) so daemon loss cannot interrupt it. It then ran ~1.7 hours:

- prepare parsed the 10.3 GB profile and PASSED the source-union gate (v33
  matches this branch's graph; no `expected=53, profile=59` mismatch).
- the StageX transition ran its full protected-exec stage tree (hex0, kaem,
  mes, musl, tcc, binutils, bash, coreutils, and the rest), which confirms the
  interpreter fix works in a real run (the busybox sh invocations executed).
- it then FAILED at the StageX transition audit:

```
StageX transition failed: invalid StageX protected-exec audit:
  GNU binutils event count is outside the accepted closure: observed 74066
```

This is a parent-change StageX protected-exec audit calibration bound: the
accepted binutils event count closure was calibrated against a different graph,
and this run's count (74066) falls outside it. The dev cache is not implicated.
Completing a cold run on this host requires the parent change to calibrate the
StageX binutils event-count closure for the current graph — careful audit
bound work, not a dev-cache fix.

## Persistent dev-store incremental reuse (2026-08-04, commit eda2a1ae)

Review point: the dev cache was effectively all-or-nothing (post-completion
provider snapshot), so a run that died mid-pipeline lost the completed packages.
This change makes dev runs share a persistent content-addressed store instead.

- Dev runs (`--dev-provider-cache`) use a persistent `<cache>/dev-store` +
  `<cache>/dev-state` rather than a fresh store per attempt. Because the store
  is content-addressed, each successfully built package is a no-op hit on the
  next dev run, so a run killed mid-pipeline keeps its completed work.
- Dev runs accept cache-hit build outcomes (`parse_build_report` and
  `require_single_build_output` gain an `allow_cached` flag driven by the dev
  flag). The promoted cold proof keeps a fresh store and still rejects cache
  hits.
- Dev runs are dev-labeled (no promoted receipt, no release alias) because they
  may reuse cached packages. `write_dev_provider_cache` still publishes the
  StageX + native provider subtrees so the protected-exec transition can be
  skipped on adoption.

Validation: `source_built_fixed_point` tests 49 passed; clippy `-D warnings`
clean on this branch; `rustfmt --check` clean. Not yet exercised end to end
(the persistent store hits only appear across real repeated dev runs).
