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
