# Bounded KernelScript experiment

Mantle carries a disabled-by-default beta experiment for KernelScript `v0.1.2`.
It has two deliberately separate parts:

1. `crunch-kernelscript-core`, a no-std functional core for typed admission,
   planning, generated-project classification, output inspection, candidate
   projection, and canonical receipts;
2. an opt-in pinned Nix probe observation under
   [`nix/kernelscript-experiment.nix`](../nix/kernelscript-experiment.nix).

The production shell invokes the first through the small
`crunch-kernelscript-adapter` binary. The adapter performs bounded no-follow
filesystem reads, delegates profile/compiler/target admission, exact generated
shape classification, planning, and receipt construction to the core, and
writes the two core reports retained by the Nix output. Successful build and VM
results remain observations: both core receipts are blocked on the external
target-authority boundary rather than promoted to accepted Onix materialization.

The architecture boundary and rejected ambient-state alternatives are recorded
in [ADR 0022](../adr/0022-bound-kernelscript-as-planning-only-experiment.md).

## Authoritative source pin

The upstream source identity is recorded in
[`packages/kernelscript-experiment/upstream-pins.ncl`](../packages/kernelscript-experiment/upstream-pins.ncl):

- release: `v0.1.2`;
- Git revision: `0c80d4e4ac0029d34cbc9d65e76d78c075b64555`;
- source archive: `kernelscript-0.1.2-source.tar.gz`;
- upstream SHA-256: `9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1`;
- measured archive BLAKE3: `439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57`.

[`packages/kernelscript-experiment/source.ncl`](../packages/kernelscript-experiment/source.ncl)
is a flat fixed-output fetch derivation for those official bytes.

The Nix observation builds the compiler from this archive. It does not use an
upstream binary, ambient opam switch, or host `kernelscript` executable. Its
OCaml/dune/menhir/library closure comes from the materialized nixpkgs input at
revision `6201e203d09599479a3b3450ed24fa81537ebc4e`, locked by
`flake.lock` with NAR hash
`sha256-ZojAnPuCdy657PbTq5V0Y+AHKhZAIwSIT2cb8UgAz/U=`. The observation records
BLAKE3 for the compiler binary and the sorted Nix closure path set.

This is stronger than mutable opam resolution. The production adapter now
remeasures the archive SHA-256 interoperability field, archive/compiler/tool
BLAKE3 identities, and sorted closure path-set identity, then admits that exact
compiler observation through `crunch-kernelscript-core`. This compiler
observation is not an accepted Onix kernel-target authority.

## Materialized target cohort and authority boundary

The opt-in Nix route materializes one `x86_64-linux` cohort from the same locked
nixpkgs input:

- Linux `6.18.20` image, development tree, config, and `vmlinux` BTF;
- clang `21.1.8`, bpftool `6.18.7`, GCC `15.2.0`;
- libbpf `1.6.3`, elfutils `0.194`, and zlib `1.3.2`.

The generated observation remeasures the kernel image, config, and BTF with
BLAKE3 and records that running-host BTF was not used.

It also records the reported sibling OnixOS commit/tree/blob identifiers that
motivated cohort selection. Those identifier strings are **reported metadata
only**: the corresponding OnixOS Git objects are not Nix inputs to this route,
are not materialized and remeasured by it, and are not accepted Onix authority.
Only the locked nixpkgs revision/NAR is materialized authority for this build.

## Pure-core contract

[`lib/kernelscript_experiment.ncl`](../lib/kernelscript_experiment.ncl) exports
the closed `KernelScriptExperimentProfile` contract. The Rust core independently
validates the typed data and enforces hard ceilings on text, collections,
generated files, output bytes, ELF sections, plans, and receipt blockers.

`crunch-kernelscript-core` has no filesystem, process, network, or async API. It
can:

- admit an immutable compiler cohort and exact target facts;
- produce offline code-generation and explicit compilation plans;
- classify supplied bounded generated-file bytes;
- reject missing, extra, escaping, executable, or oversized members;
- inspect supplied ELF/BTF/module bytes independently by output class;
- project only admitted and inspected candidate handoffs;
- emit canonical BLAKE3 receipts with strict non-claims.

Generated `Makefile` and `Kbuild` files are evidence-only. Core command admission
rejects `make`, shells, generated build scripts, path escapes, shell operators,
response-file expansion, and shell interpolation.

## Opt-in probe observation

The pinned route is exposed only on `x86_64-linux`:

- `.#kernelscript-compiler`;
- `.#kernelscript-production-shell`;
- `.#kernelscript-production`;
- `.#kernelscript-production-cohort`;
- `.#kernelscript-production-runtime-check`;
- `.#checks.x86_64-linux.kernelscript-production`.

It compiles the reviewed positive probe and private/kfunc source fixtures with
the source-built compiler, retains the exact generated C/Makefile/Kbuild shape,
and never executes generated Makefiles or Kbuild files. It then builds only the
probe eBPF object, bpftool skeleton, and userspace loader with explicit pinned
tool paths and flags. Structural checks cover ELF headers, sections,
relocations, and BTF.

The output record is
`evidence/probe-observation.json`, schema
`mantle-kernelscript-probe-observation-v1`. It remains intentionally separate
from the core receipts. `evidence/probe-core-report.json` and
`evidence/kfunc-core-report.json` carry the core-generated manifests, plans,
and canonical receipts. Both receipts keep `target_identity_blake3 = null`,
the `kernel-target-observation-only` blocker, no candidate packs, and strict
non-claims; the kfunc receipt additionally preserves the unchecked module gate.

The exact-kernel VM gate boots Linux `6.18.20`, loads the object through
bpftool's verifier/load path, and runs the generated loader through successful
`do_exit` attach/detach. Current checked evidence is in
[`cairn/changes/add-bounded-kernelscript-experiment/evidence/probe-production-evidence.md`](../cairn/changes/add-bounded-kernelscript-experiment/evidence/probe-production-evidence.md).

The Mantle template at
[`packages/kernelscript-experiment/production.ncl`](../packages/kernelscript-experiment/production.ncl)
is also disabled by default. It resolves only when the operator explicitly
provides the generated pinned cohort import path.

## Open blocker

The **private/kfunc module** task remains deliberately unchecked. The checked
Nix route does not build it, and the checked VM route does not load/unload it or
verify the kfunc/XDP object. Temporary or external prototype output is not
sufficient to complete the task.

The feature remains disabled by default. Core-backed generated-shape admission
and receipt identity do not grant external target authority or complete the
module/kfunc gate.

## Exact non-claims

The current evidence does not prove language soundness, compiler soundness,
kernel safety, cross-kernel compatibility, module load/unload, kfunc/XDP
runtime behavior, deployability, accepted Onix semantics, ChaosControl evidence,
production readiness, default enablement, or release-binary trust.

Probe verifier/load and attach/detach success is specific to the exact recorded
Linux `6.18.20` VM cohort. It does not generalize beyond that cohort.

## Upgrade procedure

A future cohort update must:

1. record a new exact release/revision and verify official SHA-256 plus measured
   BLAKE3 source bytes;
2. bind an immutable compiler executable and complete dependency closure;
3. materialize and hash the target kernel image, BTF, headers, config, and tools;
4. invoke the pure core for compiler/target admission and all plans;
5. review the exact generated-file manifest and keep generated build scripts
   inert;
6. independently inspect every selected output and preserve sibling blockers;
7. rerun exact-kernel verifier/load and attach/detach VM gates;
8. obtain separate Onix semantic and ChaosControl evidence before changing
   `experimental-unverified` status.

Changing a release string or using a newer host compiler/kernel is not an
upgrade procedure.

## Focused checks

```console
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-core \
  cargo test -p crunch-kernelscript-core
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-adapter \
  cargo test -p crunch-kernelscript-adapter
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-integration \
  cargo test -p mantle --test kernelscript_experiment
nix develop -c env CARGO_TARGET_DIR=/tmp/mantle-kernelscript-core \
  cargo clippy -p crunch-kernelscript-core --all-targets -- -D warnings
nix build --no-link .#checks.x86_64-linux.kernelscript-production
nix build --no-link .#packages.x86_64-linux.kernelscript-production-runtime-check
```

The Rust commands validate the functional core, bounded adapter, and synthetic
fixtures. The last two validate the core-backed pinned probe observation and
exact-kernel VM rail. They do not complete the external module/kfunc gate.
