# Offline HIP compilation (bounded Linux fragment)

Issue #13's initial backend is **HIP/Clang only**: x86_64 Linux build host,
AMD LLVM **22.0.0** from ROCm **7.2.3**, explicit **gfx900**, code-object ABI
version 4, and one relocatable AMDGPU ELF object. CUDA, other architectures,
other toolkit/compiler tuples, host/device linking, fat binaries and execution
are not supported by this adapter. A successful compile is not GPU execution,
correctness, driver compatibility, JIT or performance evidence.

`crunch-gpu-core` is the no_std authoring policy. `crunch-gpu` is the filesystem
and process shell; `mantle-gpu lower PLAN.json` emits an ordinary generic
input-addressed derivation, and `gpu.mkGpu` in `lib/gpu.ncl` provides Nickel
lowering. No scheduler dispatch, independent store, downloader, trust registry
or native GPU detection is added. The builder is an admitted `mantle-gpu run`
executable, not a host executable path. Unsupported tuples and unsafe plan
members are rejected before invoking a compiler. Compilation itself must go
through Mantle's ordinary signed-input, network-denied sandbox and signed-output
pipeline; calling the runner directly is not proof of isolation.

The Rust policy API is `validate(Plan) -> Result<AdmittedPlan, Rejection>`.
Admission returns the original owned plan plus typed `Paths` for its declared
members, tools, sysroot and toolkit. The shell consumes these resolved paths
and retains filesystem containment/kind checks; it does not reimplement path
policy. `crunch_gpu::lower` and `build` consume the plan. The serialized plan
and derivation/provenance wire contracts are unchanged.

## Inputs and outputs

The closed plan binds all of the following in its serialized derivation:

- signed store-root role map, source kernel, header directories and ordered,
  separately declared generated headers;
- device compiler, separately explicit host compiler, LLVM inspector and adapter
  executable identities; the tools' version labels are compatibility checks,
  **not** content authority;
- build platform, host triple, sysroot, compiler resource directory, toolkit root
  and revision, device bitcode directory and host shared-runtime library members;
- explicit architecture, optimization level 0–3, bounded preprocessor definitions,
  expected unmangled kernel entry points and source-date epoch.

There are no arbitrary compiler/linker arguments, response files, compiler
plugins, host tools on PATH, ambient includes, environment injection, retained
compiler caches or automatic source acquisition. Generated inputs must already
have been realized and admitted. Runtime libraries are bound and checked even
though this fragment does not link or execute a HIP host runtime. Native CMake
outputs can be used as ordinary declared header/library roles, but CMake does
not hide or choose the GPU toolchain.

Source, generated-header, device-bitcode and object files retain a 64 MiB
per-file bound. Declared host runtime libraries have a separate 256 MiB bound,
so the required LLVM shared library is not constrained by the GPU object limit.
Both file classes are hashed incrementally; this does not relax the output
object's admission bound.

Every referenced member is canonicalized into an explicitly admitted role root;
symlink escapes fail. Transitive shared-library, compiler-wrapper and LLVM
resource roots must be present in the role map as well. Header preprocessor
access beyond those roots is denied by the existing sandbox. For Nix-built tools
with absolute loader/RPATH references, retain logical `/nix/store` (a scratch
physical `--store` is fine). A different logical prefix needs a correctly
relocated, independently admitted closure; it is not a string substitution.

The invocation selects `--offload-device-only -c -x hip`, `--offload-arch=gfx900`,
`-mcode-object-version=4`, an explicit host target/sysroot/resource directory,
and an explicit ROCm device-library directory. `-nogpuinc -nostdinc -nostdinc++`
disable ambient/default includes. The minimal fixture uses HIP language kernel
attributes without HIP runtime headers or a HIP host program. No installed GPU,
`amdgpu-arch`, `rocminfo`, `/dev/kfd`, render node or GPU driver is consulted.

The runner requires these device-library members before compilation:
`hip.bc`, `ocml.bc`, `ockl.bc`, `oclc_daz_opt_off.bc`,
`oclc_unsafe_math_off.bc`, `oclc_finite_only_off.bc`,
`oclc_correctly_rounded_sqrt_on.bc`, `oclc_wavefrontsize64_on.bc`,
`oclc_isa_version_900.bc`, and `oclc_abi_version_400.bc`.

Outputs:

- `lib/kernel.o`: ELF64 little-endian, `ET_REL`, `EM_AMDGPU` (224), gfx900 machine
  flags (low byte `0x2c`); each expected entry point must be a defined global
  function with a defined global `<entry>.kd` kernel descriptor;
- `share/mantle/gpu-llvm-inspection.txt`: actual admitted LLVM readobj headers,
  symbols and notes;
- `share/mantle/gpu-provenance.json`: complete plan, plan BLAKE3, checked source,
  generated-header, runtime and required-bitcode file BLAKE3s, observed tool
  versions, compiler arguments, ELF observations, artifact BLAKE3, and explicit
  `compilation_only: true`.

ELF admission validates each complete relocation field with checked offsets and
the [pinned AMDGPU relocation widths](https://raw.githubusercontent.com/ROCm/llvm-project/f58b06dce1f9c15707c5f808fd002e18c2accf7e/llvm/include/llvm/BinaryFormat/ELFRelocs/AMDGPU.def).
Unknown relocation kinds and fields extending beyond their target section are
rejected. Kernel descriptor property bit 11 (`USES_DYNAMIC_STACK`) is admitted
because the pinned producer emits it; other reserved descriptor bits remain
rejected. This validates object structure, not dynamic-stack execution.

The outer signed PathInfo and actual NAR authenticate the complete input trees;
the selected-file hashes in provenance are observations, not a new admission
mechanism. An existing CMake `nativeInput { root, role, root_env }` declaration
can export the admitted result to consumers using `GPU_OBJECT_ROOT`; it does
not put a GPU runtime or target binary on PATH.

## Parent-executable real-tool proof

No real compilation acceptance is established merely by checking in this
adapter. Run the proof on the immutable integrated source, after the parent has
built the binaries and prepared the signed seed below. Do not run it on a
concurrently changing checkout. This driver downloads nothing, accepts no
licenses, installs no drivers, changes no permissions on devices and never runs
a device program.

Required inputs, supplied through existing acquisition/admission authority:

1. Current immutable `mantle` and `mantle-gpu` binaries; a signed seed store/state
   containing the same adapter executable and its runtime closure.
2. Authorized, fully present AMD LLVM 22.0.0/ROCm 7.2.3 compiler and inspector
   closures, sysroot, resource directory, bitcode and host shared libraries.
   All roots need existing signed PathInfo accepted by explicit public keys.
   Acquisition, package provenance and redistribution authorization are separate
   decisions; this adapter never makes them.
3. A signed fixture root containing **all** files from
   `tests/fixtures/gpu-hip/`, under a directory such as `fixture/`.
4. A JSON plan as described below, with independently pinned BLAKE3; explicit
   trusted signing public keys and a local build signing key; Linux bwrap/FUSE
   prerequisites for the existing Mantle sandbox; a BLAKE3 observer; Python 3.9+
   and spacious scratch. The seed must not already contain the selected GPU
   derivation's output.

Local candidate packages observed during implementation (presence is **not**
authorization, signature verification or real-tool acceptance):

- `/nix/store/chzv0dzb99zsmzxcj4a4jmawy35qmd06-rocm-toolchain`
  (`bin/amdclang++`, `bin/llvm-readobj`, `lib/clang/22`);
- `/nix/store/p3w3sf7jlb1m3q2v0csgq0rg25mpkz0d-llvm-22.0.0-rocm`
  (the toolchain's LLVM inspector resolves here);
- `/nix/store/8jdkas7zs4yzqs0n3xrrnnmc9s3xx5zb-rocm-device-libs-22.0.0-rocm`
  (`amdgcn/bitcode`, including gfx900 and ABI400 members).

The exact additional roots are the authorized transitive closure of the selected
compiler/inspector/sysroot/runner; do not infer them from these version strings.
They must all appear in `roles`. Existing Nix metadata can inventory acquisition
inputs, but Nix evaluation/build is not part of compilation or this proof.

The plan uses this field layout (metavariables here describe **required caller
inputs**, not a runnable plan with invented store hashes):

```text
schema: mantle-gpu-plan-v1
name: hip-gfx900-minimal
backend: hip-clang
build_platform: x86_64-linux
store_prefix: /nix/store
roles: map from explicit role names to actual signed logical store roots
kernel: { role: fixture, member: fixture/kernel.hip }
headers: [{ role: fixture, member: fixture }]
generated: [{ role: fixture, member: fixture/generated_config.h }]
clang: { role: compiler, member: bin/amdclang++, version: 22.0.0 }
host_compiler: { role: compiler, member: bin/amdclang++, version: 22.0.0 }
inspector: { role: compiler, member: bin/llvm-readobj, version: 22.0.0 }
runner: { role: adapter, member: bin/mantle-gpu, version: mantle-gpu-v1 }
host_target: x86_64-unknown-linux-gnu
sysroot: sysroot
resource_dir: { role: compiler, member: lib/clang/22 }
toolkit: compiler
toolkit_revision: 7.2.3
device_libraries: { role: device-libs, member: amdgcn/bitcode }
runtime_libraries: explicit { role, member } list including compiler host runtime libraries
architecture: gfx900
optimization: 2
defines: {}
entry_points: [mantle_add_one]
source_date_epoch: 1
```

Supply actual paths/digests from the parent-controlled input manifest:

```bash
python3 scripts/prove-gpu-hip.py \
  --mantle "$MANTLE" --runner "$GPU_RUNNER" --b3sum "$B3SUM" \
  --plan "$GPU_PLAN" --expected-plan-blake3 "$GPU_PLAN_BLAKE3" \
  --expected-mantle-blake3 "$MANTLE_BLAKE3" \
  --expected-runner-blake3 "$GPU_RUNNER_BLAKE3" \
  --seed-store "$SIGNED_SEED_STORE" --seed-state "$SIGNED_SEED_STATE" \
  --trusted-public-key "$AUTHORIZED_INPUT_PUBLIC_KEY" \
  --signing-key "$LOCAL_BUILD_SIGNING_KEY" --run-dir "$FRESH_GPU_PROOF_DIR"
```

The driver calls `source bundle import-store-path --pin` on every selected root
in each isolated seed copy: signatures and the actual physical NAR are checked,
not assumed from a path. It then invokes **real `mantle build`** with strict
hermeticity, offline source preflight and substitution disabled. It requires:

- two uncached independent clean builds with the same derivation and identical
  BLAKE3s for every output file, plus a verified cache hit;
- actual ELF architecture, global entry point and kernel descriptor inspection;
- changed optimization identity, changed kernel identity **and** code, and
  changed declared generated input identity **and** code;
- early unsupported architecture/toolkit/host compiler, traversal and undeclared
  role failures; a missing bitcode library failure;
- a host-only header that is present outside but inaccessible to the compiler;
- a live host loopback listener (positive host control) inaccessible to an
  ordinary network-denied derivation using the same admitted runner;
- a copied source byte tamper rejected at the existing signed PathInfo/actual NAR
  source admission boundary.

All commands, stdout/stderr, build state/logs and output trees remain under the
fresh run directory. `proof.json` is written on success **or failure**; only
`status: passed` after all cases is real-tool acceptance for this exact cohort.
The proof command's `probe-network` runner action is a negative isolation control,
not part of kernel compilation or an extra source/network capability.

## Limits and interpretation

Architecture/toolkit revisions outside the one tuple are rejected, not silently
retargeted and not served from a prior cached artifact. Admitted input-root or
option changes enter both generic derivation identity and full plan provenance;
new supported tuples require independent execution evidence before widening the
core allowlist. Version labels alone cannot authorize a new tool closure.

`SOURCE_DATE_EPOCH`, fixed locale/timezone/work directory, no debug information,
source-prefix mappings and no compiler cache reduce known variation. Two
matching clean builds measure only this exact LLVM/backend/input cohort, not all
versions, optimization modes, kernels or hosts. LLVM-generated notes, metadata,
codegen and externally supplied generated inputs can change bytes across
compiler/toolkit revisions. The proof deliberately compares full object,
inspection and provenance bytes without masking differences.

Optional device execution must be a separate capability-gated workflow/report
that identifies its driver, runtime, hardware and test inputs. This adapter
neither provides that capability nor asserts such a report exists.
