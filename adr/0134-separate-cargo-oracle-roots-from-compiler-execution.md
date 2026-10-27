# ADR 0134: Separate Cargo oracle roots from compiler execution capabilities

## Status

Proposed (2026-10-27). Source correction and construction route; fresh compiler,
native-tool and no-Cargo sandbox execution verification remains required.

## Context

A Cargo unit-graph producer needs Cargo and a reviewed oracle toolchain. Its
lowered compiler, target-facts and build-script units do not need that oracle
capability. Previously they inherited the mixed toolchain root, and the helper
prepended its bin directory. The retained real-helper bwrap observation launched
Cargo both by PATH and by its exact absolute path. The separate Mantle pipeline
probe timed out during import; it did not prove execution.

A store root is an indivisible capability. Removing a PATH entry cannot revoke
absolute access, and copying selected files out of an installed mixed toolchain
can preserve references to the old root or break compiler sysroot/RPATH behavior.

## Decision Drivers

- Preserve the authored execution-without-Cargo boundary.
- Keep the existing explicit input and whole-store-closure model.
- Preserve real compiler/std, target cfg, native tools and pkg-config execution.
- Do not introduce another sandbox, executable-name blacklist or filtered view.

## Decision

Treat producer `toolchain` and Cargo roots as oracle-only. The producer records
those whole roots in required `PlanBindings.oracle_inputs`. Pure lowering rejects
their use as execution compiler/helper/linker, runtime, target-linker or native
roots. It no longer mounts the oracle toolchain for any execution unit.
The helper takes `MANTLE_UNIT_RUSTC` and derives its compiler PATH from that
executable's directory; the old `MANTLE_UNIT_TOOLCHAIN` execution transport is
removed, without an alias. Native PATH entries remain explicitly declared.

Expose `packages.cargo-unit-compiler` for the already selected 2026-04-08 nightly
cohort, using the locked rust-overlay's individual `rustc` and `rust-std`
components (host plus x86_64 Linux's musl target) and its own
`lib/mk-aggregated.nix` builder. That builder establishes the aggregate compiler
sysroot and ELF RPATH from component inputs. This is
component construction, not a copied/filtered view of the mixed oracle output.
The internal builder interface is intentionally tied to the repository's locked
rust-overlay revision; an overlay update must review this call explicitly.

The operator admits/signs the complete new compiler closure, binds config `rustc`
to its executable, and declares the compiler root in both producer-config and
Nickel `runtime_inputs`. Oracle Cargo stays bound to the original cohort.
Other cross-target standard libraries/sysroots remain explicit matching-cohort
execution inputs; the aggregate is not an implicit arbitrary-target provider.

## Consequences

- Mixed roots reject instead of silently exposing Cargo during execution.
- Fresh configs and current producer/helper binaries are required; historical
  signed fixtures and receipts are not rewritten or promoted.
- Direct root admission is pure and bounded. Transitive closure review remains
  mandatory: a wrapper, symlink or native/library dependency on an oracle root
  restores its capability. The new aggregate is constructed independently to
  avoid that dependency, but its realized closure still needs inspection.
- Runtime proof must deny both Cargo lookup variants while observing successful
  compiler cfg queries and legitimate script/native/pkg-config behavior. Source
  admission rejection alone does not meet that acceptance requirement.
