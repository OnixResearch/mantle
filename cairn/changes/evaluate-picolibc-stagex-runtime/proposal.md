## Why

StageX currently builds a bounded native-musl runtime from authenticated musl 1.1.24 source. This path compiles 746 self-host sources and 19 predecessor sources.

Picolibc provides a static x86_64 Linux profile and narrow operating-system hooks. These properties can reduce the early C runtime surface.

Small embedded outputs do not prove a smaller bootstrap trust surface. Mantle needs a bounded comparison before any provider or parity change.

## What Changes

- Add a research-only Picolibc static Linux diagnostic outside the protected StageX authority.
- Pin source, configuration, build-tool, compiler, linker, license, and output identities.
- Run the current StageX libc behavior contract against two isolated Picolibc builds.
- Emit a deterministic comparison report against the current native-musl baseline.
- Classify the result as `candidate`, `rejected`, or `blocked` through a pure decision core.
- Record the result in an ADR and an oracle checkpoint with explicit non-claims.

Provider selection, bootstrap parity, and the final musl toolchain remain unchanged.

## Impact

- **Files**: diagnostic Nickel under `bootstrap/`, one Rust comparison script, focused fixtures, lifecycle evidence, one ADR, and the README reference list.
- **Testing**: positive runtime tests, malformed-input tests, source and tool tamper tests, two isolated builds, comparison self-tests, source-pin checks, and Cairn gates.
