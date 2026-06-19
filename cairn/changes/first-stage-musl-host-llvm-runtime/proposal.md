# first-stage musl host LLVM runtime

## Problem

The source-root musl route now reaches the `rustc-main` link, but the LLVM objects were built before Mantle prepared the private musl target compiler wrappers. CMake therefore used the ambient GNU/Nix C++ compiler and glibc-flavored feature probes, while the final `rustc` link used the source-root musl linker wrapper. The mixed host/runtime surface fails at linker builtins such as `__popcountdi2` and risks carrying host C++ runtime assumptions into the musl compiler host.

## Proposed change

Prepare the private source-root musl compiler wrapper after the host mrustc/minicargo bootstrap tools are built, but before the first translated `output/rustc`/LLVM build, whenever the route uses `x86_64-unknown-linux-musl` for both host and provider target. Use those private `cc`, `c++`, `ar`, and `ranlib` wrappers for LLVM CMake, disable ambient glibc zlib flags in that route, and export Rust's `rustc_llvm` link hints so the final `rustc` link resolves the source-root musl `libstdc++` and GCC builtins from the private runtime directory.

Keep the behavior route-local. Generic host aliases (`cc`, `ld`, `ld.lld`) remain host-oriented, and the default GNU-host Rust source route remains unchanged.

## Success criteria

- Generated first-stage scripts build host mrustc/minicargo first, then prepare the source-root musl target wrapper before the first `minicargo.mk output/rustc` build on musl-host routes.
- LLVM CMake uses private source-root musl `cc`/`c++`/`ar`/`ranlib` wrappers and does not inherit ambient glibc zlib flags on that route.
- `LLVM_STATIC_STDCPP` and `LLVM_LINKER_FLAGS` point at the private runtime directory so `rustc_llvm` links source-root musl C++ and GCC runtime archives.
- Focused tests, formatting, diff checks, and Cairn validation/gates record the generated-script invariants.
- A fresh real provider rerun is queued or captured to prove the next source-built Rust frontier from committed code.
