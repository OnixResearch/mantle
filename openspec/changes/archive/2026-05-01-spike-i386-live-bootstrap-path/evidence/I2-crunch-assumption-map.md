# I2 Crunch i386 assumption map

Task-ID: I2

Covers: `bootstrap.i386-live-bootstrap-spike.crunch-assumptions`

## Files inspected

- `lib/contracts.ncl`
- `lib/helpers.ncl`
- `lib/derivation.ncl`
- `bootstrap/tinycc-mes.ncl`
- `bootstrap/tinycc.ncl`
- `bootstrap/make-tcc.ncl`
- Repository content search for `i386`, `x86`, `x86_64`, `TCC_TARGET`, and live-bootstrap references.

## Assumptions that block a simple i386 drop-in

### 1. Public System enum has no i386 Linux value

`lib/contracts.ncl` currently admits only:

```nickel
'x86_64-linux,
'aarch64-linux,
'x86_64-darwin,
'aarch64-darwin,
```

`lib/helpers.ncl` mirrors that list in `system_to_string`. A first-class i386 derivation would need at least an `'i386-linux`/`'i686-linux` system enum addition and downstream conversion support. This is an API/spec change, not just a bootstrap script edit.

### 2. Derivations default to x86_64 Linux

`lib/derivation.ncl` sets:

```nickel
system | System | default = 'x86_64-linux,
```

Most bootstrap derivations omit `system`, so an i386 proof target needs explicit system selection or a new i386 bootstrap family rather than mutating existing amd64 derivations.

### 3. Current TCC derivations are intentionally amd64-specialized

`bootstrap/tinycc.ncl` documents that it is adapted from upstream i386 live-bootstrap but deliberately targets Crunch's amd64 chain:

```text
Adapted from live-bootstrap steps/tcc-0.9.27/pass1.kaem for Crunch's amd64 bootstrap chain.
```

Its repair body is dominated by x86_64-specific codegen/linker normalizations:

- `TCC_TARGET_X86_64` handoff behavior;
- edits to `x86_64-gen.c`;
- x86_64 GOT/PLT/static executable materialization;
- amd64 Mes runtime path compromises where upstream x86 rebuilds Mes CRT/libc.

An i386 path should not reuse this file by toggling one flag. It needs a sibling derivation that follows upstream `TCC_TARGET_I386=1` and x86 Mes CRT/libc rebuild semantics.

### 4. Current tcc-mes predecessor also carries amd64-only patches

`bootstrap/tinycc-mes.ncl` normalizes Mes/TCC defects around x86_64 codegen and uses amd64 M1 snippets such as the appended `abort` object. It parameterizes `MES_ARCH` internally in shell, but the checked-in implementation is tuned for the existing amd64 bootstrap and should not be assumed correct for a clean i386 proof without a dedicated derivation/audit.

### 5. Current Make derivation is not upstream i386 pass1

`bootstrap/make-tcc.ncl` imports the amd64 `tinycc.ncl`, adds Crunch-specific source edits, compiles `main.c` with `-DNO_FLOAT`, links with explicit Mes library paths, and verifies only `./make --version` in the builder. Prior failed evidence shows even installed outputs segfaulted on both `--version` and a trivial Makefile smoke.

For i386 proof, the right shape is a new proof derivation or spike script that tracks upstream `steps/make-3.82/pass1.kaem` more directly and adds the stronger Crunch smoke, not more edits to the amd64 `make-tcc.ncl`.

### 6. Runtime execution model needs a host feasibility check

On an x86_64 Linux kernel, native i386 static ELF execution depends on kernel IA32 compatibility support and available emulator/binfmt fallback if not enabled. Crunch's bwrap sandbox can execute host-supported binaries, but the spike must explicitly verify one of:

- native i386 static executable execution works under the current kernel/sandbox;
- qemu-i386/binfmt is available and can be declared as an input;
- the path is rejected/deferred because executing the proof target would require a larger runtime backend change.

## Follow-up implementation note

A small part of the assumption map has now been resolved for the spike: `lib/contracts.ncl` and `lib/helpers.ncl` accept and stringify the new `'i386-linux` system tag, covered by `tests/stdlib_tests.rs::i386_linux_system_tag_is_accepted`.

Verification command:

```sh
nix shell nixpkgs#clang -c env CARGO_TARGET_DIR=target cargo test --test stdlib_tests -- --nocapture
```

Result: passed, `21 passed; 0 failed`.

## Recommended prototype shape

Create a new sibling proof target rather than altering production amd64 derivations:

- `bootstrap/spike-i386-tinycc.ncl` or equivalent scratch/proof derivation for upstream `tcc-0.9.26 -> tcc-0.9.27` i386 semantics;
- `bootstrap/spike-i386-make-tcc.ncl` for upstream `make-3.82/pass1.kaem` semantics;
- explicit `system = 'i386-linux` only after the Nickel/Rust system enum can represent it, or keep the first spike as a clearly named amd64-hosted proof that emits i386 userland without pretending the system enum is already correct.

## Current decision impact

This map strengthens the previous recommendation: do **not** switch wholesale yet. The StageX route is promising, but Crunch needs a narrow i386 proof lane plus a small platform-enum/runtime feasibility decision before making it the primary bootstrap path.
