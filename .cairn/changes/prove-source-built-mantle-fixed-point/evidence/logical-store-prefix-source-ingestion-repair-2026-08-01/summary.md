# Logical store-prefix source-ingestion repair

## Result

Mantle now preserves the configured logical store prefix during source existence checks, source-closure ingestion, sandbox input resolution, and missing-source diagnostics.

The original diagnostic declared:

```text
/mantle/store/ki5gkg5d6si77dl5k4mav4s6x9s8l25r-mantle-stagex-transition
```

but reported the source as missing under `/nix/store`.

Pueue task `7235` rebuilt `stage0-posix` in a fresh physical store containing only the retained StageX transition source authority. The command used logical prefix `/mantle/store`, strict hermeticity, no substitution, and `CRUNCH_NO_FUSE=1`. Its build report records one built success, no cache hit, and no failure.

Pueue task `7244` then replayed `bootstrap/binutils-full.ncl`. The old missing-source error did not recur. The replay advanced to the independent `mes` runtime blocker:

```text
/bin/sh: kaem: not found
```

The root report now identifies `dependency binutils-2.30-gcc-pass4-v16.drv failed` rather than a StageX source-path failure.

## Root cause

Four source-ingestion paths reconstructed source locations with the default `/nix/store` prefix:

1. declared-source existence checks;
2. physical-to-logical source fallback;
3. sandbox input resolution;
4. `SourceNotFound` display formatting.

Transitive source-closure members could also receive built-output lookup semantics because sandbox input resolution checked only the derivation shape, not the complete resolved source set.

## Repair

`crates/crunch-build/src/orchestrate.rs` now:

- checks the configured physical output authority first;
- falls back to the configured logical store prefix;
- applies source lookup semantics to every path in the resolved source set;
- carries the configured prefix into missing-source errors.

`crates/crunch-build/src/error.rs` now formats `SourceNotFound` with its explicit `store_dir` authority.

Positive and negative tests cover logical fallback, physical precedence, declared-source lookup, transitive source ingestion, missing-source diagnostics, and `/nix/store` compatibility.

## Validation

- Pueue task `7247`: two custom-prefix source tests passed.
- Pueue task `7203`: two `SourceNotFound` display tests passed.
- Pueue task `7221`: all 27 `crunch-pipeline` library tests passed.
- Pueue task `7252`: strict first-party `crunch-build` library Clippy passed; `git diff --check` passed.
- Pueue task `7235`: direct Stage0 replay built successfully under `/mantle/store`.
- Pueue task `7244`: binutils replay moved beyond the original source-prefix failure to the independent `mes` blocker.

The broad `crunch-build` library run in task `7208` passed 649 tests and failed five existing fetchGit tests after the remote git-daemon test inherited its intentionally poisoned PATH. The first failure poisoned the shared test mutex. No touched source-prefix test failed.

## Non-claim

This repair proves the bounded logical-prefix and source-ingestion behavior above. The binutils replay is still failed evidence. It does not prove the native provider, Rust provider, Mantle fixed point, compiler correctness, or release eligibility.
