# V3 coreutils 6.10 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.coreutils.6.10

The smoke contract is conditional on a produced `coreutils-6.10-musl` output. Because V2 has no output path while prerequisites remain blocked or gated, no installed coreutils smoke success is claimed.

The hardened derivation now fails closed unless all declared utilities and the file/mktemp/sha256sum smoke are produced during a real build.
