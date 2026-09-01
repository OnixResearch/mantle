# Observation v9 open-file and GCC pass3 relocation boundaries

## Result

Fresh strict observation v9 constructed and published the protected StageX provider. The native closure then failed at `musl-1.2.5-gcc10-v1`.

The preserved failed attempt is in `attempt-status.json`. Its old v1 plan digest and old source profile do not support a successful proof claim.

## Open-file boundary

A replay with the inherited soft `RLIMIT_NOFILE` of 1,024 reproduced GNU `ar` failing while it opened the large musl object set. The proof must not depend on an ambient shell `ulimit`.

The v2 proof plan now binds `open_file_descriptors_max = 4,096`. On Linux, the shell checks the hard limit, sets the soft limit exactly before StageX starts, preserves the hard limit, and verifies the result. It fails before construction when the hard limit is lower or the kernel does not apply the requested value.

With a soft limit of 4,096, the failed archive boundary completed and exposed the next independent blocker.

## GCC pass3 relocation boundary

`gcc-4.0.4-musl-pass3-v22` installed an `out/bin/ld` wrapper that named its ephemeral build-time `$BINUTILS/bin/ld`. The later musl build could not resolve that scratch path.

Version v23 installs a relocation-stable wrapper. It uses only explicit `MANTLE_GCC_LINKER` authority or the logical source-built StageX linker at `$STAGEX/bin/x86_64-linux-musl-ld`. Pass4 consumers now require v23.

Pueue task `7906` rebuilt the updated pass3-to-musl handoff in the preserved offline state. The report recorded strict hermeticity, no hermeticity audit events, one built root, no cache hit, and no failure. It published:

- `/mantle/store/7z4v7r09gn10q1xas1fvjvndc2bsx0s0-gcc-4.0.4-musl-pass3-v23`
- `/mantle/store/788c8z8dn2883ixmi5wr151h8zwlmbmf-musl-1.1.24-gcc-pass4-v2`

## Validation

Focused plan, shell, bootstrap-contract, Clippy, formatting, and diff checks passed in pueue task `7905`. The diagnostic build passed in task `7906`.

## Non-claims

This evidence proves only the bounded resource and relocation repairs above. The successful diagnostic reused the failed observation state to shorten replay. It is not a fresh native-provider proof. It does not prove the full native provider, the Rust provider, Mantle stage1, fixed-point convergence, compiler correctness, or release eligibility.
