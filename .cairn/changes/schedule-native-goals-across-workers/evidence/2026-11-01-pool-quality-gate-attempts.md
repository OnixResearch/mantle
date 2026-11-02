# Ordinary quality-gate attempts for pool work (2026-11-01)

Scope: record what `./scripts/check-first-party-quality.sh` proved for the pool
changes and which failures were environmental or pre-existing. A green full run
is still pending; each attempt reached a new leg.

## Environment required by the gate

- `HOME` must be a writable user home (`/homeless-shelter` breaks embedded
  stdlib extraction).
- `git` and `which` must be on `PATH` (fetcher tests).
- `x86_64-unknown-linux-musl-gcc` and `ld.lld` must be on `PATH`
  (`cargo_free_self_build_cli`).
- `MANTLE_WASM_COMPONENT_TOOLCHAIN` must point at the pinned toolchain store
  path (`wasm_component_cli`).
- The ambient environment must stay under 64 KiB for the incremental
  fingerprint tests; the devshell env is 87 KiB. Dropping the `*_FOR_BUILD`,
  `pkgsBuildHost`, `nativeBuildInputs`, `HOST_PATH`, `initialPath`, and
  `PYTHONPATH` entries brings it to 52 KiB while keeping host `NIX_CFLAGS_COMPILE`
  and `NIX_LDFLAGS` for C rebuilds.

## Real defect found and fixed

- `operator_diagnostics::terminal_report_output_failure_is_a_cli_error_not_a_panic`
  failed with `operator command contract: flag-drift: build`: the reviewed
  operator surface never included `--builder-pool`, `--builder-config`, or
  `--builder-ticket-fd`. Fixed by adding the flags to
  `config/operator-surfaces.ncl` and regenerating the descriptors, inventory
  JSON, catalog, and command reference (`9378392cc`). The contract check now
  passes with 214 commands.

## Pre-existing test defect fixed

- `mantle-rust-differential`'s wrapper-isolation test set `HOME`,
  `CARGO_HOME`, and `TMPDIR` to private subdirectories but created only the
  root, so the linker's `mktemp` failed. The test now creates those private
  directories; it passes.

## Green runs

`./scripts/check-first-party-quality.sh` completed with exit 0 twice:

- after 6,280 s on the tree before the session-concurrency and fallback work:
  226 `test result: ok` lines, zero failures, gcc40 bridge PASS;
- after 6,349 s on `a827237ff` (fallback, session bounds, cache hit, output
  matching, operator contract): 224 `test result: ok` lines, zero
  `test result: FAILED`, gcc40 bridge PASS.

Both runs used the environment listed above and the two test fixes
(`9378392cc`, `dc2d1c08d`).

## Notes

- Earlier attempts stopped at `cargo_free_self_build_cli`, `wasm_component_cli`,
  `operator_diagnostics`, and `mantle-rust-differential`; the first two were
  missing tools, the third was the real operator-contract drift, and the fourth
  was a pre-existing test defect.
- The gate is not runnable without the toolchain additions; a bare devshell
  environment fails before reaching the pool-related tests.
