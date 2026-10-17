# Offline APK adapter admission evidence — 2026-10-10

Owner: Mantle APK adapter (`crates/crunch-android`, `crates/crunch-android-core`). Outcome: the offline acceptance boundary is typed pre-effect rejection and declared-input-sensitive derivation identity, not synthetic tool output. Durable capability: a reproducible real-builder fetch/sandbox-extraction regression rejects invalid same-run provenance before SDK dispatch. The separate `prove-apk-adapter-real-build` change owns official SDK execution and a signed-APK claim.

The test ran on Leviathan x86_64-linux in an isolated Mantle worktree with the repo-pinned Nix dev shell. `bwrap --ro-bind / / -- /bin/true` succeeded. The static BusyBox was the `pkgsStatic.busybox` output of the repository's locked Nixpkgs, `/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox`, explicitly supplied as `SNIX_BUILD_SANDBOX_SHELL`. The previous hard-coded BusyBox store path was absent on this host; the first attempt failed before extraction and was not counted as evidence. Fixture SDK executables now fail with exit 99 if invoked; neither argv logs nor placeholder APKs can satisfy the test.

Captured command/result (`/home/brittonr/mantle-cairn-finish/logs/apk-adapter-focused-v2-20261010.log`; the Nix dev shell used local mirrors of the same pinned flake input revisions and `--no-write-lock-file`):

```text
$ bwrap --ro-bind / / -- /bin/true
$ cargo test --locked --offline -p crunch-android --lib sandbox_rejects_swapped_cached_untrusted_or_drifted_extractions -- --nocapture --test-threads=1
running 1 test
test integration_tests::sandbox_rejects_swapped_cached_untrusted_or_drifted_extractions ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out
$ cargo test --locked --offline -p crunch-android-core -p crunch-android --lib -- --test-threads=1
test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real builder fetched three local fixed-output archives and Bubblewrap executed their extraction derivations. A noncached same-run outcome was admitted with trusted store signatures; swapped outputs, an absent trust key, a cached outcome, and runtime digest drift were each rejected before any SDK stage was generated. Separate adapter tests reject plan categories and archive drift before graph generation, and verify that equivalent plans share an output identity while source and signing changes alter it.

Captured full gate (`/home/brittonr/mantle-cairn-finish/logs/apk-adapter-flake-check-20261010.log`; `nix flake check -L --keep-going --no-write-lock-file --option min-free 0 --option max-free 0` with four local mirrors pinned to the lockfile's source revisions):

```text
checking derivation checks.x86_64-linux.clippy...
checking derivation checks.x86_64-linux.fmt...
mantle-nextest>      Summary [ 439.811s] 6262 tests run: 6262 passed, 150 skipped
all checks passed!
warning: The check omitted these incompatible systems: aarch64-darwin, aarch64-linux, x86_64-darwin
```

Non-claims: the offline test executed **no** SDK or JDK tool stage and produced **no** APK. The separately ignored official proof was not run; this receipt does not establish a working, signed, installable, reproducible, or device-tested APK, and neither Nix nor Cairn gates grant that claim. The existing real-build change remains open for its checked-in independent-store A/B, perturbation, signed-region tamper, and verified-source prerequisites.

## Cairn structural gates before archive

Pinned Cairn `/nix/store/1z6bhr2cl43z4kpsvjkxy9b47nqr9pkg-cairn-0.1.0/bin/cairn`, run from the isolated Mantle worktree after all adapter tasks were checked. Bounded captured results:

```text
validate --root .                            valid=true; issues=[]; receipt_hash=159b000501ea46acd7292f37f37808163bc3ec4f98165d08b746a84599ba3b39
gate proposal add-apk-build-adapter          verdict=PASS; issues=[]; receipt_hash=668560b3e496a753a08ea818bd9ab5452278197aaf1156973fee5ff2786ae3ee
gate design add-apk-build-adapter            verdict=PASS; issues=[]; receipt_hash=55654d9058c8f7487c8d74bc5ec86cb54d74af372910afd3d0b173d9be7d60c9
gate tasks add-apk-build-adapter             verdict=PASS; issues=[]; receipt_hash=38439926f8e35e8ff4e0ad01eb3cddc2cbf99693152eed7c22ded5f1f12901f4
gate proposal prove-apk-adapter-real-build   verdict=PASS; issues=[]; receipt_hash=e701134ba159205e3476bc9511d8d7d3ae5dcf03812115ec5593a7fb54769d43
sync add-apk-build-adapter --root .          dry_run=true; blocked=false; android-adapter merge diagnostics=[]
archive add-apk-build-adapter --root .       dry_run=true; blocked=false; reasons=[]
```

The Cairn verdicts establish lifecycle structure, dependency ordering, and conflict-free spec merge for the presented files; they do not establish real APK execution.

## Post-archive validation

Pinned Cairn `sync add-apk-build-adapter --root . --execute` reported `blocked=false`, `mutated=true`, receipt `51eecd678483a7e576e51aad17a85eabce96b4670c665868f5a00788036bda4c`. After the pre-archive implementation commit, `archive add-apk-build-adapter --root . --execute` reported `blocked=false`, `mutated=true`, receipt `44995cac3cd274f19a589ce65bb2783d33297bc2c62330512759c4fabf2c9182` and moved the change to `1970-01-01-add-apk-build-adapter`.

The captured **post-archive** `cairn validate --root .` JSON output is `evidence/cairn-postarchive-validation-2026-10-10.json` in this archived change: exit 0, `valid=true`, `issues=[]`, receipt `a71c2c814378411b235346ab6a5d2c7bcb2b4f8b1a8c9e5993829de3fe3a2b70`. This is a lifecycle validation, not an APK execution claim. Downstream `cairn readiness prove-apk-adapter-real-build --root . --format json` remained `classification=unavailable`: nine tasks not archive-ready and completion observation unavailable.
