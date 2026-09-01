# V85 ptrace Rust-provider aggregate-bound blocker

## Outcome

V85 ran the promoted proof from commit `dc96b465` with the acknowledged
ptrace root handshake. It used strict hermeticity, no substitution, 16 jobs,
and the unchanged 700 GB disk bound.

The source profile was `Ready` with BLAKE3
`464632a236ffdfa3c2d01eb81add6dc857cc408f140acbe7fd3f9478b6d0b9d2`.
The release binary had exact transfer parity and BLAKE3
`8f9e89e6b6a8d92f91ba6d0072110ada5199955d9a79db63201a8430e0c0ee09`.
The isolated V61 native provider revalidated with BLAKE3
`63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`.

All five Rust-provider stages completed and reconciled:

| Stage | Matched events | Denied | Promotions |
|---|---:|---:|---:|
| MRustC to Rust 1.90.0 | 88,038 | 0 | 177 |
| Rust 1.91.1 | 75,843 | 0 | 147 |
| Rust 1.92.0 | 76,071 | 0 | 156 |
| Rust 1.93.1 | 75,272 | 0 | 158 |
| Rust 1.94.0 | 75,983 | 0 | 174 |
| **Total** | **391,207** | **0** | **812** |

Each stage was local-only and reported zero blockers. V85 therefore proves
that the ptrace mechanism crossed both prior failures:

- the second-root `PTRACE_SETOPTIONS` race from V84;
- the Rust 1.93.1 spurious `EACCES` boundary from V81 and V82.

The proof failed after Rust 1.94.0 because aggregate construction compared
391,207 events against the single-stage limit of 262,144.

## Root cause

`exec_events_per_stage_max` is a stage limit. Stage reconciliation enforced it
correctly. Aggregate reconciliation then reused it as a total-provider limit,
even though the authenticated route declares five bounded stages.

ADR 0088 keeps the per-stage limit and derives the aggregate limit through
checked multiplication:

```text
exec_events_per_stage_max * declared_stage_count
```

For V85, the derived maximum is 1,310,720. The observed 391,207 events stay
inside that bound. Zero factors and arithmetic overflow remain fail-closed.

## Repair validation

The focused post-repair checks passed:

- Rust-provider action runtime: 3 passed;
- Rust action shell: 2 passed;
- Rust source provider: 121 passed.

The positive aggregate test binds V85's five-stage count and 391,207 observed
events. Negative checks reject zero factors and checked-multiplication
overflow. Focused Clippy reported no findings in either touched source file;
its 26 warnings are unchanged findings elsewhere in the root binary.

## Preserved attempt

The wrapper stopped at `2026-08-30T13:00:33-04:00` with exit code 1. The
staging tree remains at:

```text
/home/brittonr/mantle-runs/receipt-fix-v31/.source-built-fixed-point-v85-ptrace-acknowledged-20260830.source-built-fixed-point-staging-2241078
```

## Evidence

This directory preserves:

- detached launch, host, wrapper, transfer, profile, and proof records;
- fixed-point, aggregate-authority, and per-stage plans;
- all five raw ptrace audits and reconciliations;
- all five compressed build logs;
- candidate, source, build, and smoke reports for each Rust stage;
- native-provider import and revalidation records.

## Non-claims

V85 does not prove aggregate Rust-provider acceptance, checkpoint publication,
fixed-point equality, final receipt validity, or complete trust. Its completed
Rust stages remain failed-attempt evidence until a fresh promoted run succeeds.
