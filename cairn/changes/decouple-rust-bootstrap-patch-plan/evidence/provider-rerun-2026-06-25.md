# Provider rerun after patch-plan refactor — 2026-06-25

Task-ID: V3
Covers: rust_package_planning.source_built_toolchain_closure.provider_contract_independence

## Rerun 38 launch

A real source-built Rust provider rerun was queued from committed refactor code.
This is the V3 long-running proof that the patch-plan refactor keeps the provider
frontier at least as advanced as the pre-refactor rerun37 baseline.

- Pueue task: `1158`
- Commit: `deb16a2b`
- Run root: `target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25`
- Source root: `/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain`
- Target toolchain root: `/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain`
- Output: `target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out`
- Smoke evidence dir: `target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/smoke`

Launch status excerpt:

```text
commit=deb16a2b
run_root=target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25
source_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
target_toolchain_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
output=target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out
tmpdir=target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/tmp
started_unix=1782360223
```

## Current status

The rerun is still in progress. V3 remains unchecked until `status.txt`, provider
metadata/receipt, smoke evidence, and final binary probes are inspected.
