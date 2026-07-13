# Pinned KernelScript probe observation evidence

- Date: 2026-07-12
- Branch: `agent/kernelscript-production`
- Implementation lineage: pinned build slice begins at `c7745316`; this evidence
  is bound to the repository commit that contains this file and the
  `mantle-kernelscript-probe-observation-v1` claim-boundary correction.
- Question: Can the reviewed KernelScript `v0.1.2` probe be generated, built,
  structurally inspected, verifier-loaded, attached, and detached on one exact
  locked Linux cohort without ambient opam/compiler/BTF state or execution of
  generated build scripts?
- Inspected evidence: locked Nix derivations and generated observation below;
  exact-kernel NixOS VM log for bpftool load plus generated loader execution.
- Decision: **yes for the probe on this exact cohort only**. This is an
  observation, not a `crunch-kernelscript-core` admission receipt. Reported
  OnixOS Git identifiers are not accepted authority because their bytes are not
  materialized by this route.
- Owner: Mantle KernelScript experiment maintainers.
- Next action: replace duplicate shell classification/record rendering with a
  thin adapter over `crunch-kernelscript-core`; separately add checked Nix
  module construction and exact-kernel module/kfunc VM gates.

## Materialized cohort

The locked Nix authority is:

- nixpkgs revision:
  `6201e203d09599479a3b3450ed24fa81537ebc4e`;
- nixpkgs NAR hash:
  `sha256-ZojAnPuCdy657PbTq5V0Y+AHKhZAIwSIT2cb8UgAz/U=`;
- target: `x86_64-linux`, Linux `6.18.20`;
- compiler source revision:
  `0c80d4e4ac0029d34cbc9d65e76d78c075b64555`;
- compiler source archive SHA-256:
  `9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1`;
- compiler source archive BLAKE3:
  `439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57`.

Reported OnixOS commit/tree/blob strings in the observation are metadata only.
They were not materialized or remeasured by the checked derivation and are not
called accepted authority here.

## Build and structural observation

Command:

```console
nix build --no-link --print-out-paths \
  .#checks.x86_64-linux.kernelscript-production
```

Result: success.

```text
/nix/store/95fa5szaaq410kl0mxkly63w6jmsar0y-mantle-kernelscript-production-structural-check
```

The corresponding artifact output was:

```text
/nix/store/bfbrggyqq5pggscyi8syr2i2f5rxfmif-mantle-kernelscript-production-artifacts
```

Its `evidence/probe-observation.json` has BLAKE3:

```text
9d6d74ffceb2804ae9ff693f502b309898a6472843ed919667f5e4d38c0089fe
```

The observation records:

- compiler binary BLAKE3
  `0166b28f63171252a01b5f955028375231f7659eb2289b82eb89d5d92cc7faa2`;
- compiler closure store-path-set BLAKE3
  `4d940b485c3683153fa732a68abd69b9e03e17c1ced583bd906638ec1e44e047`;
- kernel image BLAKE3
  `848998e5b72b01114de5e291d2621342938d36e6936aa6e2da3fc0239f7d486e`;
- target config BLAKE3
  `af33e9e9a159b3025e6c7d158ced26ee677564c7176dfb20aec896d3dbd3b6ed`;
- target BTF BLAKE3
  `223a6b61393b8956124a574d0fac00057fc45171dd7bb56a7711ca1a224de5d7`;
- probe eBPF object BLAKE3
  `3721df3d155633bc8671905528da7701f7864b8276951c0793bdf7f020314f4c`;
- probe loader BLAKE3
  `fb189b35b2704c966223f0f301562a611fdca35829f975f42afcd9ba5c1a03e6`.

The structural derivation checked ELF headers, sections, relocations, and BTF.
The generated `Makefile` and `Kbuild` were retained but not executed.

## Exact-kernel VM observation

Command:

```console
nix build --no-link --print-out-paths \
  .#packages.x86_64-linux.kernelscript-production-runtime-check
```

Result: success.

```text
/nix/store/pq700fydagyyjgnb82m2522pspm8yshc-vm-test-run-mantle-kernelscript-production-runtime
```

Relevant current Nix test log:

```text
machine: must succeed: test -e /sys/fs/bpf/mantle-probe && rm /sys/fs/bpf/mantle-probe
machine: (finished: must succeed: test -e /sys/fs/bpf/mantle-probe && rm /sys/fs/bpf/mantle-probe, in 0.01 seconds)
machine: must succeed: /nix/store/bfbrggyqq5pggscyi8syr2i2f5rxfmif-mantle-kernelscript-production-artifacts/artifacts/probe_do_exit
machine: (finished: must succeed: /nix/store/bfbrggyqq5pggscyi8syr2i2f5rxfmif-mantle-kernelscript-production-artifacts/artifacts/probe_do_exit, in 0.12 seconds)
(finished: run the VM test script, in 13.83 seconds)
test script finished in 13.86s
```

The checked test script also asserts `uname -r == 6.18.20`, uses bpftool to load
the object as a kprobe program, verifies the pinned program exists, removes it,
and requires the generated loader output to contain successful `do_exit`
attach and detach markers.

## Blockers and non-claims

The following remain blockers and their Cairn tasks stay unchecked:

1. `productionShell` performs duplicate exact-file classification/accounting
   and observation rendering rather than invoking `crunch-kernelscript-core`.
2. The checked Nix route does not build a module, and the checked VM route does
   not load/unload a module or validate private-kfunc/XDP behavior.

This evidence does not claim language or compiler soundness, kernel safety,
cross-kernel compatibility, production readiness, default enablement, accepted
Onix semantics, ChaosControl validation, release eligibility, or module/kfunc
success.
