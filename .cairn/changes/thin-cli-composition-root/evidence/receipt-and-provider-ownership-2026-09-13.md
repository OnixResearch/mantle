# Receipt construction and provider translation ownership

Change: `thin-cli-composition-root`
Task-ID: I3 (categories: receipt construction, provider translation)
Subject revision: `31ad1ef57` (code) plus this prose-only evidence commit.

## Question

I3 names receipt construction and provider translation as categories that must
leave `main.rs`. Before extracting anything, measure where each category lives.

## Inspected evidence

Counts are `grep -c` over the tree at the recorded revision.

Receipt-shaped code:

| File | `Receipt {` occurrences |
| --- | --- |
| `src/rust_plan.rs` | 35 (owner of the plan and execution receipts) |
| `src/main.rs` | 15 |
| `src/full_source_rust_binding.rs` | 9 |
| `src/rust_source_provider.rs` | 6 |
| `src/nickel_export_core_adapter.rs` | 5 |
| `src/full_source_rust_binding_shell.rs` | 5 |

The 15 root occurrences are projections: `execute_*_rust_plan` wraps a plan
receipt with an execution result and calls the module's printer, for example
`rust_plan::RustPlanExecutionReceipt { rust_plan: request.receipt, unit_execution }`.
No root code decides receipt contents, identity, or signing.

Attestation sidecars:

| File | occurrences |
| --- | --- |
| `src/remote_build.rs` | 64 |
| `crates/crunch-attestation-core/src/canonical.rs` | 30 |
| `src/full_source_rust_binding_shell.rs` | 28 |
| `src/build_report.rs` | 24 |
| `src/early_native_row_receipt.rs` | 18 |

Provider translation: the source-root flow moved to
`src/source_root_provider_flow.rs` in this session, and the trust material it
depends on moved to `src/trusted_keys.rs`.

## Decision

- **Receipt construction is already owned by modules.** The category is
  satisfied for construction policy; what remains in the root is bounded
  projection, which is composition-root work.
- **Provider translation is satisfied** by the flow and trust adapters named
  above.

## Owner

Mantle CLI composition root (`src/main.rs`) keeps dispatch and projection;
`src/rust_plan.rs` and the attestation modules own receipt contents.

## Next action

The remaining root-side receipt code is 14 projection wrappers. Extract them
into one module so the composition root holds only dispatch, then close I3's
category list. This is a follow-up slice, not a blocker.

## Non-claims

This note measures ownership of receipt-shaped code. It does not claim receipt
correctness, signing behaviour, or that I3 as a whole is complete.
