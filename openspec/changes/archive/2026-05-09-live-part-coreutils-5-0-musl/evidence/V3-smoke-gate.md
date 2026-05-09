# V3 coreutils 5.0 musl smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.coreutils.5.0.musl

The smoke contract is conditional on a produced `coreutils-5.0-musl` output. Because V2 has no output path while prerequisites remain blocked or gated, no installed coreutils smoke success is claimed.

The hardened derivation now fails closed unless all declared utilities and the file-operation smoke are produced during a real build.
