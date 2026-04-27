# Tasks: Repair live-bootstrap archive status

## Audit and immediate repair

- [x] I1 Audit archived `live-bootstrap-seed-chain` and `live-bootstrap-intermediate-tools` tasks for false completion signals. [covers=Full-source bootstrap claim requires evidence]
  - Evidence: both archives mark deferred validation as `[x]`; `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, and `bootstrap/binutils-full.ncl` still print placeholder errors.
- [x] I2 Restore `bootstrap/seed-legacy.ncl` to the concrete reduced musl.cc provider. [covers=Legacy seed as development fast-path]
  - Evidence: `rg 'import "seed-legacy.ncl"' bootstrap/seed-legacy.ncl` returns no self-import after restore.

## Remaining live-bootstrap implementation

- [ ] I3 Replace placeholder pre-GCC derivations with functional chain-internal builds: `bootstrap/binutils-tcc.ncl` and all missing pre-musl/post-musl tool inputs. [covers=Full-source bootstrap claim requires evidence]
- [ ] I4 Replace placeholder GCC ladder and final provider derivations: `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, and `bootstrap/seed-full.ncl`. [covers=Full-source bootstrap claim requires evidence]

## Validation

- [ ] V1 Validate the restored legacy seed fast path without source-chain recursion. [covers=Legacy seed as development fast-path]
  - Command: `crunch build bootstrap/seed.ncl --legacy-seed` or the current equivalent selector smoke test.
- [ ] V2 Validate the full live-bootstrap chain stage by stage. [covers=Full-source bootstrap claim requires evidence]
  - Commands: build `bootstrap/stage0-posix.ncl`, `bootstrap/mes.ncl`, `bootstrap/tinycc.ncl`, `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/seed-full.ncl`.
- [ ] V3 Validate final source-built provider proof. [covers=Full-source bootstrap claim requires evidence]
  - Commands: build `bootstrap/selftest.ncl`, build `bootstrap/integration-test.ncl`, run `crunch self-build` with the source-built provider, and record byte-identical stage1/stage2 evidence.
- [ ] V4 Run OpenSpec validation and gates before archive. [covers=Full-source bootstrap claim requires evidence,Legacy seed as development fast-path]
  - Commands: `openspec validate repair-live-bootstrap-archive-status`; `openspec_gate stage=proposal`; `openspec_gate stage=design`; `openspec_gate stage=tasks`.
