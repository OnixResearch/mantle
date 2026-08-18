# Perl 5.005_03 GCC runtime validation

Date: 2026-07-25

Requirement: `r[bootstrap_inventory.perl_5_005_03_gcc_runtime]`

## Decision

The canonical Perl 5.005_03 compiler configuration was not the failure. Once its declared predecessor chain was materialized correctly, the unchanged GCC 4.0.4, `-O2`, no-`LONGSIZE` target built and passed the complete runtime contract. The smallest correction is therefore the predecessor handoff in `bootstrap/perl-5.000-gcc.ncl`: consume GCC, musl, binutils, stage0, and sed from `gcc-generator-base-v4`, and invoke the relocated compiler with its explicit `-B`, libc, libm, and CRT seams.

The canonical Perl 5.005_03 file changes only its stale malformed-source diagnostic label from `5.003` to `5.005_03`. No optimization, LP64-size, or compiler-generation workaround was promoted.

## Mechanism registry

| Mechanism | Observed outcome | Decision |
|---|---|---|
| Host build without the repository Nix development environment | Stopped in sandbox precursor setup before Perl | Environment/precursor evidence only |
| `/crunch/store` canonical target before predecessor materialization | First stopped at GCC 4.0 C++; after that was built, stopped at Perl 5.004_05 → Perl 5.003 → Perl 5.000 | Predecessor evidence only |
| Perl 5.000 raw pass4 input lookup | `ERROR: input gcc-4.0.4-musl-pass4-v5 not found` | Root cause: violated normalized generator-base boundary |
| Normalized base paths without relocated GCC linker seams | Objects compiled; final link failed on `crt1.o`, `crti.o`, `-lm`, and `-lc` | Incomplete handoff, rejected |
| Normalized base paths plus explicit relocated GCC seams | Perl 5.000, 5.003, and 5.004_05 built successfully | Smallest causal correction |
| Canonical Perl 5.005_03 at GCC 4.0.4 and `-O2` | Built and passed version, arithmetic, malformed-source, empty-stdout, and ELF64 checks | Accepted; target compilation was already sufficient |
| `-O0` diagnostic | Not executed after canonical target passed | Unnecessary workaround; removed |
| `LONGSIZE=8` diagnostic | Not executed after canonical target passed | Unnecessary target mutation; removed |
| GCC 10 diagnostic | Not executed after canonical target passed | Unnecessary bootstrap-lineage change; removed |

## Causal isolation transcript

- Pueue task `170` built `bootstrap/gcc-4.0-musl-cxx.ncl` successfully under `nix develop`, `/crunch/store`, `CRUNCH_NO_FUSE=1`, and `--no-substitute`.
- Pueue task `230` reached Perl 5.000 and failed with `ERROR: input gcc-4.0.4-musl-pass4-v5 not found`.
- Pueue task `243` traced the first normalized-base attempt through successful object compilation; the final link failed because raw `crt1.o`, `crti.o`, `-lm`, and `-lc` were unresolved.
- Pueue task `250` built the corrected Perl 5.000 handoff successfully.
- Pueue tasks `252` and `256` then built Perl 5.003 and Perl 5.004_05 successfully.
- Pueue task `261` built the canonical Perl 5.005_03 target successfully before any target compiler/configuration workaround was applied.

This sequence distinguishes every precursor failure from target behavior. It does not count any failed predecessor run as Perl 5.005_03 optimization, ABI, compiler-generation, or runtime evidence.

## Canonical default-prefix build

Pueue task `284` ran:

```text
mkdir -p /tmp/mantle-perl-default-store && nix develop -c env CRUNCH_NO_FUSE=1 \
  $HOME/.cargo-target/debug/mantle build --json \
  --store /tmp/mantle-perl-default-store \
  --state-dir $HOME/.local/state/crunch \
  -j 1 --no-substitute bootstrap/perl-5.005_03-gcc.ncl
```

Result:

```text
status=success
succeeded_total=1
built_total=1
cached_total=0
failed_total=0
logical_path=/mantle/store/wx66fhkv2w4y275zqkl03kr2b9msl5s9-perl-5.005_03-gcc-v7
physical_path=/tmp/mantle-perl-default-store/wx66fhkv2w4y275zqkl03kr2b9msl5s9-perl-5.005_03-gcc-v7
```

The JSON report is retained locally at `target/perl-5.005_03-triage/final-default-prefix.stdout.json`.

## Independent runtime checks

Pueue task `326` executed the produced default-prefix artifact directly. It required the version check and arithmetic result to pass, required malformed `sub {` input to fail, required rejection stdout to be empty, and required rejection stderr to be nonempty:

```text
prefix=/mantle/store
version=5.005_03
runtime_sum=42
malformed_status=nonzero
malformed_stdout=empty
malformed_stderr=nonempty
```

The builder also performs the same positive/negative checks and verifies `ELF64` with the declared source-built binutils `readelf`; task `284` could not succeed unless those checks passed.

## Immediate downstream consumer

Pueue task `327` built `bootstrap/perl-5.6.2-gcc.ncl` under the same default `/mantle/store` identity and physical store:

```text
status=success
succeeded_total=1
built_total=1
cached_total=0
failed_total=0
logical_path=/mantle/store/6rxzzfl89lrvvwdpvqf4j0xqbww5d2by-perl-5.6.2-gcc-v19
```

This is focused downstream build evidence, not a claim about the complete autotools ladder or whole bootstrap.

## Regression and quality rails

- Pre-change pueue task `156`: `bootstrap_eval` passed `19` tests.
- Post-change pueue task `273`: the focused positive and negative Perl contract tests passed `2` tests.
- Post-change pueue task `274`: `bootstrap_eval` passed `21` tests with zero failures.
- Pueue task `328`: `cargo fmt --check -p mantle -v` passed.
- Pueue task `329`: `cargo clippy -p mantle --test bootstrap_eval --no-deps -- -D warnings` passed; the displayed `snix-castore` dead-code warning came from a dependency and was outside `--no-deps` first-party denial.
- Pueue task `332`: source-pin audit passed for both touched Nickel files: `2 files, 2 fetch blocks, 0 issues`.
- `git diff --check` passed before lifecycle evidence finalization.

The regression core in `tests/bootstrap_eval.rs` checks required and forbidden source markers as a pure function. Positive coverage preserves the normalized predecessor handoff and target runtime rails; negative coverage proves the stale malformed-source diagnostic is rejected.

## Adversarial review and claim boundary

A secondary VibeThinker review inspected the normalized-base path selection, relocated GCC seams, causal conclusion, positive/negative coverage, and recorded build evidence. Decision: `no blocker found`. This review is advisory; the executed repository checks above are the acceptance evidence.

This evidence proves the recorded x86_64-linux builds and focused runtime behavior under the declared source-built predecessor chain and the recorded `/mantle/store` identity. It does not prove compiler correctness, independent clean-room reproducibility, normalized-provider admission, release eligibility, the full autotools ladder, or whole-bootstrap correctness.

## Pre-sync lifecycle validation

Final active-change validation reported no issues or findings, `valid: true`, and all six tasks complete. The final gates passed with receipts:

```text
proposal=32cbc2a5aeb843e93b2b0c98629b40269528db9a63270bbd6d089a91ac3dd6cc PASS
design=064c35471fe721ea2cc088a95d06ffd7a4e4c3fd59a5b4a0ecbfc83bdd1eaf07 PASS
tasks=2eaa27ece7b8027df73dc934b18088af82754dd1f0d5cebd6cec1495a5742e8d PASS
```

Cairn sync executed with receipt `d34847e296b3672c2b6f86bb92805bb5bb56c3bd95959936606778b4bc0743c7` and mutation-manifest hash `ba13f75ee7d3a0f7effc59cb80a37c3e5663e1d32b3ff1520caadefde78dc817`. The accepted requirement was inspected intact at `cairn/specs/bootstrap-inventory/spec.md` lines 249–293, including all six scenarios and the bounded non-claims.

## Archive and post-archive validation

Cairn archive executed to `cairn/archive/2026-07-25-repair-perl-5-005-03-gcc-runtime` with receipt `594f8bf1d40b46f2e959c9034f025a5806fe5fdc08fbf5068ba1b71cb0393bd5` and mutation-manifest hash `ab8238758b998a38213cfccc1f193f11cffef3c1224aac44eba7cb48f7331b8e`.

Command: `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`

```text
{
  "change_issues": [],
  "changes": 0,
  "cross_repo_dependencies": [],
  "cross_repo_evidence_issues": [],
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "./cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 36,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 25,
      "scenario_blocks": 59,
      "substantive_requirement_blocks": 25
    },
    {
      "path": "./cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/examples/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 22,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 22
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "./cairn/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 32,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 24,
      "scenario_blocks": 80,
      "substantive_requirement_blocks": 24
    },
    {
      "path": "./cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 66,
      "scenario_blocks": 93,
      "substantive_requirement_blocks": 66
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 35,
      "scenario_blocks": 109,
      "substantive_requirement_blocks": 35
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 129,
      "scenario_blocks": 447,
      "substantive_requirement_blocks": 129
    },
    {
      "path": "./cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "./cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/store-transports/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 26,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 58,
      "scenario_blocks": 164,
      "substantive_requirement_blocks": 58
    },
    {
      "path": "./cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 32,
  "substance": [],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}

```
