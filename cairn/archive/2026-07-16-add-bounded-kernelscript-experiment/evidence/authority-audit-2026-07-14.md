# Downstream authority audit — 2026-07-14

## Decision

`add-bounded-kernelscript-experiment` is **not archive-ready**. Keep the
change active. A read-only audit of the current OnixOS and ChaosControl
checkouts found no accepted authority binding Mantle's pinned KernelScript
input to the proposed private-kfunc target and no replayable runtime receipt.

## Findings

- Mantle records KernelScript `v0.1.2`, revision
  `0c80d4e…`, and archive hashes, but
  `evidence/probe-production-evidence.md` still records a null
  `target_identity_blake3` and the state
  `module-build-and-vm-gate-absent`.
- OnixOS defines generic kfunc, BTF, and type fields in
  `lib/kernel-bundle.ncl`, but `docs/kernel-bundles.md` bounds those facts to
  structural evidence rather than deep kfunc-signature evidence. No accepted
  contract binds `private_kfunc`, `process_value(u32) -> u32`, the module ABI,
  XDP section or attach target, and required target capabilities.
- OnixOS documents generic ownership, but its active
  `cairn/changes/realize-linux-bpf-pack-adapter/` change says no target
  component currently exists and its implementation/authority tasks remain
  unchecked.
- ChaosControl's active
  `cairn/changes/add-kernel-bundle-validation-rail/` remains
  specification-only. Its module/BPF execution and receipt tasks are
  unchecked, and no corresponding evidence directory exists.
- The OnixOS golden fixture is synthetic (`6.12.0-onix`, `storage.ko`,
  `trace.o`, placeholder digests, and public `bpf_task_acquire` requirements),
  so it is not authority for Mantle's private-kfunc case.

## Exact evidence still required

1. An accepted Onix manifest or receipt binding exact kernel build,
   architecture, release, config, headers, BTF, toolchain, KernelScript
   revision, generated source, module, and eBPF object identities.
2. A concrete private-kfunc ABI and capability contract covering the module
   and member identity, vermagic, exported signature, BTF/type requirements,
   section, attach class and target, loader, and mutation capabilities.
3. An implemented and accepted consumer assignment for this exact case.
4. A replayable ChaosControl receipt proving module
   load/observe/unload and BPF
   verify/attach/trigger/observe/detach/cleanup for the same identity cohort.

## Search scope

The audit read the complete Mantle change package and evidence, then searched:

- OnixOS checkout `a63dd601…`: `cairn/specs/kernel-bundles/`, archived
  kernel-bundle work, `cairn/evidence/`, `lib/kernel-bundle.ncl`,
  `docs/kernel-bundles.md`, CLI/tests, and the active runtime-adapter change.
- ChaosControl checkout `a00377e6…`: `crates/`, `contracts/`, `docs/`,
  `dogfood-results/`, `audits/`, and the active validation-rail change.

No match linked Mantle's pinned KernelScript revision or
`private_kfunc`/`process_value` case to sibling authority. This audit does not
assess evidence absent from the inspected checkouts and does not promote
untracked sibling scaffolds to accepted lifecycle authority.
