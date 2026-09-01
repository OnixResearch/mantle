# V59 recursive minicargo GCC-prefix blocker

## Question

Did the V57 mrustc link repair preserve parent-path rejection and reach the next real Rust-provider boundary?

## Result

Yes. V59 built and linked `bin/mrustc`, then ran `objcopy` and `strip` successfully.

The next recursive make entered `tools/minicargo/`. Its Makefile replaced inherited compiler flags.

Four parallel C++ compiles then requested `cc1plus` through this path:

```text
<staging>/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/../libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1plus
```

The action audit recorded 1,113 events. It matched 1,109 events and denied four events.

Each denied event reported:

```text
tracee exec path contains unsupported parent component
```

No provider checkpoint was published.

## Source generation

V59 used these exact inputs:

- source commit `979c78dc`;
- release binary BLAKE3 `0c4de27b5ad2fa776638f5c0c54fd72d48ecd7997766ac57f63bb3d7664759f7`;
- source-profile BLAKE3 `f30a9f8ba37f5ca7a5400f2a2c606b78397e41f9a7cbbb7ad72c2efe93a01c07`;
- exact binary round-trip parity;
- exact source checksum parity;
- strict hermeticity;
- substitution disabled;
- 16 jobs; and
- the 700 GB proof bound.

The detached wrapper PID was `1950611`, with start ticks `19410073`.

## Approach registry

| Family | Mechanism checked | Evidence | State |
|---|---|---|---|
| V57 repair | Route the receipt-derived `-B` through mrustc `LINKFLAGS_EXTRA`. | `mrustc-first-stage-build.log` shows successful mrustc link, `objcopy`, and `strip`. | Validated |
| Missing authority | Check whether canonical `cc1plus` had fixed digest authority. | `rust-provider-action-authority.json` binds `cc1plus`; denials occurred before hashing. | Falsified |
| Recursive make propagation | Check minicargo's compile and link variable seams. | `minicargo-make-rules.txt` shows `CXXFLAGS_EXTRA` and `LINKFLAGS_EXTRA`; recursive make did not receive either. | Validated |

These serial lenses are correlated. The runtime audit and preserved Makefile rules provide deterministic evidence.

## Decision

Keep global parent-component rejection unchanged.

Pass the receipt-derived GCC `-B` option into the recursive minicargo make as both variables:

```text
CXXFLAGS_EXTRA="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG"
LINKFLAGS_EXTRA="$MANTLE_GCC_SUBPROGRAM_PREFIX_FLAG"
```

Keep compatibility mode unchanged. It must not invent full-source GCC authority.

## Validation

`post-repair-validation.log` records these passing checks:

- 61 serialized `rust_source_provider::tests::` tests;
- the parent-component rejection regression test;
- strict scoped Clippy with the three existing named allowances;
- `cargo fmt -p mantle --check`; and
- `git diff --check`.

The Rust-provider module includes positive full-source routing and negative compatibility-mode coverage.

## Adversarial audit

The repair does not permit `..`, authorize a directory, or set `GCC_EXEC_PREFIX`.

It does not use `COMPILER_PATH`, path-only trust, or producerless executable authority.

V59's incomplete provider state remains diagnostic only. It is not eligible for checkpoint import.

`checkpoint-inventory.txt` records only the historical V48 checkpoint.

## Inspected evidence

- `source-built-fixed-point-v59-mrustc-linkflags-cold-detached-launch-status.txt`
- `source-built-fixed-point-v59-mrustc-linkflags-cold-wrapper-status.txt`
- `source-built-fixed-point-v59-mrustc-linkflags-cold-host-facts.txt`
- `source-transfer-v59.txt`
- `source-profile-v59-status.txt`
- `source-profile-v59-verification.log`
- `attempt-status.json`
- `proof-tail.log`
- `mrustc-first-stage-plan.json`
- `mrustc-first-stage-build.log`
- `mrustc-to-rust-1_90_0-plan.json`
- `mrustc-to-rust-1_90_0-audit.json`
- `mrustc-to-rust-1_90_0-reconciliation.json`
- `rust-provider-action-authority.json`
- `minicargo-make-rules.txt`
- `checkpoint-inventory.txt`
- `post-repair-validation.log`
- `file-digests.blake3`

## Owner

Mantle owns generated Rust-provider scripts and recursive make argument routing.

## Next action

Commit the narrow recursive-make repair. Then build and launch a fresh cold proof generation.

## Non-claims

This evidence does not prove minicargo completion, provider admission, checkpoint publication, fixed-point equality, or complete action trust.
