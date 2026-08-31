# ADR 0091: Launch restored rustc through the bound loader

## Status

Accepted (2026-08-31)

## Context

V88 restored the promoted checkpoint and passed the bound Rust sysroot
relocation from ADR 0090. The next receipt-bound rustc compatibility probe
failed.

The restored `rustc.dynamic` declares `libstdc++.so.6` and `libc.so` as needed
libraries. The Rust-provider runtime directory contains `libc.so` and
`libgcc_s.so.1`, but no C++ runtime.

The provider's shell wrapper appends inherited `LD_LIBRARY_PATH`. The promoted
proof correctly supplies no ambient library path, so the loader reported
missing C++ symbols such as `__cxa_pure_virtual`.

The full-source binding already records exact native-provider artifacts for
`libstdc++.so.6.0.28` and `ld-musl-x86_64.so.1`.

## Decision Drivers

- Use only receipt-bound runtime bytes.
- Do not forward ambient `LD_LIBRARY_PATH`.
- Keep the published checkpoint immutable.
- Use the bound shell and dynamic loader as explicit executable authority.
- Preserve the existing rustc sysroot behavior.
- Fail if the C++ soname does not resolve to the bound versioned file.
- Keep generated runtime authority visible to action planning.

## Decision

Select exactly one bound C++ runtime artifact and one bound dynamic-loader
artifact from the relocated full-source binding.

For both artifacts, require an absolute regular file and its exact BLAKE3.
Require the loader to be executable. Require both artifacts to use the same
runtime directory. Require `libstdc++.so.6` to resolve to the bound versioned
C++ runtime.

Generate a proof-local rustc wrapper with these properties:

1. Its interpreter is the receipt-bound BusyBox `sh`.
2. It derives the restored Rust-provider root from the bound rustc path.
3. It requires the exact `rustc.dynamic` and runtime directories.
4. It constructs `LD_LIBRARY_PATH` only from restored Rust-provider directories
   and the bound native runtime directory.
5. It preserves an explicit caller `--sysroot`.
6. It otherwise adds the restored provider as `--sysroot`.
7. It directly executes the bound musl loader with `rustc.dynamic`.

The generated wrapper becomes the selected rustc authority. The original rustc
remains linked through the closure identity. The dynamic loader becomes a fixed
native-helper authority for protected Rust child actions.

Do not add ambient `LD_LIBRARY_PATH` to the Rust topology environment allowlist.

## Alternatives Considered

### Forward ambient `LD_LIBRARY_PATH`

Rejected. This would add an unreviewed host dependency and could select
unbound libraries.

### Copy libstdc++ into the restored checkpoint payload

Rejected. This would mutate the immutable checkpoint beyond its recorded
binding relocation.

### Rebuild and republish the Rust provider

Rejected for this boundary. The binding already carries the exact runtime and
loader needed by the verified checkpoint.

### Execute the restored rustc shell wrapper directly

Rejected. Its `#!/bin/sh` interpreter is ambient, and the protected action path
must use the receipt-bound shell.

## Consequences

- Restored rustc starts without ambient dynamic-library state.
- The compatibility probe and later Rust units use one generated wrapper.
- Protected execution observes the generated wrapper and bound loader.
- A missing loader, runtime, soname link, digest, mode, or provider directory
  fails before stage execution.
- V88 remains failed evidence. A fresh promoted proof must verify this route.
