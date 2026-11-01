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

## Remaining

A full green run is still pending. The last attempt stopped at
`mantle-rust-differential`; earlier attempts stopped at `cargo_free_self_build_cli`
and `wasm_component_cli`, all addressed above. The run takes roughly 100-110
minutes in this workspace.
