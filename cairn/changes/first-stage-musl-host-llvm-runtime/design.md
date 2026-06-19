# Design: first-stage musl host LLVM runtime

## Context

The first-stage source-root musl route uses the same logical triple for compiler host and provider target: `x86_64-unknown-linux-musl`. Mantle already creates a private wrapper directory for target linking, copies musl CRT objects plus GCC archives into a private runtime directory, normalizes `-static-pie` to `-static`, and exposes target-specific Cargo linker variables.

That setup currently happens only immediately before `run_rustc`. The earlier mrustc top-level build invokes Rust's LLVM CMake before the wrapper exists. Evidence from the failed committed-code rerun showed CMake using an ambient GNU C++ compiler and Nix zlib flags, while the final `rustc-main` link used the source-root musl wrapper.

## Core behavior

Add a generated-script setup step after the host mrustc/minicargo bootstrap tools are built and before the first translated `output/rustc`/LLVM build. The step is guarded by both route dimensions:

- `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
- `RUSTC_PROVIDER_TARGET_TRIPLE=x86_64-unknown-linux-musl`

When both match, Mantle keeps the bootstrap mrustc/minicargo C++ tools on the compiler-host environment, then sets `RUSTC_TARGET` to the host triple, emits the existing private source-root musl linker wrapper, and switches translated rustc/LLVM compiler variables to the wrapper directory:

- `CC_PROGRAM=$target_alias_dir/cc`
- `CXX_PROGRAM=$target_alias_dir/c++`
- `AR=$target_alias_dir/ar`
- `RANLIB=$target_alias_dir/ranlib`
- `CMAKE_C_COMPILER=$target_alias_dir/cc`
- `CMAKE_CXX_COMPILER=$target_alias_dir/c++`
- `CMAKE_AR=$target_alias_dir/ar`
- `CMAKE_RANLIB=$target_alias_dir/ranlib`

The setup also clears `ZLIB_CFLAGS` and `ZLIB_LIBS` only after host mrustc/minicargo are built, so those bootstrap tools can still use their host zlib while ambient GNU/glibc zlib paths do not leak into a musl LLVM build. LLVM is already configured with zlib disabled in the observed cache, so this removes an accidental host link surface from the translated rustc/LLVM phase rather than removing a required input from the bootstrap tools.

For Rust's `compiler/rustc_llvm/build.rs`, the setup copies or reuses source-root runtime archives in the private runtime directory and exports:

- `LLVM_STATIC_STDCPP=$target_runtime_dir/libstdc++.a`
- `LLVM_LINKER_FLAGS=-L$target_runtime_dir -lgcc -lunwind ...`

`LLVM_STATIC_STDCPP` drives the upstream build script's static C++ runtime handling. `LLVM_LINKER_FLAGS` supplies the GCC builtins/unwind archives that are not emitted by `llvm-config --system-libs` but are required when the final rustc link is driven with `-nodefaultlibs`.

## Scope boundaries

The private wrapper remains script-scoped. The change does not replace global `cc`, `ld`, `ld.lld`, Cargo, or host linker aliases. Routes where the compiler host is GNU keep the existing ambient compiler discovery and still prepare the musl wrapper only for target linking.

## Validation

Focused tests inspect the generated first-stage script for route guards, early wrapper ordering, CMake/compiler variable exports, zlib clearing, LLVM link hint exports, and unchanged GNU-host ordering. Real proof remains a long provider rerun; that evidence should record whether the committed code moves past the `__popcountdi2` rustc-main link frontier.
