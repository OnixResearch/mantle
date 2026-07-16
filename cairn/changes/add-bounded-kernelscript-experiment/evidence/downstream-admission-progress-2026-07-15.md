# Downstream admission progress for private-kfunc cohort

- Date: 2026-07-15
- Question: Did the downstream OnixOS/ChaosControl path accept Mantle's exact private-kfunc cohort with bounded identities?
- Current decision: **substantial downstream implementation exists, but final authority remains blocked**. ChaosControl's exact positive/negative KVM rail is accepted under `cairn/archive/2026-07-16-add-kernel-bundle-validation-rail/` at commit `c6d8ec9`. OnixOS now has typed intent, real per-operation UCAN checks, a capability-rooted target shell, generation lifecycle/reconciliation, system wiring, degraded target receipts, and exact `BpfCleanup`-authorized failed-load compensation. Its exact Linux 6.18.20 run reaches authorized private-kfunc module load but cannot resolve module BTF for BPF load under the required service profile without `CAP_SYS_ADMIN`. The same object loads when that forbidden capability is added, so the result is a bounded capability blocker, not target authority.
- Owner: Mantle records this as dependent evidence only; downstream repositories own acceptance and final authority.
- Next action: keep this Mantle change active while OnixOS implements a reviewed non-init-user-namespace BPF token or equivalent narrow broker and target-local credential injection, then reruns successful activation/restart/replacement/rollback/cleanup without `CAP_SYS_ADMIN`.

## Newly produced downstream identities

```text
chaoscontrol.kernel_bundle_vm_compat_smoke_profile = 216bd1a6c5461209f340a9c4f4d00aacf5c2312679bb9cb5808d329c619fc589
chaoscontrol.kernel_bundle_vm_compat_smoke_receipt = fb37d05d6ee328b05d8f1bdc80ae0d622dcdef590f0dbf7e2721bb3993e76119
chaoscontrol.kvm_marker_transcript_rejected_receipt = 9f1608fa741701a8595edd535f41c0ca0341fbfd9668f4276e5baffba04247a8
chaoscontrol.kvm_blocked_input_receipt = 772571012d9cf77c371fdefd55c53c1529ee197814f07fe9df1188d298e6c717
chaoscontrol.exact_kvm_receipt = 40f624ff0ff51e46bbab3813a4122ff5329be9019c1a4d73f44d11cb242daae8
chaoscontrol.exact_kvm_kernel_image_blake3 = 223a6b61393b8956124a574d0fac00057fc45171dd7bb56a7711ca1a224de5d7
chaoscontrol.exact_kvm_initrd_image_blake3 = 48bd470f32f96bc26d3d2599f1ab0dba4b3c2dac6eab658bcbce382e21d8c9e8
onixos.historical_bpf_runtime_admission_receipt = 30e011f64315879f3bd666390229882d0e3311b6c22052d2da85461c60cac39a
onixos.historical_capability_boundary_receipt = 2f2e23413707fd79ca60932019a4eb96743b870d9e622322483e55b577eece21
onixos.compensated_capability_boundary_receipt = b5824e26aaceccbbefe71dab05d8806e920d26476cee91ddadee981e4ec38956
```

These identities bind the already-recorded Mantle cohort:

```text
onix.kernel_build_identity = onix:blake3:kernel-build:4ee8064c7daf33498bd61d85d573c28b43febf54926bfe1e58ef5df76637e0c2
onix.module_pack_identity = onix:blake3:module-pack:b06089102d69299754550d55ea23d40b3235b2be010242a2a62c6de1d3aafcef
onix.bpf_pack_identity = onix:blake3:bpf-pack:e63907102511d66cc006163e9e96e15b0e89e758a6843ab4d235faafc0eebb6a
mantle.module_blake3 = 1a738476dabe13e3d8ae2c5b0435f7b7f2908a82fadcee136e5494f6a93a81e1
mantle.bpf_object_blake3 = b8cdd1315b4066c053a14034344a1b051f85fe2c965cffdc38d79d116ebb94de
```

## Focused downstream validation evidence

ChaosControl:

```text
test kernel_bundle_validation::tests::exact_mantle_private_kfunc_profile_emits_scoped_receipt ... ok
test kernel_bundle_validation::tests::stale_or_role_confused_inputs_fail_before_receipt ... ok
test kernel_bundle_validation::tests::cleanup_and_non_claim_gaps_cannot_pass ... ok
test kernel_bundle_validation::tests::raw_log_or_missing_cleanup_cannot_pass_kvm_rail ... ok
test kernel_bundle_validation::tests::unavailable_kvm_is_blocked_not_passed ... ok
test kernel_bundle_validation::tests::kvm_markers_emit_passed_rail_receipt ... ok
```

ChaosControl local follow-up records a repo-owned initrd/loader path in addition to commit `95afb9d`'s KVM shell. The persisted exact receipt `40f624ff0ff51e46bbab3813a4122ff5329be9019c1a4d73f44d11cb242daae8` has `execution_mode = chaoscontrol-vmm-kvm`, binds kernel image `223a6b61393b8956124a574d0fac00057fc45171dd7bb56a7711ca1a224de5d7` and initrd image `48bd470f32f96bc26d3d2599f1ab0dba4b3c2dac6eab658bcbce382e21d8c9e8`, and records boot/module/BPF verify/attach/detach/cleanup observations with no issues. The marker transcript receipt is explicitly rejected as failed exact-KVM evidence, and the blocked-input receipt proves fail-closed unavailable-loader behavior.

OnixOS focused suites now cover admission, lifecycle, target shell, real UCAN, target agent, and exact dotted-name cleanup. The exact capability-boundary VM rerun also passed its expected-negative and automatic-compensation assertions:

```text
private_kfunc kfunc module loaded successfully
onix-bpf-runtime: operation=LoadBpf class=target-effect-failed ... process_value ... not found in kernel or module BTFs
"status": "degraded"
"kind": "load-bpf"
"succeeded": false
CAP_BPF present; CAP_SYS_ADMIN absent
"kind": "cleanup"
"succeeded": true
"cleanup_class": "succeeded"
private_kfunc.mod absent from /proc/modules
test script finished in 5.39s
```

OnixOS evidence: `cairn/changes/realize-linux-bpf-pack-adapter/evidence/exact-kernel-capability-boundary-2026-07-16.md` in the sibling repository.

ChaosControl closeout evidence is archived under `cairn/archive/2026-07-16-add-kernel-bundle-validation-rail/` in the sibling repository. Canonical-policy post-archive validation passed there; commits `0dbb17b` and `c6d8ec9` remain local and were not pushed.

## Focused Mantle gate

Focused Cairn tasks gate with the canonical generated policy passed during this update:

```text
"verdict": "PASS"
"task_done": 12
"task_todo": 2
```

## Non-claim boundary

Mantle must not reinterpret these active-change receipts, the ChaosControl transcript receipt, the exact positive KVM receipt, or OnixOS receipt `b5824e26aaceccbbefe71dab05d8806e920d26476cee91ddadee981e4ec38956` as final target authority. The OnixOS receipt is explicitly degraded and proves fail-closed behavior plus authorized compensation at the least-privilege kernel boundary. Its temporary VM credential fixture also passed through Nix test inputs, so it does not satisfy the production target-local secret boundary. This Mantle change remains active; no sync or archive is justified.
