# Design: first-stage musl rustc-driver rlib

## Context

The first-stage Rust source provider materializer generates a shell script that builds mrustc, translates Rust 1.90 rustc/cargo, then runs mrustc's `run_rustc` makefile. The source-root musl-host route sets both compiler host and provider target to `x86_64-unknown-linux-musl`, uses the private musl target linker wrapper, and asks `run_rustc` to use `DYLIB_EXT=rlib` because regular target dylibs are unsupported.

The remaining blocker is in Rust's own compiler manifest: `compiler/rustc_driver/Cargo.toml` requests `crate-type = ["dylib"]`. Cargo rejects that before invoking rustc for the musl compiler-host target.

## Decisions

### 1. Patch the extracted Rust source in the first-stage script

**Choice:** The generated first-stage script rewrites the already-extracted `rustc-${RUSTC_VERSION}-src/compiler/rustc_driver/Cargo.toml` after mrustc source preparation and before `run_rustc`.

**Rationale:** The patch belongs to the ephemeral first-stage source tree, not the checked-in route plan or upstream archive. Running it after mrustc's source preparation avoids racing the archive extraction and keeps the actual built source visible in the scratch tree for evidence.

### 2. Guard the patch to the musl compiler-host route

**Choice:** The rewrite only runs when `RUSTC_HOST_TRIPLE` is `x86_64-unknown-linux-musl`.

**Rationale:** GNU-host routes can use the upstream dylib driver shape and should not be changed. Target-only musl sysroot builds do not build `rustc_driver`; only the compiler-host route needs this normalization.

### 3. Fail closed on unexpected manifest shape

**Choice:** The shell rewrite accepts exactly `crate-type = ["dylib"]` or an already-normalized `crate-type = ["rlib"]`; any missing manifest or missing expected line aborts the first-stage build.

**Rationale:** Source patch drift must be reviewable. Silently continuing would either recreate the known Cargo error or mask an upstream Rust layout change.

## Risks / Trade-offs

- This only clears the `rustc_driver` crate-type blocker. Scratch continuation has already shown the next frontier is dynamic proc-macro loading for musl-host proc macros, so this change does not claim full provider success.
- The generated patch is shell-based to stay within the existing first-stage materializer script. Tests assert exact fragments so the shell stays deterministic and auditable.
