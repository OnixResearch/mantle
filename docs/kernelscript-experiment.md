# Bounded KernelScript experiment

Mantle carries a planning-only, explicitly beta experiment for KernelScript
`v0.1.2`. It is disabled by default and is not a package, release target,
OnixOS module, loader, verifier, or runtime integration.

The boundary and rejected ambient-state alternatives are recorded in
[ADR 0022](../adr/0022-bound-kernelscript-as-planning-only-experiment.md).
The currently implemented slice is a functional core. It validates typed input
facts, derives offline code-generation and compilation plans, classifies
already-produced generated projects, inspects supplied ELF bytes, projects
candidate ModulePack/BPF Pack handoffs, and emits canonical BLAKE3 receipts. It
does not execute a compiler, generated code, generated Makefiles, a linker,
`bpftool`, a module loader, or a BPF loader.

## Authoritative source pin

The upstream source identity is recorded in
[`packages/kernelscript-experiment/upstream-pins.ncl`](../packages/kernelscript-experiment/upstream-pins.ncl):

- release: `v0.1.2`
- Git revision: `0c80d4e4ac0029d34cbc9d65e76d78c075b64555`
- source archive: `kernelscript-0.1.2-source.tar.gz`
- upstream SHA-256: `9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1`
- measured archive BLAKE3: `439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57`

[`packages/kernelscript-experiment/source.ncl`](../packages/kernelscript-experiment/source.ncl)
is a flat fixed-output fetch derivation for those official bytes.

There is no accepted compiler closure yet. Upstream ships
`kernelscript.opam`, but the tagged source and release assets do not include an
immutable `opam.locked`; upstream release and CI workflows resolve mutable opam
repositories. Mantle therefore refuses to treat an upstream release binary,
ambient opam switch, or host `kernelscript` executable as a pinned compiler
closure.

## Typed profile and bounds

[`lib/kernelscript_experiment.ncl`](../lib/kernelscript_experiment.ncl) exports
the closed `KernelScriptExperimentProfile` contract. A profile binds:

- `.ks` source path, size, and BLAKE3 identity;
- exact compiler source revision/archive, dependency-lock identity, ordered
  package/version/artifact members, compiler executable, and closure identity;
- explicit OCaml, dune, menhir, Clang/C compiler, bpftool, libbpf, ELF, zlib,
  and kernel-build identities;
- target architecture, kernel release/build identity, BTF, headers, and config;
- selected output classes, exact generated-file classes, compilation flags,
  numeric limits, denied network policy, and mandatory non-claims.

The Rust core independently validates the typed data and enforces hard ceilings
on text, collections, generated files, output bytes, ELF sections, plans, and
receipt blockers. The fixture at
[`tests/fixtures/kernelscript-experiment/profile-positive.ncl`](../tests/fixtures/kernelscript-experiment/profile-positive.ncl)
contains synthetic toolchain and target identities for planning tests only.
Those identities are not authoritative Onix cohort evidence.

## Offline planning and generated-project authority

`crunch-kernelscript-core` has no filesystem, process, network, or async API.
Its code-generation plan names only the admitted compiler artifact and uses
`NetworkPolicy::Denied`. Its generated-project classifier accepts an explicit
bounded byte set, rejects missing/extra/escaping/executable/oversized files,
and emits a canonical manifest with per-file BLAKE3 identities.

Generated `Makefile` and `Kbuild` files are retained only as review evidence.
They are never compilation inputs. Mantle-owned compilation requests identify
one allowlisted tool role, explicit argument atoms, admitted source members,
exact target inputs where required, and one output path. Compilation plans carry
the canonical generated members, and output inspection rejects any reported
source-member set that differs from the generated inputs consumed by that exact
output-producing step. Command admission
rejects `make`, shells, generated build scripts, path escapes, unknown tools,
shell operators, response-file expansion, and shell interpolation before any
imperative shell could execute them.

## Kernel admission and independent inspection

The pure target gate compares every eBPF/module plan with an observed fact set
for architecture, kernel release/build identity, BTF, headers, config,
toolchain identities, and exact flags. Ambient running-host inputs are never a
fallback. Missing, stale, reordered, substituted, or mismatched facts block the
affected class.

Static inspection accepts bounded output bytes independently for each selected
class. It checks BLAKE3, size, plan identity, exact generated-source identities,
ELF64 little-endian shape, checked section ranges, target machine, bounded BTF
header/type/string ranges for eBPF, and bounded module `vermagic` metadata for
the exact kernel release. Success in one class does not
mask a sibling blocker. This is shape inspection, not execution or semantic
verification.

## Candidate handoff and receipts

Only admitted, independently inspected eBPF and module members can become
frontend-neutral `BpfPack` or `ModulePack` candidate projections. Every
projection is fixed to `experimental-unverified` and retains target, member,
manifest, inspection, and profile identities. Candidate projection never
claims verifier acceptance, load/attach success, deployability, or production
support.

Canonical `mantle-kernelscript-experiment-receipt-v1` receipts bind source,
compiler admission/closure, target admission, code-generation and generated
manifest identities, compilation plans, per-output inspection outcomes,
candidate packs, blockers, and non-claims with BLAKE3. Plan, inspection, and
candidate DTOs carry recomputable identities; receipt construction revalidates
them and exact member/inspection relationships instead of trusting an
`accepted` boolean. Receipt validation rejects
raw source, logs, credentials, URL userinfo, absolute host paths, and bounded
text violations.

## Current blockers and exact non-claims

Actual compiler materialization, code-generation execution, artifact builds,
real output inspection, and runtime handoff remain blocked by two missing
authoritative inputs:

1. Upstream provides no immutable dependency lock for the OCaml/dune/opam
   compiler closure.
2. This worktree provides no authoritative exact Onix kernel/BTF/header/config
   and compilation-tool cohort for the experiment.

Consequently, the current slice does **not** prove language soundness, BPF
verifier acceptance, kernel safety, runtime correctness, successful module
loading, successful BPF loading/attachment, deployability, Onix semantics,
ChaosControl evidence, production readiness, or release-binary trust.

## Upgrade procedure

A future cohort update must:

1. record a new exact release/revision and verify official SHA-256 plus measured
   BLAKE3 source bytes;
2. produce and review a complete immutable opam dependency lock rather than
   consulting ambient opam state;
3. bind the resulting compiler executable and full closure by BLAKE3;
4. admit one exact Onix kernel build with matching BTF, headers, config,
   architecture, release, toolchain, and flags;
5. rerun code generation, review the exact generated-file manifest, and update
   expected classes only through review;
6. execute only Mantle-owned plans, independently inspect every output, and
   preserve blockers/non-claims in receipts;
7. obtain separate Onix semantic and ChaosControl load/attach evidence before
   changing `experimental-unverified` status.

Changing a release string or using a newer host compiler/kernel is not an
upgrade procedure.

## Focused checks

```console
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-core \
  cargo test -p crunch-kernelscript-core
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-integration \
  cargo test -p mantle --test kernelscript_experiment
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-wasm \
  cargo -Zbuild-std=core,alloc check -p crunch-kernelscript-core \
  --target wasm32-unknown-unknown
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-core \
  cargo clippy -p crunch-kernelscript-core --all-targets -- -D warnings
```

These commands validate the functional core and fixtures only. They do not
resolve either authoritative-input blocker and are not compiler/build/runtime
evidence.
