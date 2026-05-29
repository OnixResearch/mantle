# Proposal: Harden native Rust unit executor and receipts

## Problem

Mantle's direct rustc executor now builds many units, but a real alternative needs robust cache identity, stale artifact detection, deterministic rebuild decisions, transitive search path correctness, and bounded failure diagnostics.

## Change

Harden the executor around explicit inputs and receipts. Every rustc invocation should be reproducible from declared unit material; cache hits must be proven by receipt-bound inputs and output digests; stale or missing material must fail before rustc.

## Impact

- **Files**: unit execution, receipt validation, cache/rebuild logic, diagnostics tests.
- **Testing**: stale cache negative tests, deterministic receipt replay, topology self-probes.
