# Baseline and search contract

Recorded: 2026-07-16

## Exact goal

Provide a production Mantle CLI path that publishes a previously admitted local OCI layout to an explicit OCI Distribution-compatible registry and pulls the exact image plus subject-bound Mantle metadata back into fresh state through ordinary OCI import admission. Add the gallery workflow only after positive and adversarial production rails pass.

## Observable completion evidence

- `mantle artifact oci-push` uploads/reuses exact content, publishes subject-bound metadata, publishes the image tag last, verifies immutable manifests, and emits a contracted receipt.
- `mantle artifact oci-pull` requires expected image and metadata-manifest SHA-256 values, reconstructs byte-identical local layout/report material, atomically publishes it, imports into fresh Mantle state as `admitted`, and emits a contracted receipt linked to the import receipt.
- Deterministic public CLI tests use an in-process authenticated registry and cover tag drift, tampered metadata/blob bytes, denied credentials, interrupted publication, and content-addressed rerun.
- Machine-artifact schemas/contracts/fixtures, gallery metadata/docs, first-party quality, Tiger Style, dependency audit, Tracey, and Cairn lifecycle checks pass or record exact pre-existing blockers.

## False completion and excluded outputs

The following are not completion:

- wrapping Docker/Podman/ORAS/Skopeo;
- uploading only a local-layout tarball;
- publishing an image tag without recoverable subject-bound Mantle metadata;
- pulling by mutable tag without an expected manifest digest;
- reconstructing as `compatibility-only` when the exact admitted export report should be present;
- test-only HTTP logic not reached by the public CLI;
- a success report written before immutable post-publication verification;
- credentials, credential paths, local paths, or response bodies appearing in receipts/errors;
- claims of registry trust, authorization, signing, exactly-once publication, upload resumption, arbitrary registry compatibility, kernel compatibility, bootability, deployability, or release eligibility.

## Audit risks

- tag mutation between operator handoffs;
- subject/report substitution that regains false admission;
- SHA-256/BLAKE3 role confusion;
- redirect or ambient-proxy credential leakage;
- partial publication being called transactional success;
- descriptor count/size overflow or unbounded response reads;
- local output/report/CAS mutation before complete validation;
- machine-contract shape drift or secret-bearing fields;
- fixtures that bypass the production CLI.

## Declared budget and approach registry

- Five mechanism families, one surviving candidate, at most two implementation redirections after adversarial failures.
- Repository authority limited to accepted/archived OCI specs, production OCI core/shell/CLI, machine-contract tooling, gallery/tests, and the pinned ureq source.
- Deterministic local loopback only; no external registry/network authority is required for validation.
- Terminal outcomes: validated and archived, exact blocker, exhausted bounded route, or user decision required for broader authentication/referrers scope.

| Family | Mechanism | State | Gap/blocker |
|---|---|---|---|
| External CLI | shell out to registry client | rejected | authority and receipts become hidden |
| Layout archive | one tar/blob artifact | rejected | weaker than ordinary image publication |
| Image mutation | add Mantle metadata to image | rejected | changes proven layout identity |
| Referrers discovery | discover subject artifacts | blocked | compatibility support is broader than bounded baseline |
| Companion tag | explicit subject-bound metadata plus image tag last | active | requires production HTTP/CLI and adversarial proof |

## Pre-change baseline

Pueue task `56` ran, in order:

```text
nix develop -c cargo test -p mantle --test kernel_bundle_oci_cli -- --test-threads=1
nix develop -c cargo test -p mantle --bin mantle 'oci_projection::tests::' -- --test-threads=1
nix develop -c cargo test -p mantle --bin mantle 'oci_projection_shell::tests::' -- --test-threads=1
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
```

The CLI, pure projection, and shell suites all passed because the chained command reached the final checker. The captured shell result was:

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1527 filtered out
```

The machine-contract checker then failed on three pre-existing root JSON producer-family coverage gaps unrelated to OCI:

```text
src/external_batch_dispatch.rs
src/remote_build/tests/external_batch_hardware_tests.rs
src/remote_farm_config.rs
```

This baseline failure is not waived. The change will rerun the checker after adding registry receipt contracts and will report whether those pre-existing inventory gaps remain or are repaired independently.
