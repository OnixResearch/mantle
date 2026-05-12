## Context

The current parity map has a concrete `binutils.tcc` row, but `src/bootstrap_parity.rs` marks it `expected_complete: false` with `EvidenceCheck::None`. The live report sees placeholder/bridge text in `bootstrap/binutils-tcc.ncl`, so the row blocks live-bootstrap and Guix parity. That is correct, but the next increment needs a narrow evidence contract and implementation path rather than another broad bootstrap attempt.

## Goals / Non-Goals

**Goals:**
- Specify the exact evidence needed for `binutils.tcc` parity promotion.
- Add a deterministic check or fixture so the parity report cannot promote unevidenced bridge output.
- Run the smallest feasible build/smoke probe and record any bounded blocker.

**Non-Goals:**
- Proving GCC 4.0/4.7/10 correctness.
- Claiming full live-bootstrap, Guix, or StageX parity.
- Reworking the whole binutils derivation in one drain if the build exceeds the local budget.

## Decisions

### 1. Evidence-first parity promotion

**Choice:** Keep `binutils.tcc` fail-closed until a checked transcript proves the exact produced tools.

**Rationale:** A derivation's presence and bridge comments are not semantic evidence. Binutils is an upstream dependency for GCC transition claims, so premature promotion would contaminate downstream rows.

**Alternative:** Mark the row complete when placeholder text is removed. Rejected because removing markers does not prove assembler/linker behavior or host-fallback absence.

**Implementation:** Add a row-specific evidence check or fixture-backed detector in the parity report, and update CLI tests to cover missing evidence and accepted evidence if promotion is implemented.

### 2. Bounded build transcript

**Choice:** Treat a bounded `crunch build bootstrap/binutils-tcc.ncl` plus tool smokes as the required runtime evidence, with blocker classification if prerequisites exceed the drain budget.

**Rationale:** The bootstrap chain can exceed one agent window; preserving a transcript and explicit blocker is more useful than unbounded retries.

**Alternative:** Run full downstream GCC immediately. Rejected because GCC evidence depends on binutils first and would blur the blocker boundary.

## Risks / Trade-offs

**Long build time** → Use `timeout`, local scratch store, and record the latest Crunch log path/tail if the build does not complete.

**False-positive placeholder detection** → Require standalone marker detection and regression tests, reusing the existing heredoc/terminator guard pattern.

**Host leakage** → Tool smokes must identify the output tool path and inspect fallback markers; host `/usr/bin/as`/`ld` must not satisfy the row.

## Validation Plan

- `openspec validate --all --strict`
- `cargo test -p crunch --bin crunch bootstrap_parity -- --nocapture`
- `cargo test -p crunch --test bootstrap_parity_cli -- --nocapture`
- `cargo run -q -p crunch -- --json bootstrap parity-report`
- If build prerequisites fit the budget: `crunch build bootstrap/binutils-tcc.ncl --store .crunch-drain/store -j 4 --trust-unsigned` followed by `as`/`ld`/`ar`/`ranlib`/`nm`/`objcopy` smokes from the output path
