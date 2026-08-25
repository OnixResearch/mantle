# V60 direct runtime-shim GCC-prefix blocker

## Question

Did V60 carry the receipt-derived GCC route through mrustc and recursive minicargo?

## Result

Yes. V60 built, linked, debug-split, and stripped both `mrustc` and `minicargo`.

The next step prepared the source-root musl LLVM host runtime. Its direct libatomic-shim compile invoked `gcc.real` without the reviewed `-B` option.

GCC then requested:

```text
<staging>/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/../libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1
```

The stage audit recorded 1,218 events. It matched 1,217 events and denied one event.

The denied event reported:

```text
tracee exec path contains unsupported parent component
```

No provider checkpoint was published.

## Source generation

V60 used these exact inputs:

- source commit `47999e54`;
- release binary BLAKE3 `15f66f9845d9453322a825dede7e3422b2daf62e0030f404502f7744f813dc39`;
- source-profile BLAKE3 `9ac96e5fab5b09fd5a8dfe17ac624ece76b517b2ec5e18a2074156515b9e811f`;
- exact binary round-trip parity;
- exact source checksum parity;
- strict hermeticity;
- substitution disabled;
- 16 jobs; and
- the 700 GB proof bound.

The detached wrapper PID was `2664527`, with start ticks `21241066`.

## Approach registry

| Family | Mechanism checked | Evidence | State |
|---|---|---|---|
| V59 repair | Pass receipt `-B` through minicargo compile and link seams. | `mrustc-first-stage-build.log` shows complete minicargo compilation, link, debug split, and strip. | Validated |
| Missing authority | Check whether canonical `cc1` had fixed digest authority. | `rust-provider-action-authority.json` binds `cc1`; denial occurred before hashing. | Falsified |
| Direct shim compile | Inspect generated raw target-compiler invocations. | The libatomic compile called `target_cc_path` directly and omitted the receipt `-B` option. | Validated |

These serial lenses are correlated. The runtime audit and generated-script source provide deterministic evidence.

## Decision

Keep global parent-component rejection unchanged.

Thread full-source mode into direct musl runtime-shim compiles. Add the receipt-derived `-B` option before fixed source and output arguments.

Apply the same rule to the LFS compatibility shim. Keep compatibility mode unchanged.

## Validation

`post-repair-validation.log` records these passing checks:

- 63 serialized `rust_source_provider::tests::` tests;
- the parent-component rejection regression test;
- strict scoped Clippy with the three existing named allowances;
- `cargo fmt -p mantle --check`; and
- `git diff --check`.

The Rust-provider module includes positive full-source routing and negative compatibility-mode coverage.

## Adversarial audit

The repair does not permit `..`, authorize a GCC directory, or normalize tracee requests.

It does not set `GCC_EXEC_PREFIX`, use `COMPILER_PATH`, or accept path-only authority.

V60's incomplete provider state remains diagnostic only. It is not eligible for checkpoint import.

`checkpoint-inventory.txt` records only the historical V48 checkpoint.

## Inspected evidence

- `source-built-fixed-point-v60-minicargo-prefix-cold-detached-launch-status.txt`
- `source-built-fixed-point-v60-minicargo-prefix-cold-wrapper-status.txt`
- `source-built-fixed-point-v60-minicargo-prefix-cold-host-facts.txt`
- `source-transfer-v60.txt`
- `source-profile-v60-status.txt`
- `source-profile-v60-verification.log`
- `attempt-status.json`
- `proof-tail.log`
- `mrustc-first-stage-plan.json`
- `mrustc-first-stage-build.log`
- `mrustc-to-rust-1_90_0-plan.json`
- `mrustc-to-rust-1_90_0-audit.json`
- `mrustc-to-rust-1_90_0-reconciliation.json`
- `rust-provider-action-authority.json`
- `checkpoint-inventory.txt`
- `post-repair-validation.log`
- `file-digests.blake3`

## Owner

Mantle owns generated Rust-provider runtime-shim commands and authority routing.

## Next action

Commit the direct compile repair. Then build and launch a fresh cold proof generation.

## Non-claims

This evidence does not prove the LLVM host setup, provider admission, checkpoint publication, fixed-point equality, or complete action trust.
