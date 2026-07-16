## Why

KernelScript offers a promising single-source path to generated userspace C, eBPF C, and optional kernel-module kfunc code, but upstream explicitly labels the language beta and provides no backward-compatibility guarantee. Onix should learn from it without making an unstable compiler or privileged artifact path part of the default stack.

Mantle is the right place to run a pinned compiler and explicit downstream toolchain in a bounded sandbox. The experiment must retain generated sources and exact inputs, avoid executing generated Makefiles as authority, and hand any module/BPF candidates to OnixOS and ChaosControl for separate compatibility and runtime validation.

## What Changes

- Add a non-default typed Nickel experiment profile binding KernelScript source, compiler revision, toolchain, BTF/headers, target kernel identity, output classes, bounds, and non-claims. r[kernelscript_experiment.profile]
- Materialize a pinned KernelScript compiler closure and run deterministic code generation in a network-disabled Mantle build. r[kernelscript_experiment.compiler] r[kernelscript_experiment.codegen]
- Parse the generated project into an explicit Mantle-owned build plan instead of executing the generated Makefile, then build selected userspace, eBPF, and optional module artifacts. r[kernelscript_experiment.artifacts]
- Bind BTF, kernel headers/config, architecture, and kernel-build identities to module/BPF candidates and reject missing or drifting kernel inputs. r[kernelscript_experiment.kernel_inputs]
- Emit candidate ModulePack/BPF Pack projections only after byte and manifest validation, without declaring them deployable. r[kernelscript_experiment.handoff]
- Add deterministic receipts, positive/negative fixtures, and a documented beta/non-production evidence gate. r[kernelscript_experiment.evidence] r[kernelscript_experiment.verification]

## Impact

- **Scope**: a dedicated experimental package/profile and fixtures; no default build, release, or OnixOS profile consumes KernelScript.
- **Toolchain**: OCaml/dune KernelScript compiler plus explicit C/clang/libbpf/bpftool/module tools are pinned as declared derivation inputs.
- **Security**: generated build scripts are evidence only and never executed automatically; privileged artifacts are not loaded by Mantle.
- **Integration**: candidate outputs can feed the separate Onix kernel-bundle and ChaosControl validation changes after their gates exist.
- **Claims**: passing proves bounded source-to-artifact materialization under one pinned cohort. It does not prove language soundness, eBPF verifier acceptance, kernel safety, runtime behavior, compatibility outside the selected kernel, or production readiness.
