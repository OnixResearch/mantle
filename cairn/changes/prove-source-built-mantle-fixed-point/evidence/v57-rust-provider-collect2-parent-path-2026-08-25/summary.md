# V57 Rust-provider `collect2` parent-path blocker

## Question

Why did cold proof V57 stop before publishing the current 17-payload provider checkpoint?

## Result

V57 failed closed during the first mrustc stage. The wrapper exited with status 1.

The stage recorded 1,067 execution events. It matched 1,066 events and denied one event.

The denied request was:

```text
<staging>/native-store/kcvijyh0sibqcc4g9sqk694xyslzhpsx-full-source-seed-toolchain/bin/../libexec/gcc/x86_64-unknown-linux-musl/10.5.0/collect2
```

The audit reason was:

```text
tracee exec path contains unsupported parent component
```

The canonical `collect2` file existed with mode `0555`. The action authority also bound its exact path and BLAKE3 digest:

```text
c1386970f8c7035643f5cf990c245f4143eb72db02bb3674c15c59b952b7592a
```

Therefore, missing bytes, missing execute mode, and missing digest authority did not cause the failure.

## Approach registry

| Family | Mechanism checked | Evidence | State |
|---|---|---|---|
| File admission | Check the canonical `collect2` file and execute mode. | Remote `stat` reported a regular file with mode `0555`. | Falsified |
| Action authority | Check whether the plan bound canonical `collect2` bytes. | `rust-provider-action-authority.json` binds the canonical path and digest. | Falsified |
| GCC route propagation | Compare mrustc compile and link flag seams. | V57 compiled all mrustc objects, then failed at the final link. The preserved Makefile uses `LINKFLAGS`, not `LDFLAGS`, for that link. | Validated |

These serial lenses are correlated. The runtime audit and preserved Makefile provide the deterministic evidence.

## Decision

Keep global parent-component rejection unchanged.

Carry the receipt-derived GCC `-B` flag through mrustc's `LINKFLAGS_EXTRA` input. Do not authorize `bin/..` paths.

Keep compatibility mode unchanged. It must not invent full-source GCC authority.

## Validation

`post-repair-validation.log` records these passing checks:

- 59 serialized `rust_source_provider::tests::` tests;
- strict scoped Clippy with the three existing named allowances;
- `cargo fmt -p mantle --check`;
- `git diff --check`; and
- the parent-component rejection regression test.

The focused module includes positive full-source routing and negative compatibility-mode coverage.

## Adversarial audit

The repair must not do any of these actions:

- permit parent components in protected execution paths;
- grant authority to the GCC directory;
- use `GCC_EXEC_PREFIX` or `COMPILER_PATH`;
- trust file names without BLAKE3 binding;
- import V57's incomplete provider state; or
- claim that V57 published a new checkpoint.

`checkpoint-inventory.txt` records one checkpoint manifest. It is the historical V48 checkpoint.

## Inspected evidence

- `source-built-fixed-point-v57-gcc-b-prefix-retry-wrapper-status.txt`
- `attempt-status.json`
- `proof-tail.log`
- `mrustc-first-stage-plan.json`
- `mrustc-first-stage-build.log`
- `mrustc-to-rust-1_90_0-plan.json`
- `mrustc-to-rust-1_90_0-audit.json`
- `mrustc-to-rust-1_90_0-reconciliation.json`
- `rust-provider-action-authority.json`
- `mrustc-Makefile-link-rule.txt` (focused excerpt; source BLAKE3 retained)
- `checkpoint-inventory.txt`
- `post-repair-validation.log`
- `file-digests.blake3`

## Owner

Mantle owns the generated first-stage mrustc command and its action-authority boundary.

## Next action

Add the narrow `LINKFLAGS_EXTRA` route with positive and negative tests. Then rebuild and launch a new cold proof generation.

V58 restoration remains blocked until a current eligible 17-payload checkpoint exists.

## Non-claims

This evidence does not prove mrustc completion, provider admission, fixed-point equality, complete action trust, or release eligibility.
