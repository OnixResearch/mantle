# Validation: External OCI registry compatibility

Date: 2026-07-17

## Independent fixture discovery

`nix build --no-link --print-out-paths .#oci-distribution-registry` resolved the repository-pinned package to Distribution 3.1.0. Two failed production pushes exposed real interoperability defects hidden by the in-process fixture:

1. Distribution rejected `application/vnd.oci.artifact.manifest.v1+json` before metadata publication.
2. After companion conversion, Distribution rejected the ordinary image manifest because it used `schemaVersion: 1`.

ADR 0030 records the correction: ordinary image/index documents and companion metadata/signature documents now use OCI image-manifest schema version 2. Companions preserve `artifactType`, subject, annotations, and role descriptors through a canonical empty config and payload layers. Mantle report schema versions remain independent.

## Focused implementation evidence

- Task `188` passed the regenerated projection goldens, all focused projection and registry core tests, local OCI CLI tests, and 10 non-ignored registry CLI/support tests with one external test intentionally ignored.
- Task `205` passed machine-contract generation/check (`18 contracted, 47 classified`), 17 example inventory tests, and 11 gallery workflow tests.
- Task `212` passed focused root-package Clippy with `-D warnings`, the complete configured first-party Tiger Style rail, and diff hygiene after the companion-manifest correction.
- Task `215` reran the complete focused projection/registry/local-CLI packet and then executed the external test explicitly. Its final result was `1 passed; 0 failed; 10 filtered out`.

## External compatibility result

`registry_cli_interoperates_with_pinned_distribution_and_rejects_wrong_signature_digest` launched the independent process, used public Mantle commands for gallery import/export, signed registry push, and immutable-digest pull into fresh state, validated ordinary push/pull/import reports, compared every source/pulled layout file byte-for-byte, and then proved a wrong signature-manifest digest produced no layout, state, import report, or pull receipt.

The sanitized durable output is `evidence/external-registry-summary.json`. It records normalized fixture version `github.com/distribution/distribution/v3 v3.1.0+unknown`, `exact_layout_match: true`, `import_state: admitted`, `wrong_signature_manifest_digest_rejected_without_outputs: true`, all three immutable manifest SHA-256 values, policy/key identities, and explicit non-claims. It contains no executable path, temporary path, credential, token, or registry port.

## Repository-wide quality

Task `219` ran `nix develop -c ./scripts/check-first-party-quality.sh` and passed Rustfmt, strict first-party Clippy, and serialized first-party workspace tests. Both root test binaries reported `1552 passed; 0 failed`.

Task `220` ran `nix flake check --no-build -L`; every x86_64-linux package/check/dev-shell derivation evaluated and the command ended with `all checks passed`. The pinned registry package itself had already been realized by `nix build --no-link --print-out-paths .#oci-distribution-registry`.

Task `22` reran the complete first-party Tiger Style rail and diff hygiene after the final source/test edits; both passed.

Lifecycle sync, Tracey, archive, commit, and push remain V3 work.
