# Downstream authority audit — 2026-07-15

## Decision

`add-bounded-kernelscript-experiment` remains **not archive-ready**. A fresh
read-only audit still found no accepted downstream authority that binds Mantle's
pinned KernelScript `v0.1.2` private-kfunc fixture to an OnixOS target and no
replayable ChaosControl behavior receipt for the same identity cohort.

This audit intentionally did not modify OnixOS or ChaosControl. Both sibling
checkouts contain existing untracked or modified lifecycle work, so those trees
were treated as user-owned read-only inputs for this session.

## Completion contract checked

The change can close only when all of these are present for the same cohort:

1. An accepted OnixOS manifest or receipt binding the exact kernel build,
   architecture, release, config, headers, BTF, toolchain, KernelScript revision,
   generated source, module, and eBPF object identities.
2. A concrete private-kfunc ABI and capability contract covering module member
   identity, vermagic, exported signature, BTF/type requirements, XDP section,
   attach target, loader, and mutation capabilities.
3. An implemented and accepted OnixOS consumer/target ownership assignment for
   this exact case.
4. A replayable ChaosControl `kernel-bundle/vm-compat-smoke` receipt proving
   module load/observe/unload and BPF verify/attach/trigger/observe/detach/cleanup
   for the same identities.

Synthetic Mantle fixtures, structural KernelScript receipts, public-only OnixOS
examples, active unarchived downstream specs, and process success without those
identity links are false-completion cases.

Update note: later on 2026-07-15, `downstream-authority-attempt-2026-07-15.md`
advanced the Mantle-local module/BPF materialization and OnixOS structural
identity validation. This audit's downstream completion decision remains valid
because the accepted OnixOS runtime-adapter authority and ChaosControl behavior
receipt are still absent.

## Approach-family registry

| Family | Mechanism inspected | Evidence | State |
|---|---|---|---|
| OnixOS target authority | Accepted `onix-kernel-bundle-v1` and active `realize-linux-bpf-pack-adapter` lifecycle package | `cairn/specs/kernel-bundles/spec.md` accepts structural ModulePack/BPF Pack binding, but `docs/kernel-bundles.md` explicitly excludes deep kfunc-signature/eBPF safety verification. The active `cairn/changes/realize-linux-bpf-pack-adapter/tasks.md` still has all implementation, target shell, authority, lifecycle, evidence, and closeout tasks unchecked. | blocked |
| ChaosControl runtime receipt | Active `add-kernel-bundle-validation-rail` lifecycle package | `cairn/changes/add-kernel-bundle-validation-rail/tasks.md` still has profile/admission, guest harness, module, BPF, KVM behavior rail, receipt, and closeout tasks unchecked. No accepted behavior receipt exists for Mantle's private-kfunc fixture. | blocked |
| Mantle local KernelScript evidence | Existing production probe/private-kfunc core receipts plus later same-day materialization attempt | `probe-production-evidence.md` and `downstream-authority-attempt-2026-07-15.md` now record `target_identity_blake3: null` and `kernel-target-observation-only` for both routes. The later attempt removed the prior Mantle-local `module-build-and-vm-gate-absent` blocker by building/loading the module and XDP object in the exact-kernel NixOS VM smoke, but this still does not establish accepted OnixOS runtime-adapter authority or a ChaosControl receipt. | blocked |

## Inspected checkout identities

```text
mantle
04b7e2bc946e
## main...origin/main [ahead 93]
?? examples/cowsay.ncl

onixos
a63dd601b6d3
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

chaoscontrol
a00377e62843
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

## Downstream evidence observations

### OnixOS

The accepted kernel-bundle contract provides shape and identity roles for
ModulePacks and BPF Packs. It does not establish the missing private-kfunc ABI or
runtime authority. The active runtime-adapter change states the desired future
boundary but is unimplemented:

- `cairn/changes/realize-linux-bpf-pack-adapter/tasks.md` lines 3-8 require
  exact Mantle and ChaosControl fixtures, target UCAN enforcement, typed machine
  intent, pure admission, and positive/negative planning fixtures; all remain
  unchecked.
- Lines 12-16 require a target shell, least-privilege profiles, fresh local
  rehashing, target-fact observation, authority gating, and negative shell
  fixtures; all remain unchecked.
- Lines 20-34 require lifecycle, replacement, rollback, restart, observation,
  receipts, docs, validation, and archive evidence; all remain unchecked.

The generic golden fixture still uses public `bpf_task_acquire` style capability
evidence, not Mantle's `private_kfunc` / `process_value(u32) -> u32` module case.

### ChaosControl

The active kernel-bundle validation package names the right future behavior
receipt role, but it is not implemented or archived:

- `cairn/changes/add-kernel-bundle-validation-rail/tasks.md` lines 3-7 require
  Onix/Mantle fixture admission and typed campaign/profile logic; all remain
  unchecked.
- Lines 10-19 require guest protocol, boot control, ModulePack execution, BPF
  execution, and positive/negative behavior fixtures; all remain unchecked.
- Lines 23-32 require `kernel-bundle/vm-compat-smoke` receipts, distinct wiring
  and KVM rails, non-claim guards, docs, validation, and archive evidence; all
  remain unchecked.

The sibling `harden-ebpf-trace-evidence` package is about ChaosControl's host
KVM trace collector. Its proposal and design explicitly separate that work from
imported BPF Pack behavior validation, so it cannot satisfy this KernelScript
change.

## Adversarial audit

- Treating Mantle's exact-kernel probe VM pass as authority would overclaim: it
  only covers the probe object/loader observation and not the private-kfunc
  module route, target ownership, or ChaosControl receipt.
- Treating OnixOS structural pack validation as authority would overclaim: the
  accepted docs exclude deep BTF, CO-RE, kfunc-signature, eBPF safety, and target
  runtime verification.
- Treating active downstream lifecycle packages as authority would overclaim:
  both are unarchived and their implementation/receipt tasks remain unchecked.
- Editing user-owned untracked downstream packages to synthesize minimal prose
  would not produce the required target manifest, ABI, authority, or behavior
  receipt and would risk overwriting unrelated work.

## Required next artifacts

The smallest authority-producing path is still downstream-first:

1. In OnixOS, finish or narrow `realize-linux-bpf-pack-adapter` for one exact
   supported cohort, including target authority, exact BPF Pack admission,
   local-byte rehashing, least-privilege shell boundaries, and receipts.
2. In ChaosControl, finish or narrow `add-kernel-bundle-validation-rail` for one
   exact boot/module/BPF case, including a replayable
   `kernel-bundle/vm-compat-smoke` receipt for Mantle's private-kfunc fixture.
3. Back in Mantle, replace the null `target_identity_blake3` with the accepted
   downstream cohort identity, build/inspect the module/test classes, rerun the
   KernelScript positive/negative rails, then run Cairn sync/archive.

Until those artifacts exist, this Mantle change must remain active and the
remaining tasks must stay unchecked.

## Non-claims

This audit does not claim KernelScript language/compiler soundness, private-kfunc
ABI validity, OnixOS target authorization, ChaosControl runtime behavior,
eBPF verifier acceptance, module load/unload success, deployability, production
readiness, release eligibility, or that untracked sibling work is defective. It
only records that the required accepted downstream evidence is absent from the
inspected checkouts in this session.
