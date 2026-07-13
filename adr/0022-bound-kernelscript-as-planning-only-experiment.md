# ADR 0022: Bound KernelScript as a planning-only experiment

## Status

Proposed

## Context

KernelScript `v0.1.2` can generate userspace, eBPF, module, test, and generated
build-script files from a `.ks` source. Its release publishes an authoritative
source archive, but not an immutable opam dependency lock. Its CI and release
workflows resolve mutable opam repositories. Mantle also lacks one authoritative
Onix target cohort binding the exact kernel build, architecture, release, BTF,
headers, config, compiler flags, and compilation tools needed to assess eBPF or
module output.

Accepting an upstream binary, ambient opam switch, generated Makefile, running
host kernel, or local `/sys/kernel/btf/vmlinux` would make the experiment depend
on undeclared authority. Conversely, pure planning and admission checks are
useful before those inputs exist, provided they cannot be mistaken for compiler,
build, verifier, loader, runtime, or release evidence.

ADR 0010 keeps Mantle build-shaped rather than runtime-shaped. ADR 0015 keeps
machine artifacts Rust-owned with Nickel as a typed review boundary. The
accepted kernel-bundle OCI projection preserves Onix semantics as external.

## Decision

Mantle will carry KernelScript only as an explicit, disabled-by-default beta
experiment until an immutable compiler closure and exact Onix target cohort are
available.

The experiment profile is a closed typed Nickel record. A no-std Rust functional
core independently validates normalized facts and owns bounded code-generation
plans, generated-project classification, explicit compilation plans, kernel
input admission, output shape inspection, candidate projection, and canonical
BLAKE3 receipts. The production Nix shell uses a small std adapter for bounded
no-follow reads and calls this core for generated-shape/receipt semantics. The
core itself has no filesystem, process, network, async, loader, or running-host
API.

The official source archive is pinned by upstream SHA-256 and measured BLAKE3.
A source pin is not a compiler-closure pin. Mantle will not claim compiler
materialization until a reviewed immutable opam lock binds every dependency and
the resulting executable plus closure bytes receive exact BLAKE3 identities.

Generated `Makefile` and `Kbuild` files are retained as bounded review evidence
only. Mantle-owned plans identify allowlisted tools, argument atoms, generated
source members, target inputs, and outputs directly. Command admission rejects
`make`, shells, generated build scripts, unknown tools, shell syntax, response
files, and path escapes before execution.

Every eBPF or module plan binds an exact Onix target cohort. Ambient running-host
BTF, headers, config, release, architecture, and tools are never fallbacks. Each
selected output class is inspected independently; sibling success cannot hide a
blocker. Static ELF/BTF/module shape inspection is not BPF verifier acceptance,
load/attach evidence, kernel safety, or runtime correctness.

Frontend-neutral ModulePack/BPF Pack projections remain
`experimental-unverified` until separate Onix semantic and ChaosControl runtime
gates accept the exact member and target identities. Mantle receipts preserve
blockers and non-claims and exclude source bodies, raw logs, credentials, and
full host paths.

## Consequences

Mantle can test profile drift, dependency/network rejection, generated-project
authority, planner allowlists, exact target admission, malformed/stale output,
candidate overclaim, and receipt leakage without standing up a compiler or
kernel. Missing authoritative inputs remain exact bounded blockers rather than
being replaced with host state.

The pinned Nix observation can execute code generation, build and inspect the
probe, and emit core-backed blocked receipts. It still cannot admit the
materialized Linux probe cohort as accepted Onix target authority, complete the
private/kfunc module gate, load that module, or grant deployability. Completing
those tasks requires new external authority and runtime evidence, not merely
unchecking a feature flag.

## Alternatives Considered

### Use the upstream release binary as the compiler pin

Rejected. A release binary does not identify the complete OCaml/dune/opam
closure or prove how its bytes were produced.

### Resolve opam dependencies during the derivation

Rejected. Mutable repository state and network resolution violate offline exact
materialization.

### Execute the generated Makefile in a sandbox

Rejected. Sandboxing constrains side effects but does not turn generated command
text into a reviewed Mantle plan or establish exact tool/member authority.

### Compile against the running host kernel

Rejected. Host BTF, headers, config, release, and architecture are ambient facts
that cannot replay an exact candidate cohort.

### Treat a compiling object as deployable

Rejected. Compilation and static shape checks do not establish Onix semantics,
BPF verifier acceptance, module/BPF load success, ChaosControl behavior, or
production readiness.
