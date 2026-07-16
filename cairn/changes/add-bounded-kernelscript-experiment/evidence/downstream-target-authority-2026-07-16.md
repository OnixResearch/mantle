# Exact downstream target authority for the private-kfunc cohort

- Date: 2026-07-16
- Question: Did separate OnixOS and ChaosControl gates accept the exact Mantle private-kfunc candidate bytes without widening Mantle's own authority or promoting KernelScript beyond the bounded experiment?
- Inspected evidence: Mantle's pinned production materialization, archived ChaosControl exact KVM receipt, OnixOS target-provisioned lifecycle receipt index, credential-closure scan, three consecutive exact VM runs, service capability assertions, focused implementation checks, and canonical Cairn gate receipts.
- Decision: **yes for the exact Linux 6.18.20 KVM target cohort**. ChaosControl accepted the exact positive/negative runtime cohort. OnixOS then admitted and operated the same Mantle bytes through generation-scoped UCAN authority and a narrow BPF-token broker, while the lifecycle service lacked `CAP_SYS_ADMIN` and private credentials remained target-local rather than Nickel/Nix inputs. This closes the external-authority dependency for Mantle's bounded experiment; it does not turn KernelScript artifacts into default, production, release-eligible, physical-host-ready, or generally compatible outputs.
- Owner: Mantle owns only the pinned source-to-candidate materialization and this evidence linkage. ChaosControl owns its exact KVM receipt. OnixOS owns semantic admission and target lifecycle authority.
- Next action: complete Mantle's focused quality/dependency checks and canonical Cairn gates, then sync/archive this experiment without changing its non-default candidate status. OnixOS separately retains an active change only because canonical Cairn currently blocks sync/archive on unrelated legacy accepted-spec identity debt.

## Exact linked cohort

```text
mantle.artifacts = /nix/store/6cz7sqcq3mp7vnpz2nipcjx600b15fxv-mantle-kernelscript-production-artifacts
kernel.release = 6.18.20
kernel.build_identity = onix:blake3:kernel-build:4ee8064c7daf33498bd61d85d573c28b43febf54926bfe1e58ef5df76637e0c2
kernel.image_blake3 = 223a6b61393b8956124a574d0fac00057fc45171dd7bb56a7711ca1a224de5d7
kernel.base_btf_blake3 = 08c14844b3b9cde37f8b3bba9effde104fa59194195777f3dd859db3919e8064
mantle.module_blake3 = 1a738476dabe13e3d8ae2c5b0435f7b7f2908a82fadcee136e5494f6a93a81e1
mantle.bpf_object_blake3 = b8cdd1315b4066c053a14034344a1b051f85fe2c965cffdc38d79d116ebb94de
mantle.loader_blake3 = 87e46ac46caa731848d339c02bbeea7a3a667bc4f8a63bf9152c9f8ae01ad602
onix.module_pack_identity = onix:blake3:module-pack:b06089102d69299754550d55ea23d40b3235b2be010242a2a62c6de1d3aafcef
onix.bpf_pack_identity = onix:blake3:bpf-pack:e63907102511d66cc006163e9e96e15b0e89e758a6843ab4d235faafc0eebb6a
chaoscontrol.exact_kvm_receipt_blake3 = 40f624ff0ff51e46bbab3813a4122ff5329be9019c1a4d73f44d11cb242daae8
```

The ChaosControl receipt is archived under
`cairn/archive/2026-07-16-add-kernel-bundle-validation-rail/` in the sibling
repository. It binds exact kernel/initrd identities plus boot, module, BPF
verify/attach/detach, and cleanup observations. The marker-only transcript and
unavailable-loader cohort remain explicit negative evidence rather than being
promoted as exact runtime proof.

## OnixOS target lifecycle evidence

OnixOS commit `fc6eb30` records the implementation and evidence. The exact
target-provisioned driver is:

```text
/nix/store/nd1s0pz7h126xzspfwcpdq40p302kmmy-nixos-test-driver-onixos-exact-bpf-runtime-target-provisioned-lifecycle
/nix/store/2xm7r9wg73lc8i5h8c7jayx5lmb9vaqp-nixos-test-driver-onixos-exact-bpf-runtime-target-provisioned-lifecycle.drv
fixture_blake3 = 48b219b6fbb99b403c8efec787a1a9fb98994d9f38aaebc181447d510b86e584
authority_inventory_blake3 = 45fb0517cf42553f8f90219ed809b8bda960f126f87b9c9ef5d0eb6cddc6c24c
test_script_blake3 = e861845a679098607f583f1c43bbe711406dfd6cbf9f218504e8a543cc7a915f
consecutive_successful_runs = 3
```

Target reports bind the following lifecycle outcomes:

| Phase | Outcome | Lifecycle receipt BLAKE3 |
|---|---|---|
| generation-0001 activation | succeeded | `2fd49d279a77c6e7dac5e4fd5c6ada633dcfe0a9600c0ca018e398c8ad24d5cd` |
| generation-0002 replacement with stage-only credential | degraded, `ucan-authorization-denied` before BPF load | `3dfb03c2c8f5312d12e19130e5e300a82c62649d10819c74c679d3fef317cb3b` |
| replacement with generation-0002 | succeeded with declared observation gap | `069ae5182117ab3f4d1ceefef849d6264d17af4a71316ec313cec5b2bb6c0ddb` |
| rollback to retained generation-0001 | succeeded with declared observation gap | `b917b3785459e81435ee713884c57fb9b29a0af27e5914121f6e523b7cefec23` |
| recovery from removed-pin drift | succeeded | `ba5fc46852138f8553d16c38dc55893664d8147cce048788c91acf269692d992` |
| generation-0002 cleanup | succeeded; module absent and foreign map preserved | `a72623165e3dfa3df9d3cc8d4e0a72e56c4b4c30ef5c4ab32ed7c48729b2b05a` |

The same run also proved mutation-free restart reconciliation and fail-closed
missing-pin identity refusal. Exact live program reconciliation compares `id`,
`type`, `name`, `tag`, and `bytes_xlated`.

## Least-privilege and credential boundary

```text
lifecycle capabilities = CAP_BPF, CAP_NET_ADMIN, CAP_SYS_MODULE
lifecycle CAP_SYS_ADMIN = absent
broker capabilities = CAP_SYS_ADMIN only
broker lifecycle capabilities = absent
token commands = prog_load:btf_load:btf_get_fd_by_id
token program type = XDP
token expected attach type = XDP
```

The socket-activated broker authenticates the exact runtime UID and returns one
bounded protocol response with exactly one `SCM_RIGHTS` descriptor plus the
exact nonzero module-BTF ID. The runtime independently validates the returned
bpffs mount and exact token delegation before loading the first and only program
in the selected object.

The fixture builds its driver before credentials exist in target state. It then
copies the private credential and holder key after VM boot, requires target
owner `onix-bpf-runtime` and mode `0600`, and never imports those files into
Nickel or Nix. The bounded closure scan compared both private files against
69,987 regular closure files and passed. This is evidence against Nix path
capture, not a general host/hypervisor secrecy claim.

## OnixOS gate status and bounded archive blocker

OnixOS proposal, design, and tasks gates all returned `PASS`; its tasks gate
recorded 23 done and zero todo. Canonical validation reported no change,
cross-repository, or substance issue for the BPF package. Cairn nevertheless
blocked OnixOS sync/archive because unrelated accepted legacy specs elsewhere
in that repository lack canonical requirement IDs and substantive blocks. No
manual bypass was attempted.

Mantle treats the committed gate and exact target receipts as external evidence;
it does not claim that the OnixOS accepted spec was synced or archived. That
repository-wide lifecycle debt does not alter the observed target result or
expand Mantle's authority.

## Preserved Mantle claim boundary

Mantle's build-time core receipts intentionally remain
`kernel-target-observation-only`: the build tool does not mutate a kernel and
does not rewrite historical materialization receipts after external execution.
This closeout links separate exact external receipts to the candidate cohort.
It does not promote the beta profile into default packages/releases, remove its
`experimental-unverified` build-time label, claim language/compiler soundness,
prove Linux/libbpf/BPF safety, establish compatibility outside Linux 6.18.20,
or establish physical-host or release readiness.
