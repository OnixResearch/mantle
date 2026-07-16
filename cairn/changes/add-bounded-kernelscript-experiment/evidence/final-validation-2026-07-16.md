# Final bounded KernelScript validation

- Date: 2026-07-16
- Question: Do the focused Mantle materialization, positive/negative tests, first-party quality checks, dependency policy, exact runtime handoff, and lifecycle package satisfy the bounded experiment closeout without promoting KernelScript artifacts?
- Inspected evidence: current focused Rust/Nix checks, package-scoped cargo-deny results, exact Mantle artifact identities, archived ChaosControl runtime receipt, committed OnixOS target lifecycle receipts, and the active Cairn change package.
- Decision: **focused implementation and external-evidence validation pass**. The experiment remains non-default and its build-time outputs remain candidate evidence; closeout links exact external receipts rather than mutating Mantle's historical build receipts or granting Mantle kernel authority.
- Owner: Mantle KernelScript experiment maintainers.
- Next action: commit the complete implementation/evidence packet, then run canonical Cairn sync/archive plans and execute only if they remain unblocked.

## Focused Rust and shape checks

```text
pueue 249: crunch-kernelscript-core — 20 passed, 0 failed
pueue 253: crunch-kernelscript-adapter --lib — 9 passed, 0 failed
pueue 254: mantle kernelscript_experiment — 6 passed, 0 failed
pueue 216: crunch-kernelscript-core wasm32-unknown-unknown core+alloc check — PASS
pueue 217: crunch-kernelscript-core + adapter --all-targets --no-deps Clippy -D warnings — PASS
pueue 256: mantle kernelscript_experiment --no-deps Clippy -D warnings — PASS
pueue 248: package-scoped rustfmt plus leaf integration rustfmt — PASS
```

The suites include positive exact profile/generated-shape/receipt parity and
negative missing/extra shape, hard bounds, source digest drift, symlink,
descriptor replacement, default enablement, unknown production field, external
authority-blocker, and generated-Makefile non-execution cases.

## Focused dependency policy

```text
pueue 243: cargo-deny for crunch-kernelscript-core
  advisories ok, bans ok, licenses ok, sources ok
pueue 244: cargo-deny for crunch-kernelscript-adapter
  advisories ok, bans ok, licenses ok, sources ok
```

A whole-workspace audit was also attempted and is not claimed as passing. It is
blocked by unrelated current workspace debt:

```text
source: nickel-export-core git source not allowlisted
license: winx 0.36.4 Apache-2.0 WITH LLVM-exception not allowlisted
RUSTSEC-2026-0190: anyhow 1.0.102
RUSTSEC-2026-0204: crossbeam-epoch 0.9.18
RUSTSEC-2023-0056: vm-memory 0.10.0
RUSTSEC-2024-0002: vmm-sys-util 0.11.2
```

The two KernelScript package-root graphs pass all configured cargo-deny classes.
The broad failures are outside the KernelScript core/adapter graph and are not
silently reclassified as a pass.

## Current production rails

```text
pueue 219: checks.x86_64-linux.kernelscript-production — PASS
pueue 219: packages.x86_64-linux.kernelscript-production-runtime-check — PASS
production artifacts = /nix/store/6cz7sqcq3mp7vnpz2nipcjx600b15fxv-mantle-kernelscript-production-artifacts
```

The structural rail binds the pinned compiler/source cohort, exact generated
members, explicit Mantle-owned build plans, static ELF/BTF/module inspection,
and build-time non-claims. The exact runtime rail requires Linux 6.18.20 and
loads/verifies the named probe/private-kfunc cohort in its dedicated NixOS VM.

## Separate target authority

`downstream-target-authority-2026-07-16.md` links the same Mantle module/BPF/
loader bytes to:

- archived ChaosControl exact KVM receipt
  `40f624ff0ff51e46bbab3813a4122ff5329be9019c1a4d73f44d11cb242daae8`;
- OnixOS target-provisioned activation, denied-UCAN replacement, successful
  replacement, rollback, mutation-free reconciliation, missing-pin refusal,
  recovery, foreign-resource-preserving cleanup, and module unload receipts;
- three consecutive complete lifecycle runs;
- a 69,987-file Nix-closure scan showing the target credential/key files were
  not captured as standalone closure files;
- lifecycle capabilities exactly `CAP_BPF`, `CAP_NET_ADMIN`, and
  `CAP_SYS_MODULE`, with `CAP_SYS_ADMIN` isolated to the narrow broker.

OnixOS implementation/evidence is committed at `fc6eb30`. Its own change gates
pass with 23 done and zero todo; sync/archive remains blocked only by unrelated
legacy accepted-spec identity debt in that repository. Mantle does not claim an
OnixOS archive that did not occur.

## Canonical Cairn gates before implementation commit

Canonical policy:
`/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`.

```text
proposal = PASS
proposal receipt = a5ac7a700bc042d2f1864d53afb0025b854b30cbbe09938e9cc87f0d6f25f9bb
design = PASS
design receipt = 35002cd1d633c62e43a91d4ea196e1b5cc378b4ce18a13c4f4159afdefec9eb6
tasks = PASS (13 done, 1 closeout todo)
tasks receipt = 1a06bf2c0f59be6bf09208bc7c5dd040fff1e2c32a2e5b1ddcbcefa1ca56933f
validate = true
change issues = 0
cross-repository evidence issues = 0
substance issues = 0
```

The remaining task is the mutation step itself. No manual accepted-spec or
archive-directory edit is permitted; Cairn must produce and execute an
unblocked deterministic plan.

## Non-claims

This checkpoint does not claim KernelScript language/compiler soundness, Linux
or libbpf correctness, BPF safety, compatibility outside the exact Linux
6.18.20 cohort, physical-host readiness, default enablement, production support,
or release eligibility. It does not convert Mantle into a runtime authority and
does not rewrite build-time `kernel-target-observation-only` receipts after the
fact.
