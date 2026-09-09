# Prefix implementation checkpoint

## Implemented behavior

Dev-only prefix manifests bind exact stage payload sets and share immutable content objects. Provider publication occurs at each completed boundary. Mantle stage 1 publishes before stage 2 starts. A failed publication stops continuation.

Restore follows the selected manifest's checkpoint identity. It does not require later-stage payloads. A transition fixture restored into a fresh directory after removal of the earlier attempt. Missing objects and symlink namespaces reject.

Native-prefix restoration relocates host evidence and distinguishes executable paths from package roots. Relocation requires an owned restore copy. Cached native observations retain their exact incomplete status. Deterministic replay rejects altered counts, while promoted admission still requires complete reconciliation.

## Current evidence

The baseline recorded:

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 88 passed; 0 failed; 3 ignored; 0 measured; 2570 filtered out; finished in 1.22s
```

The new prefix regression first failed on the old all-provider requirement:

```text
test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 2661 filtered out; finished in 0.01s
```

The latest focused runs recorded:

```text
test result: ok. 108 passed; 0 failed; 3 ignored; 0 measured; 2573 filtered out; finished in 82.93s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2681 filtered out; finished in 0.00s
machine schema contract check: PASS (34 contracted, 68 classified)
dev-resume architecture verified: findings=0 negative-fixtures=9
```

First-party Clippy and formatting passed in the same command chain. Additional compatibility tests recorded:

```text
test result: ok. 72 passed; 0 failed; 0 ignored; 0 measured; 2612 filtered out; finished in 0.33s
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 2641 filtered out; finished in 0.08s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2678 filtered out; finished in 0.02s
```

Machine generation changed only the `bootstrap.dev-resume-report` producer identity. Its schema, fixture, and consumer-policy identities stayed unchanged.

## Preserved failures and pending checks

The first integration compile rejected re-exports with insufficient visibility. The first Nix check also rejected the obsolete `publish_dev_resume_bundles` architecture marker. The checker now requires the replacement publication sites and exercises positive and missing-marker controls.

Focused Nix attempt 2 remains active at this checkpoint. Its Tiger Style and architecture legs completed, but the combined command still requires final inspection.

Whole-flake evaluation remains blocked by this unchanged upstream derivation path:

```text
error: path '/nix/store/666qf3k9djcfzv6lwcp4l4v7fgck77ng-source.drv' is not valid
```

A refreshed evaluation reproduced the error. A remote copy could not supply that exact derivation. No lockfile or source pin changed for this blocker.

Runtime confirmation, the independent promoted proof, and final change acceptance remain open. This checkpoint does not mark I3, V2, V3, or V4 complete. The failed `97f47ae2` attempt and historical proofs remain unchanged. No new bootstrap started at this checkpoint.
