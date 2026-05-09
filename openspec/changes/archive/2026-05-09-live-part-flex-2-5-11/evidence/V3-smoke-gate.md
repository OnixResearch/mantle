# V3 flex 2.5.11 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.flex.2.5.11

The smoke contract is conditional on a produced `flex-2.5.11-musl` output. Because V2 has no output path while prerequisites remain blocked or gated, no installed flex smoke success is claimed.

The hardened derivation now fails closed unless `bin/flex`, `bin/lex`, and the installed version smoke are produced during a real build.
