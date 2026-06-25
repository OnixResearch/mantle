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

## Completed status

Pueue task `1158` completed successfully. The provider command exited with
`status=0` after `17855` seconds (`4h 57m 35s`). The run completed from commit
`deb16a2b`, the committed patch-plan refactor code.

Final `status.txt`:

```text
commit=deb16a2b
run_root=target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25
source_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
target_toolchain_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
output=target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out
tmpdir=target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/tmp
started_unix=1782360223
status=0
finished_unix=1782378078
```

Completion excerpt from `stderr.txt`:

```text
Materialized Rust source provider /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out
  recipe_digest_blake3: 77ff5fb4416ae590bd209c756ff02afe9eeeb11c6026139af6deb5505480b441
  metadata_path: /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out/share/mantle-rust-provider/provider.json
  metadata_digest_blake3: 526d6decd98515e0f3f758e30c43f93ed15658923087c2eb55b3350bd07a4158
Smoked Rust source provider /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out
  rustc_path: /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/provider-out/bin/rustc
  target_triple: x86_64-unknown-linux-musl
  smoke_output_digest_blake3: f4432c44b479f40b8d72c30e586f501e83682c5cfbfc9448d2a11e879fe7d48b
  smoke_evidence_summary: /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-patch-plan-rerun38-2026-06-25/smoke/smoke.json
  smoke_evidence_metadata_digest_blake3: 526d6decd98515e0f3f758e30c43f93ed15658923087c2eb55b3350bd07a4158
  smoke_evidence_policy_digest_blake3: 6636766cbc070f531db3fbcb7da40f8617d311a40d2a5141096f8c76edcabc08
```

Provider metadata summary from `provider-out/share/mantle-rust-provider/provider.json`:

```text
schema: mantle-rust-source-provider-v1
host_triple: x86_64-unknown-linux-musl
target_triple: x86_64-unknown-linux-musl
source_built: true
uses_prebuilt_rust: false
rustc digest: fc8954789b9c63e3fa820aefeb882158d4abec75acf339d8b62fe005f5104b61
cargo digest: d3c5a0fdda5d1aad2b49a0d10d449b97c86a500f4f7b7ed633664bbcbf1cd5c1
rustdoc digest: 2e87d90e03552622e88da51f2b3f3e1c8e38ae9189210d47d0b2b527f16b1adc
receipt digest: 1d7f579fe0fd804a9fe1a8b4ad32f80f37d0bf5b542bdf5d7ac885412459f885
```

Receipt-bound patch-plan evidence from
`provider-out/share/mantle-rust-provider/receipts/build.json`:

```text
name: record-rust-bootstrap-patch-plan
program: mantle-rust-bootstrap-patch-plan
schema=mantle-rust-bootstrap-patch-plan-v1
stage=RustBootstrap
stage-id=rust-1.94.0-final
route=rust-source-musl-host-plan
host=x86_64-unknown-linux-musl
target=x86_64-unknown-linux-musl
rust-version=1.94.0
input-digest=dd3766dbcedb1596c0d3aa889ecdd40be1ab8415518a5e2ad8bfb782cdee51a1
output-digest=01d2a3d0ed91864b9d81810303385cad27eb047d25367f59fd174d37c7d44275
operation-count=6
operation=rust-bootstrap-target-tool-config:RustBootstrapTargetToolConfig:bind source-root target tools and runtime paths in Rust bootstrap config:source=rust-1.94.0
operation=rust-bootstrap-workspace-isolation:RustBootstrapWorkspaceIsolation:keep optional compiler workspaces outside the bootstrap member set:source=rust-1.94.0
operation=rust-bootstrap-rustc-driver-rlib:RustBootstrapRustcDriverRlib:build rustc_driver as rlib for static musl compiler host:source=rust-1.94.0
operation=rust-bootstrap-sysroot-fallback:RustBootstrapSysrootFallback:prefer explicit sysroot environment before dynamic compiler fallback:source=rust-1.94.0
operation=rust-bootstrap-rustc-private-tool-rlibs:RustBootstrapRustcPrivateToolRlibLookup:teach rustc-private tools where stage2 compiler rlibs live:source=rust-1.94.0
operation=provider-contract-assertion:ProviderContractAssertion:assert provider contract mantle-rust-source-provider-v1 from explicit route facts
```

Smoke evidence from `smoke/smoke.json`:

```text
schema: mantle-rust-source-provider-smoke-evidence-v1
metadata_digest_blake3: 526d6decd98515e0f3f758e30c43f93ed15658923087c2eb55b3350bd07a4158
policy_digest_blake3: 6636766cbc070f531db3fbcb7da40f8617d311a40d2a5141096f8c76edcabc08
output_digest_blake3: f4432c44b479f40b8d72c30e586f501e83682c5cfbfc9448d2a11e879fe7d48b
target_triple: x86_64-unknown-linux-musl
```

Final binary probe from pueue task `1232`:

```text
rustc: rustc 1.94.0 (4a4ef493e 2026-03-02) (built from a source tarball)
cargo: cargo 1.94.0 (85eff7c80 2026-01-15) (built from a source tarball)
rustdoc: rustdoc 1.94.0 (4a4ef493e 2026-03-02) (built from a source tarball)

smoke stdout bytes: 0
smoke stderr bytes: 0
```

Conclusion: the patch-plan refactor did not regress the source-built Rust
provider frontier. It still materializes and smokes a source-built `rustc`,
`cargo`, and `rustdoc` 1.94.0 provider, and it now receipt-binds the final Rust
bootstrap patch plan.
