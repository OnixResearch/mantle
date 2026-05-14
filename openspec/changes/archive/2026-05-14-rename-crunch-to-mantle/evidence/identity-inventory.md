# Mantle rename identity inventory

This inventory classifies tracked `Crunch`/`crunch` identity occurrences before the implementation rename. It is evidence for Phase 1 of `rename-crunch-to-mantle` and the input to the later stale-branding check.

## Canonical spelling

- Canonical product name: `Mantle`.
- Canonical command/package spelling: `mantle`.
- Typo check: tracked search found no `mantel` spelling outside the active OpenSpec task that names the typo to forbid.
- Rename rule: current user-facing surfaces move to `Mantle`/`mantle`; historical/archive evidence may retain `Crunch`/`crunch` only when classified as archival or compatibility context.

## Scan method

Tracked files were scanned with `git ls-files` for:

- `\b[Cc]runch\b`
- `/crunch/store`
- `crunch.lock`, `crunch-project.ncl`, `crunch.toml`, `crunch.ncl`
- `.crunch`

Current baseline: 488 tracked files, 5,436 matched tokens. This is intentionally broad and includes Rust crate names, archived OpenSpecs, generated evidence, and internal Nickel variable names; later checks should narrow to unclassified current-facing old branding.

## Bucket summary

| Bucket | Files | Matches | Classification |
| --- | ---: | ---: | --- |
| Rust/package implementation surface | 94 | 779 | Current surface for root package/bin/help/defaults; crate names may be staged internal compatibility unless externally exposed. |
| Current docs/examples/ADRs | 37 | 811 | Current user-facing docs must be migrated or explicitly moved to migration/history sections. ADRs may keep historical decision prose but need current-reader framing. |
| Current OpenSpec specs | 48 | 1,384 | Current specs must be updated to Mantle except compatibility requirements and legacy scenarios. |
| Bootstrap/Nickel library surface | 104 | 418 | Most `let crunch = import "lib.ncl"` style locals are internal implementation names; file/default package surfaces remain current-facing. |
| Tests/fixtures | 25 | 677 | Split between current assertions to rename and deliberate legacy compatibility tests to keep. |
| Evidence/packages/fixtures | 20 | 275 | Release/proof/parity/generated artifact identity inventory; new artifacts should use Mantle, historical evidence remains readable. |
| Historical/archive | 138 | 955 | Allow by default under `openspec/changes/archive/**` and `.autoresearch/**` unless surfaced as current docs. |
| Active rename change | 4 | 51 | Intentional OpenSpec wording for this change; allow. |
| Other tracked scripts/config | 18 | 85 | Review case-by-case; quality gates and scripts that present commands are current-facing. |

## Current user-facing surfaces to migrate

These are current-reader or operator surfaces, not archival evidence:

- `README.md` — main entry point, build/install/self-build/release command examples.
- `docs/operator-workflows.md`, `docs/benchmark-suite.md`, `docs/bootstrap-stage0-inventory.md` — operator docs and command examples.
- `Cargo.toml` root `[package] name = "crunch"`, `[[bin]] name = "crunch"`, workspace package names, Tiger Style default package selectors.
- `src/main.rs` CLI identity: `#[command(name = "crunch")]`, help/about text, default `/crunch/store`, `$CRUNCH_STATE_DIR`, `crunch.ncl` default selectors.
- `src/project_cmd.rs`, `src/project_build.rs`, `src/build_cmd.rs`, `src/bootstrap.rs`, `src/self_build.rs`, `src/shell_cmd.rs`, `src/store_cmd.rs`, `src/attest_cmd.rs`, `src/build_log.rs`, `src/build_report.rs` — command text, default state/config paths, project file handling, and operator diagnostics.
- `lib/project.ncl`, `builders/mk_derivation.ncl`, `bootstrap/crunch.ncl`, and default package-selector behavior around `crunch.ncl`.
- `scripts/check-no-std-core.sh`, `scripts/quality-gate-common.sh`, `scripts/prove-self-hosting.sh`, `scripts/check-bootstrap-source-pins.rs`, `scripts/no_std_core_checks.py`, `flake.nix`, `dylint.toml` — developer/operator command names or package selectors.
- Current specs under `openspec/specs/**` — current requirements must say Mantle unless the scenario is explicitly about legacy Crunch compatibility.
- `AGENTS.md` — loaded operator guidance; migrate current product guidance after implementation decisions are locked, while preserving clearly historical notes if needed.

## Compatibility surfaces to preserve deliberately

These old names must not disappear blindly; they need explicit behavior and tests:

- Legacy command entry point `crunch` if retained.
- `crunch-project.ncl`, `crunch.lock`, `.crunch/` project/generated inputs.
- Default package file `crunch.ncl` and selectors like `.#name` against that file until migration behavior is defined.
- Logical store prefix `/crunch/store` for old artifacts and compatibility-mode tests.
- Environment/config names such as `CRUNCH_STATE_DIR` and state paths like `~/.local/state/crunch` if a transition window is chosen.
- Existing release/proof/parity evidence whose schema or manifest names begin with `crunch-*`.

## Generated, release, proof, and parity artifact inventory

Current generated/release/proof surfaces carrying product identity:

- `src/release_evidence.rs` and `crates/crunch-release-core/**` — release evidence manifest schemas, workflow command fields, proof bundle paths.
- `src/release_attestation.rs`, `src/release_reproducibility.rs`, `src/witness_rebuild.rs`, `src/witness_handoff.rs` — release/witness proof command surfaces.
- `crates/crunch-attestation-core/src/release.rs` — `crunch-release-attestation-v1` and `crunch-witness-attestation-v1` schema constants.
- `src/bootstrap_parity.rs` and `bootstrap/evidence/*.json` — parity report schema/receipt names such as `crunch-bootstrap-*-v1`.
- `src/self_build.rs`, `tests/self_hosting.rs`, `tests/release_cli.rs`, `tests/attest_cli.rs` — self-build and release proof fixtures/assertions.
- `packages/clankers/*.json`, `packages/clankers/*.ncl`, and `packages/clankers/*proof*` — package proof metadata containing Crunch command/proof identity.
- `Cargo.lock` and generated package metadata — will change when package/binary metadata is renamed; treat as generated but tracked.

Migration rule for this bucket: new post-rename artifacts should identify `Mantle`/`mantle` without increasing proof claims. Pre-rename evidence can remain valid as historical Crunch-era evidence if validation code recognizes it explicitly; otherwise it should fail with a clear unsupported-version diagnostic.

## Historical/archive allowlist seeds

The stale-branding check may allow these paths by default:

- `openspec/changes/archive/**`
- `.autoresearch/**`
- old release/proof fixture files that are explicitly named as historical compatibility fixtures
- ADR decision records when the occurrence is historical prose rather than current instruction

## Next implementation inputs

1. Rename the root package/binary/help/defaults first, while deciding whether `crunch` remains a wrapper/alias.
2. Add compatibility tests for legacy command/project/store names before broad doc edits.
3. Update current docs/specs and release/proof schemas after CLI/default behavior is executable.
4. Convert this inventory into the stale-branding check allowlist: archive paths, compatibility tests/docs, historical evidence fixtures, and external names are allowed; current README/CLI/release manifest old branding fails.
