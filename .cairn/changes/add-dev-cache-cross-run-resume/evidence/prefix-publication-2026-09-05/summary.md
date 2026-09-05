# Per-stage publication and cold dev launch

## Completion boundary

I3 is implemented in `8e941df20b7d888929692ad5330b76ffcdb7cfa9`. Dev prefix manifests publish before later stages start. They reference shared, remeasured objects. The transition restore test removes the original attempt before it restores into a fresh directory.

The native restore path keeps package roots separate from executable paths. It relocates host evidence only in owned restore copies. Exact replay preserves missing dev action events without turning them into promoted evidence.

The implementation does not complete V2, V3, or V4. A new cold dev run is active. The old `97f47ae2` failure and all historical proof identities remain unchanged.

## Verification

The committed-source run recorded:

```text
test result: ok. 108 passed; 0 failed; 3 ignored; 0 measured; 2573 filtered out; finished in 210.65s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2681 filtered out; finished in 0.00s
machine schema contract check: PASS (34 contracted, 68 classified)
dev-resume architecture verified: findings=0 negative-fixtures=9
```

First-party Clippy passed. The updated tasks gate returned `PASS`. Cairn validation and the proposal and design gates also passed. Tracey reported 157/157 referenced requirements.

All four focused Nix checks passed: `tigerstyle`, `dev-resume-integration`, `dev-resume-stage-publication`, and `dev-resume-architecture`. The Nix test results were:

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2681 filtered out; finished in 0.00s
test result: ok. 108 passed; 0 failed; 3 ignored; 0 measured; 2573 filtered out; finished in 0.59s
```

Compatibility checks also passed 72 Cargo-free tests, 43 Rust-binding tests, and six derivation-action tests. Their exact output is in `compatibility.log`.

The architecture checker retains the core authority fixtures. It also requires the replacement publication sites and exercises missing-marker controls. The initial obsolete-marker failure remains in `nix.attempt1.log`.

## Runtime source and preparation

The source snapshot comes from the exact implementation commit. The release binary has BLAKE3:

```text
db1ee54cfaa4be553525434f7e8d601e18d644a3c30a9d2cb92fec5d4cb1e2f2
```

Its size is 123642568 bytes. The transferred binary matches the local binary byte-for-byte. The source checksum dry run is empty.

The remote filesystem rejected mandatory reflinks. The failed copy only touched the new source directory. A checked ordinary copy replaced its incomplete vendor files. The old source and vendor inputs remain unchanged. `vendor-reflink-failure.log.zst` retains the failed copy output.

The refreshed profile has manifest BLAKE3:

```text
2f9c4a8f18e3e966e3b31b1b63f15193f3a513c396918a8930a66589f1c36197
```

The refresh preserved 64 records and added none. It replaced the Mantle source identity with `943e7f24a8224c296e23848476923b8c50c1e3868b0b3ff2a0525e726e282da8`.

The vendor identity stayed `2a34c833f27a7e2a6a509b466f414969d8b50ecfca8a1555c6845ddfc1298572`. Verification reported `Ready`, with zero missing, stale, unsupported, or untrusted records. This establishes source availability and identity, not proof success.

## Active cold attempt

- Host: `leviathan.cymric-daggertooth.ts.net`.
- Attempt: `dev-cold-8e941df2`.
- Start: `2026-09-05T19:07:04-04:00`.
- Wrapper PID: `1623993`.
- Observed Mantle PID: `1624266`.
- Watcher: local Pueue task `1265`.
- Cache: `/home/brittonr/mantle-runs/dev-resume-20260904/dev-provider-cache-8e941df2`.
- Output: `/home/brittonr/mantle-runs/dev-resume-20260904/dev-cold-8e941df2`.

The preflight required an absent cache and output. It recorded 409250521088 free bytes, a 200000000000-byte proof bound, and 16 jobs. The run uses strict hermeticity, disables substitution, and enables dev resume.

`cold-run.command.txt` contains the exact command. The remote attempt owns its live log and terminal status. `cold-start-observation.txt` records an active process after startup. No terminal result was available at that observation.

The watcher reports only that terminal status appeared. Its own exit status does not establish cold-run success. The cold status file and final reports remain authoritative.

## Remaining work and blockers

V2 still requires the completed cold, cached, and adopt cycle and the required fresh-directory prefix checks. V3 requires a separate fresh promoted cold proof. Neither requirement is complete.

Whole-flake evaluation still fails while evaluating the unchanged Nickel cohort:

```text
error: path '/nix/store/666qf3k9djcfzv6lwcp4l4v7fgck77ng-source.drv' is not valid
```

A refreshed evaluation reproduced the failure. The remote store could not supply that exact derivation. No source pin or lockfile changed. V4 remains open with this blocker and the remaining runtime acceptance work.

No specification sync, archive, push, or release-alias update occurred. The active worktree must remain available while the runtime work continues.
