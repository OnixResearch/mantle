# Downstream admission progress for private-kfunc cohort

- Date: 2026-07-15
- Question: Did the downstream OnixOS/ChaosControl path accept Mantle's exact private-kfunc cohort with bounded identities?
- Decision: **partial downstream admission exists, but final authority is still blocked**. ChaosControl now has a pure `kernel-bundle/vm-compat-smoke` profile/receipt binding the exact Onix bundle identities and Mantle module/BPF byte identities. OnixOS now has a pure BPF runtime admission receipt binding that ChaosControl receipt, exact target facts, scoped abilities, loader identity, bounds, and non-claims. Neither downstream change is archive-ready: ChaosControl still lacks the dedicated KVM execution rail, and OnixOS still lacks target shell/generation/restart/rollback enforcement.
- Owner: Mantle records this as dependent evidence only; downstream repositories own acceptance and final authority.
- Next action: wait for archived/accepted ChaosControl KVM and OnixOS runtime-adapter evidence before archiving this Mantle change.

## Newly produced downstream identities

```text
chaoscontrol.kernel_bundle_vm_compat_smoke_profile = 216bd1a6c5461209f340a9c4f4d00aacf5c2312679bb9cb5808d329c619fc589
chaoscontrol.kernel_bundle_vm_compat_smoke_receipt = fb37d05d6ee328b05d8f1bdc80ae0d622dcdef590f0dbf7e2721bb3993e76119
onixos.bpf_runtime_admission_receipt = 30e011f64315879f3bd666390229882d0e3311b6c22052d2da85461c60cac39a
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
```

OnixOS:

```text
test pure_domains::bpf_runtime_adapter::tests::exact_mantle_private_kfunc_cohort_is_admitted_with_bounded_plan ... ok
test pure_domains::bpf_runtime_adapter::tests::broad_or_mismatched_authority_is_rejected ... ok
test pure_domains::bpf_runtime_adapter::tests::missing_chaoscontrol_receipt_or_target_drift_denies_before_plan ... ok
```

## Focused Mantle gate

Focused Cairn tasks gate with the canonical generated policy passed during this update:

```text
"verdict": "PASS"
"task_done": 12
"task_todo": 2
```

## Non-claim boundary

Mantle must not reinterpret these active-change pure receipts as final OnixOS target authority or ChaosControl deterministic VMM evidence. This change remains active until downstream owners accept and archive the runtime/KVM rails or provide an equivalent accepted receipt.
