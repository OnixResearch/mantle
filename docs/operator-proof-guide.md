# Mantle operator proof guide

Use this guide to run, inspect, and report Mantle proof evidence without
promoting a narrow proof into a broader release, compiler, or deployment claim.
Treat the machine receipts as the source of truth. Human summaries and status
updates must quote the exact evidence class, paths, verdicts, and blockers that
were observed in the current run.

## Workflow chooser

| Workflow | Primary command | Main output | Claim when current and successful |
|---|---|---|---|
| Self-build fixed point | `./scripts/prove-self-hosting.sh` | `target/self-hosting-proof/run-.../` | The checked-out Mantle can rebuild a byte-identical stage2 Mantle from the staged source under the selected proof mode. |
| Non-Nix-host self-build | `./scripts/prove-self-hosting.sh --non-nix-host` | `target/self-hosting-proof/run-.../` | The fixed-point proof also ran with Nix commands scrubbed from the proof-runner `PATH`. |
| Host-tool-free stage0 boundary | `./scripts/prove-self-hosting.sh --no-host-tools --stage0-inventory <file>` | `target/self-hosting-proof/run-.../protected-exec-audit.json` | The protected stage0 boundary used declared seed executables under protected exec supervision. |
| Genuine release rebuild | `./scripts/prove-real-release-determinism.sh --toolchain-archive <archive>` | `target/release-evidence/<release-id>-proof/` plus summary/verify JSON | The named packaged artifact rebuilt twice from the exact v2 descriptor/authority identities under fresh isolated roots and matched. |
| Cargo-free fixed point | `mantle self-build --cargo-free --fixed-point --strict-hermetic --out /tmp/mantle-cargo-free` | `/tmp/mantle-cargo-free/` | Mantle built the requested Mantle binary through the bounded native Rust topology and stage1/stage2 binary digests matched under strict proof admission. |
| Nix-free demo bundle | `mantle --json nix-free-demo validate <summary.json>` | generated demo README plus machine summary | The recorded source-root Cargo-free fixed-point demo profile is claimable only when all validator diagnostics are absent. |
| Foreign import receipt review | [`docs/foreign-derivation-import-trust-model.md`](foreign-derivation-import-trust-model.md) | import receipt, graph, index, and policy files | Admission evidence is claim-safe only when reported with receipt non-claims and without output-trust or build-success wording. |
| Kani release evidence | [`docs/kani-release-evidence.md`](kani-release-evidence.md) | bundled `kani-model-check-receipt` plus Kani toolchain metadata | Mantle links Kani/Rust/CBMC/solver/wrapper/closure identity to a bundled receipt while Valence owns Kani semantics and non-claims. |
| Offline build runbook | [`docs/operator-workflows.md`](operator-workflows.md#offline-build-runbook) | source-bundle preflight report and build JSON report | The runbook can show source/input availability, network policy, route, and offline Cargo sidecar evidence; it is not build success or reproducibility proof by itself. |

Run the fast prerequisite check before starting expensive self-build proof work:

```bash
./scripts/prove-self-hosting.sh --check
```

Use the checked-in guide drift checks after documentation, trust-model, or
proof-surface edits:

```bash
nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs
nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs
nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test
```

## Foreign import receipt trust model

Foreign derivation import receipts are admission records, not proof success
records. The trust model guide explains graph provenance, policy digests, source
verification, cache/substitution trust, sandbox capabilities, realization,
output verification, Guix-like examples, Nix-like examples, and exact receipt
non-claims:

- [`docs/foreign-derivation-import-trust-model.md`](foreign-derivation-import-trust-model.md)

Use that guide before claiming anything stronger than import admission. A receipt
alone does not claim build success, package correctness, bootstrap parity, output
trust, reproducibility, or foreign-frontend availability; later Mantle
realization and verification evidence must carry those claims.

## Offline build runbook evidence

Follow the command sequence in
[`docs/operator-workflows.md#offline-build-runbook`](operator-workflows.md#offline-build-runbook):

```bash
mantle source bundle export --build-root ./package.ncl --import-path lib --to source-bundle.json
mantle source bundle import --from source-bundle.json --pin
mantle source bundle verify --from source-bundle.json --imported
mantle source bundle preflight --build-root ./package.ncl --import-path lib
mantle build --offline-source-preflight --no-substitute ./package.ncl
```

For proof-before-claim reporting, cite the source preflight `ready_class`,
`source_state_blake3`, and `next_actions[]` plus build JSON
`network_policy_reports[]`, `cargo_build_evidence[]`, and
`cargo_build_evidence_diagnostics[]`.
The source bundle evidence proves declared source/input availability and identity only.
The source-bundle route execution is future work.
If a report has next actions or diagnostics, describe the blocker and the next
command instead of claiming success.

## Self-build proof

### Prerequisites

The ordinary self-build proof needs these prerequisite categories:

- Linux with user namespaces and bubblewrap available.
- Repo nightly Rust toolchain, `cargo`, `clang`, `mold`, `pkg-config`, and
  OpenSSL development metadata.
- `git` for the checked-in proof helper and current tracked-source inventory.
- A static sandbox shell, normally a `busybox-static` binary, selected through
  `SNIX_BUILD_SANDBOX_SHELL` or discovered by the helper.
- The checked-in source tree, checkout-local `vendor-deps/`, and `.cargo/vendor-config.toml`.
- Around 4 GiB of writable scratch space under
  `target/self-hosting-proof/work/` or `CRUNCH_PROOF_SCRATCH_DIR`.

### Commands

```bash
./scripts/prove-self-hosting.sh --check
./scripts/prove-self-hosting.sh
./scripts/prove-self-hosting.sh --non-nix-host
./scripts/prove-self-hosting.sh --no-host-tools --stage0-inventory <file>
```

Use `--bundle-dir <dir>` when the proof bundle must be written outside the
repo-local default. Relative bundle paths are anchored to the repo root.

### Evidence to inspect

Successful runs write `target/self-hosting-proof/run-.../` and refresh
`target/self-hosting-proof/latest`. Inspect these files before reporting a
result:

- `manifest.json` — schema, proof mode, staged source, selected provider kind,
  stage1/stage2 binary digests, bootstrap-tool digests, and prerequisite digest
  facts.
- `summary.txt` — human summary of fixed-point status, selected paths, and
  protected execution status when present.
- `binaries/stage1-mantle` and `binaries/stage2-mantle` — durable proof
  binaries used by release packaging and witness rebuilds.
- `stage0/stdout.txt`, `stage0/stderr.txt`, and `stage0/diagnostics.txt` — saved
  stage0 command output and diagnostics.
- `stage2/stdout.txt`, `stage2/stderr.txt`, and `stage2/diagnostics.txt` — saved
  stage2 command output and diagnostics.
- `stage0-prerequisites/inventory.md` — copied prerequisite inventory.
- `protected-exec-audit.json` — no-host-tools protected execution audit when the
  protected mode is used, including declared seed roles and bounded version
  evidence digests.

### Reporting outcomes

- **Success:** report the proof mode, bundle path, `manifest.json` digest when
  available, stage1/stage2 digest equality, and whether the non-Nix-host or
  no-host-tools boundary was used. For no-host-tools runs, include the accepted
  stage0 inventory digest from the proof lines or protected-exec audit.
- **Blocked:** report the exact failing command, bundle or diagnostics path,
  blocker text, and next action. Blocked evidence is not proof success.
- **Failed:** report the command, exit status, saved stdout/stderr paths, and
  the first deterministic failure message. Do not summarize a failed run as a
  proof.
- **Stale evidence:** if source, proof helper, vendor inputs, proof policy, or
  relevant docs changed after the bundle was produced, mark the bundle stale and
  rerun or narrow the claim to the old tree.

Proof-mode admission is closed by default for both selected hermeticity mode and
hermeticity audit events. Clean strict evidence records the selected
`hermeticity_mode`, strict proof eligibility verdict, event-set digest, and
policy basis; practical or impure mode, missing closure facts, protected-env
leaks, undeclared host-tool facts, and unapproved event classes appear as
deterministic blockers. Approved exceptions must name the event class, affected
proof class, and narrower claim instead of satisfying the stricter proof class.

The self-build proof does not prove compiler correctness, does not prove full
Cargo compatibility, does not prove release reproducibility, does not prove
deploy success, and does not prove general Nix replacement completeness.

## Genuine release rebuild proof

Use the reviewed production recipe with an explicit content-bound toolchain
closure:

```bash
./scripts/prove-real-release-determinism.sh \
  --proof-bundle target/self-hosting-proof/run-... \
  --toolchain-archive /path/to/content-bound-toolchain.tar
```

Before either proof run, Mantle hashes a canonical v2 descriptor over the exact
source archive, recipe, executable, ordered arguments, tool/toolchain inputs,
provider, policies, selected target identities, and fresh roots. Its pure
authority planner rejects target bytes and same-content copies, hardlink object
aliases, symlinks, the whole release bundle, prior proof outputs, the ordinary
reproduction output, undeclared reads, reused roots, and read/write overlap. The
sandbox receives only separately materialized approved regular files; the
release bundle is not mounted.

Inspect `deterministic-build-proof.json` for
`mantle-deterministic-proof-receipt-v2`, both descriptor/authority-plan BLAKE3
fields, `target_authority_excluded = true`, empty plan blockers, and each run's
matching citations, approved read identities, and empty authority violations.
Then inspect the release verify receipt and generated summary. Version 1 remains
parseable for diagnosis but is always `missing-genuine-rebuild-evidence` and
cannot satisfy release verification, the standalone checker, Nix witness
admission, summaries, or bootstrap-parity evidence.

For bootstrap parity, genuine v2 release evidence is partial evidence only. A
missing, legacy, or incomplete genuine-rebuild fact leaves `crunch.self-build`
blocked; even accepted release evidence does not complete Guix or StageX parity.
Report the exact descriptor/plan digests and fresh roots. The claim does not
establish compiler/verifier soundness or full-bootstrap reproducibility.

## Cargo-free fixed-point proof

The Cargo-free fixed-point lane is the bounded native Rust topology path. Keep
`--out` outside the source root so proof artifacts do not perturb source digests.

```bash
mantle self-build --cargo-free --fixed-point --strict-hermetic --out /tmp/mantle-cargo-free
```

When using the standalone script rail directly, keep the same evidence boundary
and run its preflight before the expensive path:

```bash
nix develop -c cargo -Zscript scripts/prove-cargo-free-fixed-point.rs --check --root .
```

Inspect these output files in the selected output directory:

- `meta.json` — `mantle-cargo-free-fixed-point-proof-v1` schema, status,
  `fixed_point`, `hermeticity_mode`, strict `proof_eligibility`, stage
  summaries, source-built toolchain closure status, blocker, and `non_claims`.
- `preflight.json` — selected root, toolchain and policy facts before stage
  execution.
- `non-claims.txt` — human-readable claim exclusions.
- `stage1/receipt.json` and `stage2/receipt.json` — native topology execution
  receipts.
- `stage1/stderr.txt`, `stage2/stderr.txt`, `stage1/smoke-stdout.txt`, and
  `stage2/smoke-stdout.txt` — execution diagnostics and smoke output.

A successful proof-admissible Cargo-free fixed-point report requires
`status = "success"`, `fixed_point = true`, `hermeticity_mode = "strict"`,
`proof_eligibility.admitted = true`, matching stage binary BLAKE3 digests, no
blocker, and bounded non-claims. A practical-mode or blocked report with
`status = "blocked"` is still useful frontier evidence, but it is diagnostic
only and is not a Nix-free or release proof success claim.

### Current refreshed evidence (2026-07-03)

The current inspected Cargo-free fixed-point evidence succeeds for the bounded
Cargo-free proof mode. Use this as the current status until a newer same-tree
bundle supersedes it:

- Command shape: `mantle --json self-build --cargo-free --fixed-point --out "$PROOF_OUTPUT_DIR" --rustc "$(command -v rustc)" --target x86_64-unknown-linux-gnu`
- Output bundle: `/tmp/mantle-cargo-free-fixed-point-vendor-repair-20260703T204500Z`
- Verdict: `status = "success"`, `fixed_point = true`; stage1 and stage2 both reported `execution_status = "success"`.
- Stage binary BLAKE3: stage1 and stage2 both produced `ca00cd5866b0128434f95a0e0cf63ff2e5cc947eb90b60206cb9078bfe44215d`.
- Per-stage topology units: 687 executed, 0 failed.
- Rust-plan receipt hash: `f35cc306c76d14a44599ce438f420ea9bcfbc6a843d9a9a64f5edb5915e2ef6d` for both stages.
- Native registry source digest: `964dc130610257aadedbc27a24284a58560fb01086c6686f2f5839af753a0ffd`; native registry source planning was ready with zero blockers.
- Cargo guard: `cargo_marker_absent = true` for both stages.
- Non-claims: `not-crunch-bootstrap`, `not-release-reproducibility`, `not-source-built-toolchain-closure`, and `not-full-cargo-compatibility`.

This evidence predates strict proof-mode admission metadata and is a bounded Cargo-free fixed-point diagnostic success claim only. It is not strict proof admission, Mantle bootstrap, release reproducibility, source-built toolchain closure, or full Cargo compatibility evidence.

This lane does not prove compiler correctness, does not prove full Cargo
compatibility, does not prove release reproducibility, does not prove deploy
success, and does not prove general Nix replacement completeness.

## Nix-free demo bundle validation

Nix-free demo wording is claimable only from the demo-profile validator. The
machine summary schema is `mantle-nix-free-demo-summary-v1`, and the current
profile is `source-root-cargo-free-fixed-point`. Use `mantle --json
nix-free-demo validate <summary.json>` for a stable machine decision,
`mantle nix-free-demo readme <summary.json>` to regenerate the derived demo
README, and `mantle --json nix-free-demo generate --out <dir> ...` to assemble a
self-contained demo bundle from explicit existing evidence inputs. The generator
packages evidence only; it does not run hidden proofs. Do not invent a wider CLI
success claim.

The machine summary must include these fields:

- `schema`
- `profile`
- `fixed_point_verdict`
- `stage1_binary_blake3`
- `stage2_binary_blake3`
- `source_root_identity`
- `toolchain_policy_digest_blake3`
- `command_owned_wrappers`
- `guards`
- `replay_hints`
- `non_claims`

The `guards` list must contain denial evidence for `cargo`, `nix`, `rustup`,
and `ambient-wrapper`. The validator rejects missing or mismatched evidence with
stable diagnostics such as `missing-fixed-point-evidence`,
`missing-guard-evidence`, and `missing-non-claims`.

A generated demo README may say `Nix-free fixed-point demo: claimable` only when
validation succeeds. When validation does not succeed, the generated outcome is
`Demo claim: not claimable`; report the narrower fixed-point artifact or blocker
and the diagnostic codes instead. For blocked or synthetic evidence, pass
explicit `--non-claim` values and expect the generated `validation.json` to keep
the bundle non-claimable rather than turning packaging into proof success.

Minimal successful generator shape:

```bash
mantle --json nix-free-demo generate \
  --out /tmp/mantle-demo-bundle \
  --proof-status success \
  --source-root-identity source-root-v1 \
  --toolchain-policy-digest-blake3 <blake3> \
  --stage1-binary-blake3 <blake3> \
  --stage2-binary-blake3 <same-blake3> \
  --transcript proof.log \
  --receipt-digest receipt:<blake3> \
  --artifact-digest stage2-mantle:<blake3> \
  --guard cargo:denied:"cargo denied by fixture" \
  --guard nix:denied:"nix denied by fixture" \
  --guard rustup:denied:"rustup denied by fixture" \
  --guard ambient-wrapper:denied:"ambient wrapper denied by fixture" \
  --non-claim "not release reproducibility"
```

Blocked-proof example:

```bash
mantle --json nix-free-demo generate \
  --out /tmp/mantle-demo-blocked \
  --proof-status blocked \
  --source-root-identity source-root-v1 \
  --toolchain-policy-digest-blake3 <blake3> \
  --transcript proof.log \
  --receipt-digest receipt:<blake3> \
  --artifact-digest stage1-receipt:<blake3> \
  --guard cargo:denied:"cargo denied by fixture" \
  --guard nix:denied:"nix denied by fixture" \
  --guard rustup:denied:"rustup denied by fixture" \
  --guard ambient-wrapper:denied:"ambient wrapper denied by fixture" \
  --non-claim "blocked before producing a stage binary" \
  --non-claim "not Nix-free fixed-point success"
```

Use this exact non-claim vocabulary when summarizing any proof family in this
guide:

- does not prove compiler correctness
- does not prove full Cargo compatibility
- does not prove release reproducibility
- does not prove deploy success
- does not prove general Nix replacement completeness

The Nix-free demo profile stays within those same non-claims.

## Cleanup

- Keep proof bundles that support a current claim until their digest, path, and
  command transcript have been recorded in the relevant evidence file.
- Heavy scratch directories under `target/self-hosting-proof/work/` or external
  `/tmp/mantle-*` proof roots may be removed after their durable bundle has been
  copied and verified.
- Do not delete bundle-local `manifest.json`, `meta.json`, receipt files,
  `summary.txt`, `non-claims.txt`, or generated demo README files while they are
  referenced by evidence.
- If `/tmp` fills during source-built or Cargo-free proof work, move `--out`,
  `CRUNCH_PROOF_SCRATCH_DIR`, or proof bundle roots to a filesystem with enough
  free space before rerunning.

## Drift check

Run the guide guard after proof documentation, command, or receipt-field edits:

```bash
nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs
nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs --self-test
nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs
nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test
```

The proof guide guard checks that this guide still names the current command
snippets, important bundle paths, required demo summary fields, outcome
vocabulary, and bounded non-claims. Its self-test covers positive validation
plus negative stale command, missing bundle field, and overbroad proof-claim
fixtures. The foreign import trust-model guard checks the linked guide's trust
boundaries, non-claims, Guix-like and Nix-like examples, README linkage, and this
proof-guide linkage.
