# ast-grep structural evidence

Mantle exposes a pinned ast-grep toolchain and admits bounded structural-evidence sidecars without interpreting rule semantics.

## Pinned toolchain

The flake pins ast-grep `0.42.1` through `packages.<system>.ast-grep-toolchain`. The package copies the executable into a stable profile, exposes both `ast-grep` and `sg`, and writes:

```text
share/mantle/ast-grep-toolchain.json
```

The identity record contains the package/version, relative executable path, upstream Nix store path, digest algorithm, and BLAKE3 digest of the packaged executable. The package build fails if the flake-pinned nixpkgs package version drifts from `0.42.1`.

Use the profile directly or through the default development shell:

```bash
nix run .#ast-grep-toolchain -- --version
nix develop -c ast-grep --version
nix build .#checks.x86_64-linux.ast-grep-package-identity --no-link -L
```

The identity smoke recomputes the executable BLAKE3, compares it with the identity record, checks the schema/package/version/path fields, and runs `ast-grep --version`.

## Sidecar contract

A repository-owned scan or rule-test workflow may write this file into a Mantle build output:

```text
share/mantle/ast-grep-structural-evidence.json
```

Schema `mantle-ast-grep-structural-evidence-v1` records:

- `tool`: package/version plus observed and expected executable BLAKE3 identity;
- `command`: `scan` or `rule-test` plus a BLAKE3 identity for the normalized argv;
- `rule_bundle`: observed and expected repository-owned rule-bundle BLAKE3;
- `scan_scope`: a bounded description plus observed and expected scan-input BLAKE3;
- `output`: scan mode `json-compact`, `json-pretty`, or `json-stream`, or rule-test mode `text-color-never`, plus bounded finding/test counts;
- `receipt.digest_blake3`: the repository-owned execution receipt identity;
- `claim_scope` and `claim_labels`: structural-tool evidence only;
- all required non-claims.

Positive scan and rule-test fixtures plus fail-closed fixtures live in [`tests/fixtures/ast-grep-structural-evidence/`](../tests/fixtures/ast-grep-structural-evidence/).

The owning repository defines deterministic byte encodings for argv, rule bundles, and scan inputs before hashing them. Mantle validates that the sidecar carries lowercase BLAKE3 identities, that observed/expected identities agree, and that the reported tool version is the pinned `0.42.1`. Scan runners select an explicit `--json=compact|pretty|stream` mode; rule-test runners use `--color never`. Mantle does not invent repository rule semantics.

## Functional core and shell

`crunch-release-core` parses, bounds, canonicalizes, and validates already-loaded sidecar DTOs. It has no process, filesystem, environment, clock, or network access. Canonical sidecar identity is BLAKE3 over compact canonical JSON with sorted claim labels and non-claims.

The root-package shell adapter owns filesystem metadata checks, bounded reads, UTF-8 decoding, and the raw sidecar-file BLAKE3. Mantle does not automatically invoke ast-grep. Repository-owned runners invoke `ast-grep scan` or `ast-grep test`, capture raw output, and write the sidecar.

## Build-report evidence

`crunch-build-report-v1` includes:

- `ast_grep_structural_evidence[]` for valid sidecars, including raw-file and canonical-sidecar BLAKE3 identities;
- `ast_grep_structural_evidence_diagnostics[]` for malformed, oversized, stale, mismatched, or overclaimed sidecars.

An absent sidecar produces neither a claim nor a diagnostic. An invalid sidecar is never promoted into the accepted evidence array.

## Release attachment

The existing external-evidence path can carry a validated ast-grep sidecar. Mantle revalidates any external evidence that uses an ast-grep role, schema, or claim scope before release attachment and requires the external-evidence non-claims to preserve the sidecar boundary.

```bash
mantle release create \
  --release-id mantle-<version> \
  --binary <stage2-mantle> \
  --proof-bundle <proof-bundle> \
  --external-evidence <ast-grep-sidecar.json> \
  --external-evidence-role ast-grep-structural-evidence \
  --external-evidence-schema mantle-ast-grep-structural-evidence-v1 \
  --external-evidence-claim-scope structural-tool-evidence-only \
  --external-evidence-non-claim ast-grep.boundary.not-source-behavior \
  --external-evidence-non-claim ast-grep.boundary.not-build-correctness \
  --external-evidence-non-claim ast-grep.boundary.not-cache-correctness \
  --external-evidence-non-claim ast-grep.boundary.not-release-eligibility
```

This attachment proves only the identity and validated shape of structural tool evidence. It does not prove source behavior, build correctness, cache correctness, rule soundness, release eligibility, or release reproducibility. Mantle owns no ast-grep rule catalog and never runs automatic codemods during builds.
