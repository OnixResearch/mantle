# Downstream authority attempt for private-kfunc KernelScript cohort

- Date: 2026-07-15
- Question: Can the remaining downstream authority path for Mantle's private-kfunc KernelScript artifacts be produced from the active OnixOS and ChaosControl work without mutating user-owned downstream state?
- Decision: **partially advanced, still not complete**. Mantle now materializes the private-kfunc module, BPF object, skeleton, and loader; the module contains module BTF with `process_value` tagged `bpf_kfunc`; the focused NixOS VM gate loads the module, verifies/loads the XDP object, and runs the generated loader on Linux `6.18.20`. Existing OnixOS kernel-bundle tooling can canonicalize an exact Onix manifest for the same Mantle bytes and emits an `onix:blake3:kernel-build` identity. The active OnixOS runtime-adapter change and the active ChaosControl kernel-bundle validation change remain unimplemented/unarchived, so Mantle still does **not** have a replayable ChaosControl `kernel-bundle/vm-compat-smoke` receipt or target mutation authority.
- Owner: Mantle KernelScript experiment maintainers; downstream runtime authority remains with OnixOS and ChaosControl maintainers.
- Next action: wire the validated Onix manifest and Mantle artifact identities into the active OnixOS runtime-adapter change, then implement or narrow the ChaosControl `kernel-bundle/vm-compat-smoke` rail for this exact cohort.

## Repo state preservation

Captured after the attempt:

```text
## mantle
## main...origin/main [ahead 94]
 M nix/kernelscript-experiment.nix
?? examples/cowsay.ncl
## onixos
## main...origin/main
 M README.md
 M cairn/changes/onixos-valence-evidence-spine/metadata.json
?? .pi-last-control-fixture-evidence
?? .pi-last-fuse-current-source-fixture-evidence
?? .pi-last-fuse-fallback-evidence
?? .pi-last-fuse-fixture-evidence
?? .pi-last-template-current-source-evidence
?? cairn/changes/koi-terminal-avf-host/
?? cairn/changes/mobile-nixos-mantle-compat-artifact-ingress/
?? cairn/changes/mobile-nixos-onixos-device-descriptor/
?? cairn/changes/onixos-mantle-mobile-boot-artifact/
?? cairn/changes/realize-linux-bpf-pack-adapter/
?? libonixos_lifecycle_state_machine.rlib
## chaoscontrol
## main...origin/main
 M .gitignore
?? cairn/changes/add-kernel-bundle-validation-rail/
?? cairn/changes/add-spacewasm-mvp-differential-rail/
?? cairn/changes/complete-vm-snapshot-state/
?? cairn/changes/harden-ebpf-trace-evidence/
?? cairn/changes/harden-virtio-queue-validation/
?? cairn/changes/reject-assertion-identity-conflicts/
?? cairn/changes/remove-wall-clock-smp-preemption/
?? cairn/changes/verify-fault-application-outcomes/
?? openspec/changes/extract-replay-evidence-core/
```

Downstream worktrees were inspected read-only. The pre-existing untracked Mantle `examples/cowsay.ncl` was not modified, cataloged, or committed.

## Mantle materialization and static inspection

Command (pueue task `34`):

```console
nix build --no-link --print-out-paths .#packages.x86_64-linux.kernelscript-production
```

Result:

```text
/nix/store/6cz7sqcq3mp7vnpz2nipcjx600b15fxv-mantle-kernelscript-production-artifacts
```

Focused structural check (pueue task `33`) passed:

```console
nix build --no-link .#checks.x86_64-linux.kernelscript-production
```

The observation digest is:

```text
1f3c566a24981f3518709bc5fba1342f93c0dd4fbcc46d55193040f0a5170d05
```

The observation records:

```json
{
  "module_status": "checked-nix-module-build",
  "runtime_status": "checked-by-separate-nixos-vm-smoke",
  "private_kfunc": {
    "ebpf_object_blake3": "b8cdd1315b4066c053a14034344a1b051f85fe2c965cffdc38d79d116ebb94de",
    "loader_blake3": "87e46ac46caa731848d339c02bbeea7a3a667bc4f8a63bf9152c9f8ae01ad602",
    "module_blake3": "1a738476dabe13e3d8ae2c5b0435f7b7f2908a82fadcee136e5494f6a93a81e1"
  }
}
```

Module BTF proof:

```text
[58] .BTF              PROGBITS
[59] .BTF.base         PROGBITS
[6] FUNC 'process_value' type_id=5 linkage=static
[7] DECL_TAG 'bpf_kfunc' type_id=6 component_idx=-1
```

The private/kfunc core receipt is still blocked only on external target authority:

```json
{
  "receipt_identity_blake3": "c53720b0ca626b75ee36e14a3c694420308b9705204f08773caf38a2a134af84",
  "stage_status": "blocked",
  "target_identity_blake3": null,
  "blockers": [
    {
      "code": "kernel-target-observation-only",
      "subject": "target"
    }
  ]
}
```

## Mantle exact-kernel VM gate

Command (pueue task `33`):

```console
nix build --no-link .#packages.x86_64-linux.kernelscript-production-runtime-check
```

Result: passed.

Relevant `nix log` lines (captured during the final runtime check):

```text
machine: must succeed: test $(uname -r) = 6.18.20
machine: must succeed: /nix/store/.../bpftool prog load .../probe_do_exit.ebpf.o /sys/fs/bpf/mantle-probe type kprobe
machine: must succeed: .../probe_do_exit
machine: must succeed: insmod .../private_kfunc.mod.ko
machine # private_kfunc kfunc module loaded successfully
machine: must succeed: /nix/store/.../bpftool prog load .../private_kfunc.ebpf.o /sys/fs/bpf/mantle-kfunc type xdp
machine: must succeed: test -e /sys/fs/bpf/mantle-kfunc && rm /sys/fs/bpf/mantle-kfunc
machine: must succeed: cd .../artifacts && ./private_kfunc
(finished: run the VM test script, in 13.44 seconds)
test script finished in 13.47s
```

This proves exact-cohort NixOS VM behavior for the selected module/BPF case. It is not a ChaosControl receipt and not target mutation authority.

## OnixOS kernel-bundle authority attempt

Existing OnixOS kernel-bundle tooling was used without editing the downstream repo.

Base manifest command (pueue task `14`):

```console
nix run path:/home/brittonr/git/OnixResearch/onixos#onix -- \
  kernel-bundle inspect /tmp/mantle-kernelscript-onix-base-manifest.json --json
```

Result: passed. Canonical base identity:

```text
kernel_build_identity = onix:blake3:kernel-build:4ee8064c7daf33498bd61d85d573c28b43febf54926bfe1e58ef5df76637e0c2
kbi_id = kbi:sha256:08916e1abc20e438f01391fa9d5e271e094637101a595c3ac28d0678be5f63d0
receipt_identity = onix:blake3:kernel-bundle-receipt:c0002ada52e7bb9045ffb89c940e0cb15742f94b42e68574713e5ec88ccd3c7d
```

Full manifest command (pueue task `18`):

```console
nix run path:/home/brittonr/git/OnixResearch/onixos#onix -- \
  kernel-bundle inspect /tmp/mantle-kernelscript-onix-full-manifest.json --json
```

Result: passed after representing `vermagic` as the checker's accepted exact release string `6.18.20`. Canonical identities:

```text
kernel_build_identity = onix:blake3:kernel-build:4ee8064c7daf33498bd61d85d573c28b43febf54926bfe1e58ef5df76637e0c2
module_pack = onix:blake3:module-pack:b06089102d69299754550d55ea23d40b3235b2be010242a2a62c6de1d3aafcef
bpf_pack = onix:blake3:bpf-pack:e63907102511d66cc006163e9e96e15b0e89e758a6843ab4d235faafc0eebb6a
bundle_identity = onix:blake3:bundle:a669c75d896ca3fefcf8141576ceb1b883c34cf2418ea55d9d53393130a56e82
manifest_identity = onix:blake3:manifest:2b6b1bebbde7b93ca974b81d125be970d472c19f2deb17368cb9e5fd7a012688
receipt_identity = onix:blake3:kernel-bundle-receipt:8f0aec06ffec1316314ce960c6aab88d7dc2dd214f1e9284ab1836518dacb780
```

Projection entries bind the same Mantle bytes:

```text
bpf/private_kfunc.ebpf.o -> mantle://blake3/b8cdd1315b4066c053a14034344a1b051f85fe2c965cffdc38d79d116ebb94de
modules/private_kfunc.mod.ko -> mantle://blake3/1a738476dabe13e3d8ae2c5b0435f7b7f2908a82fadcee136e5494f6a93a81e1
```

This is Onix kernel-bundle shape/identity validation. It is not the active OnixOS BPF runtime adapter, and it does not authorize stage/load/attach/detach/cleanup mutations.

## Remaining downstream blockers

OnixOS active change `cairn/changes/realize-linux-bpf-pack-adapter/` remains unarchived and all tasks are unchecked. Its spec requires the runtime adapter to verify canonical Onix manifest/BPF Pack identity, matching Mantle materialization, matching ChaosControl VM-smoke cohort, current target architecture/kernel/BTF facts, declared capability facts, exact attach target, and pinned loader identity before any BPF load syscall.

ChaosControl active change `cairn/changes/add-kernel-bundle-validation-rail/` remains unarchived and all tasks are unchecked. Repository search found no current kernel-bundle validation implementation beyond the active lifecycle package; the spec requires a dedicated `kernel-bundle/vm-compat-smoke` receipt binding exact Onix/Mantle/ChaosControl identities, boot/module/BPF cases, typed observations, bounds, and cleanup.

Therefore this session produced the Mantle materialization, exact NixOS VM behavior, and an Onix kernel-bundle validation path for the exact bytes, but it did **not** produce the final downstream authority path required to archive Mantle's `add-bounded-kernelscript-experiment` change.

## Focused Mantle regression checks

Focused Rust checks passed after updating the production test assertion from the old blocked runtime marker to the current checked module/runtime markers:

```console
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-focused-20260715 \
  cargo test -p crunch-kernelscript-core -p crunch-kernelscript-adapter
```

Result (pueue task `23`):

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```console
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-focused-20260715 \
  cargo test -p mantle --test kernelscript_experiment
```

Result (pueue task `22`):

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Cairn lifecycle check after the attempt

Command (pueue task `36`):

```console
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- \
  validate --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- \
  gate tasks add-bounded-kernelscript-experiment --root . \
  --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Result: validation passed with `valid: true`, one active change, no issues, and `task_done: 12`, `task_todo: 2`. The tasks gate passed advisory with no issues. The two remaining unchecked tasks are the broad final validation/audit closeout and the archive/sync gate, both intentionally blocked by missing downstream authority.

## Non-claims

This evidence does not claim KernelScript language/compiler soundness, accepted OnixOS runtime-adapter authority, target mutation authority, ChaosControl behavior evidence, kernel safety, eBPF safety, universal compatibility, deployability, production enablement, release eligibility, or physical host readiness.
