# V3 binutils 2.41 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.binutils.2.41

The smoke contract is conditional on a produced `binutils-2.41-full` output. Because V2 has no output path while prerequisites remain blocked or gated, no installed binutils smoke success is claimed.

The hardened derivation now fails closed unless required tool entrypoints and smoke artifacts are produced during a real build.
