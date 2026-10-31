# Design: Distribute Rust compilation units

## State and existing contracts

This is proposed work. The accepted `rust-unit-plan-builds` specification
already requires ordinary generic dynamic plans, per-package sources, direct
dependencies plus library reference manifests, a declared helper and fail-closed
unsupported graphs. `run-cargo-build-scripts-as-plan-units` adds target facts,
sandboxed execution, `flags` outputs and explicit native roles; its outstanding
lifecycle acceptance remains its owner's obligation.

Relevant source owners are `crates/mantle-cargo-unit-plan/src/lib.rs`,
`producer.rs`, `unit_helper.rs`, `build_script.rs`, `lib/cargo_unit_plan.ncl`
and the generic `crunch-build` dynamic-plan admission path. Extend these seams,
not `src/rust_plan.rs` with a second project execution loop. Architecture and
upstream dependencies are in
[ADR 0138](../../../adr/0138-distribute-compiler-units-through-native-build-goals.md).

## Execution flow

The selected project still produces its unit graph offline through its declared
sandboxed producer. The producer may execute locally or on a fitting native
worker; the client must admit its signed `plan` and `sources` outputs, all
required source-slice identities and plan-root bindings before registering
children. The generic Worker then schedules each ready unit to the selected
pool. An ordinary final consumer binds the intended plan root; exporting only
the producer is not a successful compiled application.

Each unit keeps its declared helper, rustc, sysroot, linker/runtime roots,
package source slice, direct dependencies and reference-scanned transitive
closure. The helper reads the same argument/environment files and dependency
manifests inside each worker sandbox. Concrete tool and source inputs travel
through native input admission; workers do not evaluate Nickel or resolve
packages from the network. The generic worker learns no Cargo target kinds.

## Host, execution and target roles

Frontend admission must express three facts without conflating them:

- The system on which the selected compiler/helper/linker executable runs.
- The target triple of the artifact being produced and its declared sysroot.
- The execution system and ABI of tools consumed during compilation, notably
  proc-macro dynamic libraries and compiled build scripts.

Ordinary cross-target libraries are still compiled on the compiler's execution
system. A target triple alone must never select a worker that cannot run the
compiler. Proc macros must match the host compiler process which loads them;
build-script binaries must run on their declared host system while receiving
the declared `HOST`, `TARGET`, target facts and native roles. Model these as
frontend-produced generic requirements and inputs, not Rust branches in pool
placement. Unadmitted triples/runtime combinations fail lowering or preflight
before source upload. This change does not claim new target coverage.

## Finished-artifact dependency barriers

A dependency becomes available only after the current attempt has finished and
its entire declared output set passed native output admission. This applies to
libraries/rmeta, executable proc macros, compiled scripts, target-facts outputs,
`OUT_DIR`, generated source, typed `flags` and native library/linker inputs.
A successful build script does not excuse a missing flag file or generated
input; a failed script's previous output must not be reused as its new result.

For a downstream unit on a different worker, transfer the complete dependency
manifest/reference closure before running rustc. Missing transitive libraries,
proc-macro runtime dependencies or native roles are typed failures. Preserve
unit-helper argument ordering, immediate/transitive flags, `links` metadata,
path remapping and output naming. No dependent may consume provisional bytes,
ready notifications, local filesystem leftovers or another attempt's outputs.
Final linking remains an ordinary declared unit with its complete finished
closure; it may execute on any fitting worker, not necessarily the client.

## Identity and reuse

Keep the existing Rust unit identity, `-C metadata`, canonical plan bytes,
source-slice mapping, dependency resolution and action-result admission. Pool
names, endpoint generations, worker physical paths, attempt IDs and resource
quantities are scheduling data, not compiler inputs. The logical store prefix
and semantic tools/arguments/environment/target/sandbox/network facts remain
identity-bearing. Changing a toolchain or declared native input must change
all affected identities even if a nominal unit label is unchanged.

Use immutable completed store artifacts and existing trusted action-result
reuse. Do not share `target/`, incremental state, build-script scratch or warm
workspace affinity across hosts. A placement change must not cause an unchanged
unit to recompile after a verified hit. Local compiler-cache/slot and wait
sockets remain excluded from pool execution. Early-interface metadata and
provisional sessions stay with their existing externally gated changes.

## Supported fragment and configuration

Reuse the existing opt-in `cargo_unit_plan` entry point and proposed native
pool selection; no separate Rust remote flags, credentials or compiler service.
Keep plan/source-slice/direct-input limits unchanged. The full workspace already
documented as outside the source-slice cap remains an explicit blocked case,
not an acceptance target disguised as a small fixture. Unsupported Cargo modes,
source kinds, triples and unbound script execution fail with existing typed
blockers, never host Cargo compilation. Preserve current lane non-claims:
`not-cargo-free-execution`, `not-full-cargo-compatibility`,
`not-compiler-correctness`, `not-release-reproducibility`, and
`not-bootstrap-correctness`.

## Acceptance matrix

1. **Same-plan baseline:** build a small branching graph with the identical
   toolchain, unit plan and logical prefix locally and through a one-/two-worker
   pool from separate stores. Execute the final application against an
   independent expected result. Compare stable artifact bytes for this same
   lane; compare only declared runtime semantics against the default offline
   Cargo lane, whose metadata/naming may legitimately differ.
2. **Actual concurrency:** use real rustc work and bounded synchronization at
   the executor boundary to retain overlapping lifetimes on two worker IDs.
   Neither two process wrappers nor two selected endpoints proves rustc ran.
3. **Host-sensitive consumers:** exercise a real proc macro and a sandboxed
   script producing generated Rust source/cfg/env plus a declared native
   library. Run the final program so a stale generated value is observable.
4. **Selective change:** edit one leaf package, a proc macro, generated input
   and a declared native provider in separate retained runs. Verify affected
   canonical identities/results change while the independent branch is reused;
   a producer rerun alone is not compilation evidence.
5. **Refusals:** unsupported triple, removed native role, missing transitive
   input, genuine rustc/script failure, worker death/cancel and stale output
   must fail without final successful publication or speculative dependents.
6. **Fresh repeat:** rebuild from clean client/worker state and retain exact
   source/tool/plan/output identities, compiler outcomes, admission receipts and
   final program output. Final independent-host acceptance belongs to
   `qualify-distributed-compilation`; this feature's local two-worker integration
   cannot make that stronger claim.

## Risks and rejected alternatives

Fine-grained actions may move a large sysroot/closure for little compute; retain
actual missing-input transfer and execution counts without claiming locality
from advertisements. Host-dependent scripts or absolute generated paths must
be rejected or corrected at the declared frontend boundary, never patched by
mounting client host directories. Shipping whole Cargo roots, adding a rustc
RPC, transmitting live incremental state and widening plan limits are rejected
because they either lose unit distribution or add independent correctness
contracts not needed by this feature.
