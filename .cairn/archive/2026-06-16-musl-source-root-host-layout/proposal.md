## Why

The current Rust provider host is GNU, which honestly needs a source-built GNU-compatible host C/linker/libc root. A better route for the full zero-seed closure is a future musl-host Rust provider: when the Rust provider host triple is `x86_64-unknown-linux-musl`, the existing source-root musl toolchain can be the host C/linker/runtime root as well as the target root.

## What Changes

- Teach native closure capability classification that source-root musl metadata may satisfy a host root only when the expected Rust provider host triple is the same musl target.
- Map host-root member collection through the source-root layout (`bin/x86_64-linux-musl-gcc`, `bin/x86_64-linux-musl-ld`, `<target>/lib/...`) when that capability is selected.
- Keep GNU-host Rust providers rejected before host member collection when given a target-only musl source root.

## Impact

- **Files**: native closure materializer shell/core tests, Cairn spec/evidence.
- **Validation**: focused positive/negative unit tests, formatting/whitespace checks, and a real current-provider frontier transcript proving the GNU-host rejection still holds.
