# Core-backed pinned KernelScript probe observation evidence

- Date: 2026-07-12; updated 2026-07-15 after private-kfunc materialization.
- Branch: `agent/kernelscript-core-adapter`
- Question: Does the tracked production Nix shell delegate exact generated-shape
  classification and receipt construction to `crunch-kernelscript-core` while
  preserving the locked probe observation and external target authority blocker?
- Inspected evidence: isolated baseline/final Rust tests, the actual production
  package and structural check, retained core reports, the exact-kernel VM
  test/log listed below, and `downstream-authority-attempt-2026-07-15.md`.
- Decision: **yes for the adapter seam and exact Linux `6.18.20` probe plus
  private-kfunc observation**. The compiler observation is admitted through the
  core. Both core receipts keep `target_identity_blake3 = null`, carry
  `kernel-target-observation-only`, emit no candidate packs, and retain strict
  non-claims. The prior private/kfunc `module-build-and-vm-gate-absent` blocker
  is removed by the checked module build and exact-kernel VM load/attach smoke.
- Owner: Mantle KernelScript experiment maintainers.
- Next action: leave the change active and obtain separate accepted OnixOS
  runtime-adapter authority plus a ChaosControl `kernel-bundle/vm-compat-smoke`
  receipt before archive.

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
Generated Makefiles remain evidence-only; the generated `Kbuild` is consumed only by the bounded module-build step after the core records the exact generated shape.

The current production command in pueue task `34` returned:

```console
nix build --no-link --print-out-paths \
  .#packages.x86_64-linux.kernelscript-production
```

```text
/nix/store/6cz7sqcq3mp7vnpz2nipcjx600b15fxv-mantle-kernelscript-production-artifacts
```

The current production structural check in pueue task `33` passed:

```console
nix build --no-link \
  .#checks.x86_64-linux.kernelscript-production
```

The check proves both actual generated member arrays, report/receipt digest
linkage, admitted compiler observations, rejected target authority, empty
candidate packs, receipt non-claims, and built/static-inspected probe plus
private-kfunc artifacts.

## Current core identities and blockers

Probe core report:

- report BLAKE3:
  `15f63fe59beff5da6d4f67b60271556f63e7ecff9e8c8fd1b21293c24bba39ae`;
- generated manifest BLAKE3:
  `224708662bf054022fe6697bc7816a18f338f811a91bd5d7fb312885aa9818b8`;
- receipt BLAKE3:
  `47f3d5052ca5248b789138704ca898f593fbc36323fb6277aefbebaeac512df5`;
- `target_identity_blake3: null`;
- blocker: `kernel-target-observation-only`.

Private/kfunc core report:

- report BLAKE3:
  `8b90a042ac23f36562f07587167a19f08bba6efd1dccc9cbe79556cb57f5257e`;
- generated manifest BLAKE3:
  `163b9edade98ba2a6637357bc6488c4088bbdc563f05747c2604436d6b84a1cf`;
- receipt BLAKE3:
  `c53720b0ca626b75ee36e14a3c694420308b9705204f08773caf38a2a134af84`;
- `target_identity_blake3: null`;
- blocker: `kernel-target-observation-only`.

The retained `mantle-kernelscript-probe-observation-v1` digest is:

```text
1f3c566a24981f3518709bc5fba1342f93c0dd4fbcc46d55193040f0a5170d05
```

Its authority status remains bounded: the current Mantle observation records
`module_status = checked-nix-module-build`, `runtime_status = checked-by-separate-nixos-vm-smoke`, and `core_admission_status = generated-shapes-admitted-receipts-blocked-on-external-target-authority`. It records the same private-kfunc module/BPF bytes later canonicalized by OnixOS kernel-bundle inspect, but Mantle still does not claim accepted OnixOS runtime-adapter authority or ChaosControl behavior evidence.

## Exact-kernel VM observation

Pueue task `33`:

```console
nix build --no-link \
  .#packages.x86_64-linux.kernelscript-production-runtime-check
```

The checked test requires `uname -r == 6.18.20`, bpftool verifier/load, probe
loader execution, `insmod private_kfunc.mod.ko`, XDP load of
`private_kfunc.ebpf.o`, generated private-kfunc loader attach/detach markers,
and cleanup. This is exact-cohort Mantle/NixOS VM observation, not generalized
target authority and not a ChaosControl receipt.

## Final focused Rust evidence

All Cargo targets were isolated under `/tmp`.

- Pueue `23`: `cargo test -p crunch-kernelscript-core -p crunch-kernelscript-adapter` — `9 passed` in the adapter suite and `20 passed` in the core suite.
- Pueue `22`: `cargo test -p mantle --test kernelscript_experiment` —
  `6 passed`.
- Pueue `24`: production structural check and runtime VM check finished
  successfully.
- Earlier retained evidence also includes strict Clippy for core + adapter,
  package-scoped rustfmt plus the leaf integration file, and `cargo
  -Zbuild-std=core,alloc check -p crunch-kernelscript-core --target
  wasm32-unknown-unknown`.

Positive coverage proves exact core admission, deterministic receipt identity,
and production-shell/core parity. Negative coverage proves extra/missing shape
rejection, hard pre-read bounds, aggregate/count caps, actual archive SHA-256
drift rejection, no-follow symlink rejection, descriptor replacement safety,
claim boundaries, and preserved external target-authority blocking.

## Non-claims

This evidence does not claim language/compiler soundness, accepted OnixOS
runtime-adapter authority, target mutation authority, kernel safety,
cross-kernel compatibility, deployability, ChaosControl validation, production
readiness, default enablement, release eligibility, or external authority
completion.
