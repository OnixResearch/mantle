# Core-backed pinned KernelScript probe observation evidence

- Date: 2026-07-12
- Branch: `agent/kernelscript-core-adapter`
- Question: Does the tracked production Nix shell delegate exact generated-shape
  classification and receipt construction to `crunch-kernelscript-core` while
  preserving the locked probe observation and external target/module blockers?
- Inspected evidence: isolated baseline/final Rust tests, the actual production
  package and structural check, retained core reports, and the exact-kernel VM
  test/log listed below.
- Decision: **yes for the adapter seam and exact Linux `6.18.20` probe
  observation only**. The compiler observation is admitted through the core.
  Both core receipts keep `target_identity_blake3 = null`, carry
  `kernel-target-observation-only`, emit no candidate packs, and retain strict
  non-claims. The private/kfunc receipt additionally carries
  `module-build-and-vm-gate-absent`; that external task remains unchecked.
- Owner: Mantle KernelScript experiment maintainers.
- Next action: leave the change active and obtain separate authoritative
  Onix/module/kfunc evidence before touching the remaining unchecked tasks.

## Baseline before edits

Pueue task `1326`:

```console
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-baseline-core \
  cargo test -p crunch-kernelscript-core
```

```text
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Pueue task `1327`:

```console
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-baseline-integration \
  cargo test -p mantle --test kernelscript_experiment
```

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Production adapter boundary

`crunch-kernelscript-adapter` is a small std binary/library. Its imperative
shell:

- validates hard request bounds before reading source/generated files;
- uses no-follow descriptor opens and descriptor metadata;
- caps generated member count, per-file bytes, and aggregate bytes before
  allocation;
- remeasures the official archive SHA-256 interoperability digest and BLAKE3;
- reads the filtered repo-owned `packages/kernelscript-experiment/` source;
- delegates profile/compiler/target admission, codegen planning, exact generated
  classification, compilation planning, and canonical receipt construction to
  `crunch-kernelscript-core`.

The tracked Nix shell invokes the adapter twice, once for the probe and once for
the private/kfunc generated shape. It no longer contains the previous
`find`/`sort`/`diff`/`wc` exact-shape classifier or hand-built core receipt.
Generated Makefile/Kbuild files remain evidence-only.

The actual production command in pueue task `1808` returned:

```console
nix build --no-link --print-out-paths \
  .#packages.x86_64-linux.kernelscript-production
```

```text
/nix/store/16bs0rqm0nlxp32lq99ylvl35ck7710h-mantle-kernelscript-production-artifacts
```

The production structural check in pueue task `1783` returned:

```console
nix build --no-link --print-out-paths \
  .#checks.x86_64-linux.kernelscript-production
```

```text
/nix/store/p93ggk2wc7pp06i9gfqbpqwrvydhx0ic-mantle-kernelscript-production-structural-check
```

The check proves both actual generated member arrays, report/receipt digest
linkage, admitted compiler observations, rejected target authority, empty
candidate packs, receipt non-claims, and the unchanged module blocker.

## Current core identities and blockers

Probe core report:

- report BLAKE3:
  `a9a79adadba7304333579810ceeb64461310d6bdb3efd7cc24fd27ba0d300211`;
- generated manifest BLAKE3:
  `1f014c68a41c0c9f761a0b8aeb1921317133688112a83a8acb0edfd380d50698`;
- receipt BLAKE3:
  `0e2052c92f7c04cd02d4b8bbe794eeed108fd6f1d76d17dc03a99be49e23fd0b`;
- `target_identity_blake3: null`;
- blocker: `kernel-target-observation-only`.

Private/kfunc core report:

- report BLAKE3:
  `e7f9eeb219fe08abd2576dbb006de13886e902e25c64ae829562e996fa977668`;
- generated manifest BLAKE3:
  `4d60420961566a43e722a9c14bd34be71a546a6e46007fab7f09d5756f646fa5`;
- receipt BLAKE3:
  `e2e286e2290ffda8448d9497ee93b4e86eedcf0f1b2ed5b30069972a92d276fd`;
- `target_identity_blake3: null`;
- blockers: `kernel-target-observation-only` and
  `module-build-and-vm-gate-absent`.

The retained `mantle-kernelscript-probe-observation-v1` digest is:

```text
f25eea0a0abfae600c1811d7c8de229b91a822ff3866765a87a26054176b4e15
```

Its authority status remains
`reported-onixos-metadata-not-materialized-or-accepted`. It records the same
probe object/loader identities as before and still says:

```text
module_status = blocked-no-checked-nix-build-or-vm-load-gate
```

## Exact-kernel VM observation

Pueue task `1784`:

```console
nix build --no-link --print-out-paths \
  .#packages.x86_64-linux.kernelscript-production-runtime-check
```

```text
/nix/store/izr0kp7wsg0njza8zr98nyx0ns77ylx9-vm-test-run-mantle-kernelscript-production-runtime
```

Current `nix log` evidence from pueue task `1822`:

```text
machine: must succeed: test -e /sys/fs/bpf/mantle-probe && rm /sys/fs/bpf/mantle-probe
machine: must succeed: /nix/store/16bs0rqm0nlxp32lq99ylvl35ck7710h-mantle-kernelscript-production-artifacts/artifacts/probe_do_exit
(finished: run the VM test script, in 12.80 seconds)
test script finished in 12.83s
```

The checked test also requires `uname -r == 6.18.20`, bpftool verifier/load,
and generated-loader attach/detach markers. This is exact-cohort observation,
not generalized target authority.

## Final focused Rust evidence

All Cargo targets were isolated under `/tmp`.

- Pueue `1778`: `cargo test -p crunch-kernelscript-core` — `20 passed`.
- Pueue `1779`: `cargo test -p crunch-kernelscript-adapter` — `9 passed`.
- Pueue `1781`: `cargo test -p mantle --test kernelscript_experiment` —
  `6 passed`.
- Pueue `1780`: strict Clippy for core + adapter finished successfully.
- Pueue `1809`: package-scoped rustfmt plus the leaf integration file passed.
- Pueue `1810`: `cargo -Zbuild-std=core,alloc check -p
  crunch-kernelscript-core --target wasm32-unknown-unknown` finished
  successfully.

Positive coverage proves exact core admission, deterministic receipt identity,
and production-shell/core parity. Negative coverage proves extra/missing shape
rejection, hard pre-read bounds, aggregate/count caps, actual archive SHA-256
drift rejection, no-follow symlink rejection, descriptor replacement safety,
claim boundaries, and preserved module blocking.

## Non-claims

This evidence does not claim language/compiler soundness, accepted Onix target
materialization, kernel safety, cross-kernel compatibility, core output
inspection, module build/load/unload, private-kfunc/XDP runtime behavior,
deployability, ChaosControl validation, production readiness, default
enablement, release eligibility, or external authority completion.
