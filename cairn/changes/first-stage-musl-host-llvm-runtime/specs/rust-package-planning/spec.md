## ADDED Requirements

### Requirement: First-stage musl host LLVM runtime coherence

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_host_llvm_runtime] Mantle MUST build the first-stage source-root musl compiler host with a coherent musl LLVM/C++ runtime surface.

#### Scenario: LLVM host build uses private source-root musl wrappers

GIVEN the generated first-stage Rust provider script is building with `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
AND `RUSTC_PROVIDER_TARGET_TRIPLE=x86_64-unknown-linux-musl`
WHEN the script prepares the first mrustc/LLVM build
THEN it MUST create the private source-root musl wrapper directory before invoking the first `minicargo.mk output/rustc` build.
AND it MUST set the first-stage C, C++, archive, ranlib, and CMake compiler variables to private source-root musl wrapper paths.

#### Scenario: LLVM runtime link hints use source-root musl archives

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN Rust's `rustc_llvm` build script links static LLVM into `rustc`
THEN Mantle MUST export `LLVM_STATIC_STDCPP` pointing at the private source-root musl `libstdc++.a` archive.
AND Mantle MUST export `LLVM_LINKER_FLAGS` with the private runtime directory and GCC builtins/unwind archive link flags.

#### Scenario: host leakage stays bounded

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN it switches LLVM and first-stage C/C++ builds to the private musl wrappers
THEN it MUST clear ambient GNU/glibc zlib CFLAGS and LDFLAGS for that route.
AND it MUST NOT replace global `cc`, `ld`, `ld.lld`, Cargo, or host linker aliases with source-root musl target tools.

#### Scenario: GNU-host route remains unchanged

GIVEN the generated first-stage Rust provider script is building with a GNU compiler host and a musl provider target
WHEN it prepares the first mrustc/LLVM build
THEN it MUST continue using the compiler-host linker for the host build.
AND it MUST prepare the private source-root musl wrapper only for later target linking.
