## Context

KernelScript compiles one `.ks` source into generated userspace C, eBPF C, optional kernel-module C, tests, a Makefile, and README. The generated targets depend on exact KernelScript, clang/LLVM, libbpf, bpftool, BTF, kernel headers/config, architecture, and userspace compiler cohorts. Upstream warns that syntax and APIs can change without compatibility.

Mantle can make those inputs explicit and reproducible, but build success cannot establish verifier/runtime safety. The experiment should maximize inspectability and minimize authority.

## Decisions

### 1. Keep one explicitly beta profile

**Choice:** Define a typed Nickel `mantle-kernelscript-experiment-v1` profile with experiment ID, `.ks` source ref/BLAKE3, pinned compiler source revision, compiler/toolchain cohort, target architecture, target Onix kernel-build identity, BTF/header/config refs, selected output classes, named resource/size/count bounds, expected generated files, and non-claims. The profile is never part of default package or release matrices.

**Rationale:** An unstable language needs an exact replay envelope and visible opt-in.

### 2. Build the compiler as a declared Mantle input

**Choice:** Materialize KernelScript through a pinned source derivation and locked OCaml/dune/opam dependency closure. Network access is limited to fixed-output source acquisition; compiler build and code generation run offline in bwrap. The compiler binary and closure receive BLAKE3 identities.

**Rationale:** An ambient `opam install` or host compiler would make codegen evidence irreproducible.

### 3. Split code generation from native compilation

**Choice:** Stage one runs `kernelscript compile` with explicit output and BTF arguments and captures every generated file. A pure parser classifies the bounded expected project shape and computes a codegen manifest. Stage two consumes only admitted generated sources and explicit toolchain plans.

**Rationale:** Separating stages makes compiler drift visible and lets reviewers inspect generated privileged code before compilation.

### 4. Never execute the generated Makefile automatically

**Choice:** The generated Makefile and README are retained as evidence but are not executable inputs. A Mantle-owned pure planner derives allowlisted clang/cc/bpftool/module compilation steps from the experiment profile and admitted generated-file classes. Unknown commands, paths, shell substitutions, generated extras, or missing expected outputs block the experiment.

**Rationale:** Running compiler-generated shell text would grant the beta compiler an unnecessary second execution authority.

### 5. Keep artifact classes separate

**Choice:** Outputs are classified independently as generated source bundle, userspace loader, eBPF object set, optional kernel module set, test binary, and candidate pack manifest. Each class has exact member identities and build receipts. Success in one class does not imply success in another.

**Rationale:** A valid userspace binary can coexist with an eBPF compile failure or unsupported module path.

### 6. Bind privileged artifacts to kernel inputs

**Choice:** eBPF and module plans require exact architecture, Onix kernel-build identity, BTF, kernel headers/config, compiler flags, and toolchain identities. Missing BTF blocks BPF features that require it. Missing/mismatched headers or release metadata blocks module production. The experiment does not infer inputs from the running host.

**Rationale:** Ambient `/sys/kernel/btf/vmlinux` or `/lib/modules/$(uname -r)` would silently bind outputs to the build host rather than the declared target.

### 7. Validate static artifact shape but do not load

**Choice:** Mantle may run bounded ELF/BTF/section/relocation/module-metadata inspection and compile-time tests in the sandbox. It never calls `insmod`, attaches BPF programs, writes bpffs, requires root, or claims verifier acceptance from object parsing. Runtime load/attach belongs to the dedicated ChaosControl guest rail.

**Rationale:** Build systems should not mutate the host kernel, and static inspection is not runtime verification.

### 8. Produce candidate pack projections only

**Choice:** When the kernel-bundle OCI projection contract is available, the experiment can emit frontend-neutral candidate ModulePack/BPF Pack objects and manifests bound to the target identities. They remain `experimental-unverified` until Onix canonical validation and ChaosControl behavior evidence pass.

**Rationale:** Packaging improves handoff without promoting the artifacts prematurely.

### 9. Preserve complete bounded evidence

**Choice:** Receipts bind source, compiler, dependency closure, target kernel inputs, codegen command, generated-file manifest, explicit compile plans, output identities, static inspection outcomes, blockers, and non-claims with BLAKE3. Raw source may remain a normal build input, but receipts do not embed source bodies, build logs, host paths, or credentials.

**Rationale:** Reproduction needs exact identities; durable evidence does not need unbounded payloads.

## Risks / Trade-offs

- KernelScript may change output layout or semantics between revisions. Exact cohort pins and generated-file allowlists intentionally turn upgrades into reviewed changes.
- Reconstructing build commands instead of using the generated Makefile requires maintenance but prevents hidden shell authority.
- OCaml/opam closure materialization may be expensive; the experiment should keep one small fixed fixture cohort.
- Static BPF/module inspection can detect malformed artifacts but cannot predict every verifier or kernel behavior.

## Non-Goals

- Endorsing KernelScript for production or making it a default Onix language/toolchain.
- Implementing the KernelScript language, copying its compiler, or promising syntax compatibility.
- Running generated Makefiles, loading modules, attaching eBPF, mutating bpffs, or accessing the host kernel.
- Proving compiler/language soundness, verifier acceptance, kernel safety, performance, or release eligibility.
- Broad XDP/TC/struct_ops/kfunc coverage; the initial fixture set remains deliberately small and named.
