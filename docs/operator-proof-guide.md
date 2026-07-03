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
| Cargo-free fixed point | `mantle self-build --cargo-free --out /tmp/mantle-cargo-free` | `/tmp/mantle-cargo-free/` | Mantle built the requested Mantle binary through the bounded native Rust topology and stage1/stage2 binary digests matched. |
| Nix-free demo bundle | `mantle --json nix-free-demo validate <summary.json>` | generated demo README plus machine summary | The recorded source-root Cargo-free fixed-point demo profile is claimable only when all validator diagnostics are absent. |

Run the fast prerequisite check before starting expensive self-build proof work:

```bash
./scripts/prove-self-hosting.sh --check
```

Use the checked-in guide drift check after documentation or proof-surface edits:

```bash
nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs
```

## Self-build proof

### Prerequisites

The ordinary self-build proof needs these prerequisite categories:

- Linux with user namespaces and bubblewrap available.
- Repo nightly Rust toolchain, `cargo`, `clang`, `mold`, `pkg-config`, and
  OpenSSL development metadata.
- `git` for the checked-in proof helper and current tracked-source inventory.
- A static sandbox shell, normally a `busybox-static` binary, selected through
  `SNIX_BUILD_SANDBOX_SHELL` or discovered by the helper.
- The checked-in source tree, `vendor-deps/`, and `.cargo/vendor-config.toml`.
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
  protected mode is used.

### Reporting outcomes

- **Success:** report the proof mode, bundle path, `manifest.json` digest when
  available, stage1/stage2 digest equality, and whether the non-Nix-host or
  no-host-tools boundary was used.
- **Blocked:** report the exact failing command, bundle or diagnostics path,
  blocker text, and next action. Blocked evidence is not proof success.
- **Failed:** report the command, exit status, saved stdout/stderr paths, and
  the first deterministic failure message. Do not summarize a failed run as a
  proof.
- **Stale evidence:** if source, proof helper, vendor inputs, proof policy, or
  relevant docs changed after the bundle was produced, mark the bundle stale and
  rerun or narrow the claim to the old tree.

The self-build proof does not prove compiler correctness, does not prove full
Cargo compatibility, does not prove release reproducibility, does not prove
deploy success, and does not prove general Nix replacement completeness.

## Cargo-free fixed-point proof

The Cargo-free fixed-point lane is the bounded native Rust topology path. Keep
`--out` outside the source root so proof artifacts do not perturb source digests.

```bash
mantle self-build --cargo-free --out /tmp/mantle-cargo-free
```

When using the standalone script rail directly, keep the same evidence boundary
and run its preflight before the expensive path:

```bash
nix develop -c cargo -Zscript scripts/prove-cargo-free-fixed-point.rs --check --root .
```

Inspect these output files in the selected output directory:

- `meta.json` — `mantle-cargo-free-fixed-point-proof-v1` schema, status,
  `fixed_point`, stage summaries, source-built toolchain closure status,
  blocker, and `non_claims`.
- `preflight.json` — selected root, toolchain and policy facts before stage
  execution.
- `non-claims.txt` — human-readable claim exclusions.
- `stage1/receipt.json` and `stage2/receipt.json` — native topology execution
  receipts.
- `stage1/stderr.txt`, `stage2/stderr.txt`, `stage1/smoke-stdout.txt`, and
  `stage2/smoke-stdout.txt` — execution diagnostics and smoke output.

A successful Cargo-free fixed-point report requires `status = "success"`,
`fixed_point = true`, matching stage binary BLAKE3 digests, no blocker, and
bounded non-claims. A blocked report with `status = "blocked"` is still useful
frontier evidence, but it is not a Nix-free fixed-point success claim.

This lane does not prove compiler correctness, does not prove full Cargo
compatibility, does not prove release reproducibility, does not prove deploy
success, and does not prove general Nix replacement completeness.

## Nix-free demo bundle validation

Nix-free demo wording is claimable only from the demo-profile validator. The
machine summary schema is `mantle-nix-free-demo-summary-v1`, and the current
profile is `source-root-cargo-free-fixed-point`. Use `mantle --json
nix-free-demo validate <summary.json>` for a stable machine decision and
`mantle nix-free-demo readme <summary.json>` to regenerate the derived demo
README; do not invent a wider CLI success claim.

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
and the diagnostic codes instead.

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
```

The guard checks that this guide still names the current command snippets,
important bundle paths, required demo summary fields, outcome vocabulary, and
bounded non-claims. Its self-test covers positive validation plus negative stale
command, missing bundle field, and overbroad proof-claim fixtures.
