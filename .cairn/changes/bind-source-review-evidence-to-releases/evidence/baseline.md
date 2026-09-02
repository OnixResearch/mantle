# Baseline (V0)

Commands and exact output live beside this file.

- `nix develop -c cargo test -p crunch-release-core`: 235 passed, 0 failed (`baseline-release-core.log`).
- `nix develop -c cargo test -p mantle --test release_cli`: 150 passed, 0 failed (`baseline-release-cli.log`).
- `nix develop -c cargo test -p mantle --bin mantle release_attestation::`: 11 passed, 2544 filtered (`baseline-release-attestation.log`).
- `cairn validate --root .` on the Cairn repository: `"valid": true`, no findings (`baseline-cairn-validate.log`).
- Machine-contract baseline exposed a pre-existing stale `build.build-json-report` producer binding on `origin/main`; this change refreshes it through the generator without touching any other surface semantics (`machine-contract-check.log` shows the final PASS).
