## Why

`crunch bootstrap parity-report` still classifies `binutils.tcc` as a live-bootstrap/Guix placeholder because the row only records that `bootstrap/binutils-tcc.ncl` exists. Downstream GCC parity should not advance until the binutils bridge has reproducible assembler/linker/runtime evidence and the report can distinguish real evidence from bridge placeholders.

## What Changes

- **Evidence contract**: Define the minimal transcript required to promote `binutils.tcc` out of placeholder status.
- **Validation path**: Add a bounded task plan for evaluating/building `bootstrap/binutils-tcc.ncl` and smoke-testing produced tools.
- **Report guard**: Keep parity fail-closed unless the transcript proves tool output and no host fallback.

## Capabilities

### Modified Capabilities
- `bootstrap.binutils.tcc.chain`: Requires concrete assembler/linker smoke evidence before parity promotion.
- `bootstrap.parity.map`: Requires the `binutils.tcc` row to remain blocked unless evidence is present and checked.

## Impact

- **Files**: `src/bootstrap_parity.rs`, `tests/bootstrap_parity_cli.rs`, `bootstrap/binutils-tcc.ncl`, `openspec/specs/bootstrap/spec.md`, and evidence files under this change.
- **APIs**: No public API change; CLI JSON row semantics may gain stricter evidence fields/status.
- **Dependencies**: No new dependencies expected.
- **Testing**: `openspec validate --all --strict`, `cargo test -p crunch --bin crunch bootstrap_parity`, `cargo test -p crunch --test bootstrap_parity_cli`, and a bounded `crunch build bootstrap/binutils-tcc.ncl`/tool smoke when prerequisites are available.
