# Remote-admission baseline

Date: 2026-09-04

The first baseline attempt stopped because `/tmp` returned `Disk quota exceeded (os error 122)`.

The failed logs remain in `../baseline-attempt1-quota-2026-09-04/`. No source changed before the corrected baseline.

After disposable target removal, pueue task 218 completed these commands:

- `cargo test -p crunch-build --lib remote_attempt --offline`: 28 passed, 0 failed.
- `cargo test -p crunch-release-core --lib trellis_proof --offline`: 7 passed, 0 failed.
- `cargo test -p mantle --bin mantle remote_attempt --offline`: 14 passed, 0 failed.
- `cargo -Zscript --offline scripts/check-machine-schema-contracts.rs`: PASS, 32 contracted and 66 classified.
- `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`: status 0 with no findings.

Pueue task 241 completed all three change gates with status 0:

- `gate proposal verify-remote-admission-with-trellis`.
- `gate design verify-remote-admission-with-trellis`.
- `gate tasks verify-remote-admission-with-trellis`.

The command logs and status files in this directory are the baseline evidence.
