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

## Lifecycle pre-archive

- Task `25` passed Cairn validation, proposal/design/tasks gates, and diff hygiene with 5/6 tasks complete and only archive closeout open.
- Task `26` ran sync dry-run/execution; execute receipt `9deec19990270287da31a2177f42bb58dd2e971562fb9f660cc243aa63457493`.
- The accepted `cairn/specs/kernel-bundle-oci/spec.md` was inspected after sync. It contains all three independent-compatibility scenarios, explicitly requires OCI image-manifest schema version 2, and preserves the earlier bounded signature-policy non-claims.
- Evidence-backed implementation/verification links were added to `tools/tracey_refs.rs` only after implementation, focused/full tests, accepted requirement text, and durable external summary existed.
- Tasks `27` and `28` passed Tracey `145/145`, Cairn validation, all three gates, and diff hygiene with only V3 still open.

Task `29` is the final pre-archive packet: diff hygiene passed, validation returned `valid: true`, all three gates returned `PASS`, tasks reported 6/6 complete with zero remaining, and Tracey remained `145/145 referenced`.

## Archive and exact post-archive state

Task `32` ran archive dry-run/execution with `CAIRN_ARCHIVE_DATE=2026-07-17`; execute receipt `a47ae38324cb67fc4fb7cdcf4f34e7addcb17e15d73baa3e87de7d770e8e86d9`.

Task `33` produced exact machine-readable receipts beside this transcript:

- `post-archive-validation.json` reports `"changes": 0`, empty issue/finding lists, and `"valid": true`.
- `post-archive-change-list.json` reports an empty `changes` array.
- `post-archive-tracey.txt` reports `traceability coverage ok: 145/145 referenced (profile mantle-default)`.

Archive commit and push are the remaining mechanical V3 operations.
