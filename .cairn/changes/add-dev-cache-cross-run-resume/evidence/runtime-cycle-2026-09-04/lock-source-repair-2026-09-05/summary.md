# Native Git planning repair

## Outcome

The repair is validated for the preserved failed source snapshot. The final planning-only replay produced a ready graph with 800 derivations and no planning blockers. It did not compile a unit or restart the bootstrap.

The implementation commits are:

- `5833ede3ffc40798f15b1b908b04aa92330263e4`: source-reference comparison, explicit vendor routing, and early blocked-plan reporting.
- `da2c7fb67398bb3b5b62b2f638b08a53cb948bb4`: captured Git sibling resolution after vendor relocation.

The failed `97f47ae2a644651734324f44991cd4cae847499a` attempt retains its original binary, source profile, provider, cache, and audits. Neither implementation commit changes the production lockfile, dependency pins, or vendor configuration.

## Repairs and owners

Cargo's qualified Git dependency references omit the resolved `#commit`. The native matcher now accepts that reference shape for comparison only. Full resolved sources remain the package identities. Explicit fragments, repository URLs, and selectors remain exact.

The vendor adapter binds each resolved source to its declared directory. It rejects missing mapped payloads, conflicting routes, and ambiguous directory lookups. The two `artifact-auth-core 0.1.0` revisions keep separate vendor paths and content digests.

Cargo's versioned vendor layout also moves relative Git siblings. The new pure resolver uses the parent source identity, package name, and version constraint. Normal and build dependencies cannot fall back to another revision. Invalid paths, invalid versions, missing facts, and ambiguous facts reject.

The CLI shell reports planning blockers before the child-action runtime starts. Blocked child-action requests emit the full planning receipt and exit nonzero. The earlier authority-bound rustc identity check remains intact.

Mantle owns maintenance through `src/rust_plan.rs`, `src/rust_plan/vendor_sources.rs`, `src/rust_plan/git_paths.rs`, and `src/cli_application.rs`. No new external capability or dependency is necessary.

## Full source replay

All replays used the preserved source under:

```text
/home/brittonr/mantle-runs/dev-resume-20260904/.dev-cold-97f47ae2.source-built-fixed-point-staging-3039000/inputs/mantle-source
```

| Planner | Package planning | Derivations |
| --- | --- | --- |
| Original `97f47ae2` | Five missing Git-source facts | 0, blocked |
| Repaired `5833ede3` | One relocated sibling manifest blocker | 0, blocked |
| Repaired `da2c7fb6` | Ready, no blockers | 800, ready |

The first row describes the diagnostic replay. The original failed stage-1 receipt was empty.

The final replay used the existing receipt-bound Rust 1.94.0 compiler for identity capture. It selected `x86_64-unknown-linux-musl`, profile `dev`, and deterministic release paths. It disabled the Cargo oracle and supplied no execution flag. `final-replay.command.txt` contains the exact remote command.

The final replay started at `2026-09-05T16:05:22-04:00` and finished at `2026-09-05T16:09:26-04:00`. It stayed within its 300-second limit. Its CLI exit status was zero, but acceptance also required the receipt checks.

The inspector validated:

- Registry, Git, package, target-unit, host-unit, and derivation planning are ready.
- All planning blocker lists are empty.
- The declared derivation count equals the 800 retained derivations.
- Both Artifact revisions retain distinct paths, revisions, and content digests.
- Captured Git facts, lockfile facts, and the source closure match the preceding replay.
- No topology execution field is present.
- The preceding blocked receipt fails the inspector's readiness checks.

The remote wrapper also verified unchanged Cargo manifests, lockfile, vendor configuration, and both Bounded Tree manifests. The execution, store, state, and forbidden Cargo paths stayed absent.

The inspector reported:

```text
planning replay verification: PASS
```

`verify-replay.scm` runs through the Pi Steel tool with the repaired and preceding JSON paths as arguments. Its exact result is in `verify-replay.log`. The compressed receipts retain the original JSON bytes.

## Content identities

| Artifact | BLAKE3 |
| --- | --- |
| Final release planner, 123575064 bytes | `4b69f987464387d4aceb6156e8acb5826b544522fd139bd44c9fe770b7edecbe` |
| Final uncompressed planning receipt, 10059863 bytes | `1ee07bed95b3c13902f8c67a7c5414d1205803ccc56d5dbf4902e4545331f85d` |

The local receipt digest matched the remote digest after transfer. `final-replay-binary.blake3` and `final-replay-receipt.blake3` retain the full remote paths. `content-blake3.txt` binds the local evidence files.

## Committed-source checks

`committed-relocation-check.log` binds these results to `da2c7fb67398bb3b5b62b2f638b08a53cb948bb4`:

```text
test result: ok. 239 passed; 0 failed; 0 ignored; 0 measured; 2422 filtered out; finished in 3.45s
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.78s
machine schema contract check: PASS (34 contracted, 68 classified)
Rust-plan hexagon architecture verified: findings=0 negative-fixtures=13
```

First-party Clippy and formatting passed in the same command chain. The optimized binary build also passed from that commit. It used an isolated target directory, not the shared release output.

The focused Nix command passed `tigerstyle`, `rust-plan-source-admission`, and `rust-plan-child-action-preflight`. Its test results were:

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 57 filtered out; finished in 0.01s
test result: ok. 239 passed; 0 failed; 0 ignored; 0 measured; 2422 filtered out; finished in 0.78s
```

The committed-source Cairn validation and proposal, design, and tasks gates passed. Tracey reported:

```text
traceability coverage ok: 157/157 referenced (profile mantle-default)
```

## Repeatability and limits

The immediate outcome is a ready native plan for the preserved source. The durable contribution is source-aware dependency resolution and precise rejection before action startup. Positive and negative tests cover these decisions through the native planner, CLI, and Nix checks.

The evidence retains failed regression runs, the debug replay timeout, the failed executable launch, and the later linker-environment failure. The old Pueue records disappeared before wrapper-log export. `pueue-capture-status.txt` states which raw transcripts are unavailable.

This repair does not establish compilation, a fixed point, provider correctness, universal Cargo compatibility, or release eligibility. It does not change historical V48, V98, or Radiance evidence. It does not establish a strict immutable-Octet pass beyond the existing recorded tooling limits.

I3, V2, V3, and V4 remain open. Interruption-time publication, the complete dev cycle, an independent promoted proof, and final change acceptance remain separate work. No bootstrap restart, cache adoption, prefix restoration, promotion, archive, push, or release-alias update occurred.
