# Tasks: Evaluate VectorCDC chunk profiles

## Specification

- [x] S1 Create proposal, design, tasks, and delta spec for the evaluation track. [covers=delta-transfer.vectorcdc-evaluation]

## Implementation / Evidence

- [x] I1 Inventory current FastCDC chunking entrypoints, constants, tests, and delta negotiation call sites. Evidence: `design.md`. [covers=delta-transfer.vectorcdc-evaluation.boundary]
- [x] I2 Add a pure deterministic chunker boundary with FastCDC as the default implementation and invariant tests for ordered contiguous full coverage. Evidence: `verification.md`. [covers=delta-transfer.vectorcdc-evaluation.boundary]
- [x] I3 Build a representative benchmark corpus from local store/build artifacts plus synthetic small-delta mutations, with recorded corpus provenance. Evidence: `evidence/i3-corpus-manifest.json`. [covers=delta-transfer.vectorcdc-evaluation.evidence]
- [x] I4 Capture baseline FastCDC evidence: CDC throughput, total ingest wall time, chunk-size distribution, chunk count, dedup/reuse ratio, object-count impact, and separated BLAKE3/zstd/object-store costs where practical. Evidence: `evidence/i4-fastcdc-baseline.json`. [covers=delta-transfer.vectorcdc-evaluation.evidence]
- [ ] I5 Prototype one hashless/VectorCDC-style candidate behind an explicit feature/config gate with scalar fallback or unsupported-platform fail-closed behavior. [covers=delta-transfer.vectorcdc-evaluation.optional-deps]
- [ ] I6 Compare candidate metrics against FastCDC and record an adoption decision: keep FastCDC, tune FastCDC, continue research, or open a separate promotion OpenSpec. [covers=delta-transfer.vectorcdc-evaluation.promotion]

## Verification

- [x] V1 Run strict OpenSpec validation for this change. Evidence: `verification.md`. [covers=delta-transfer.vectorcdc-evaluation]
- [x] V2 Run focused Rust/Nix checks for any implementation slice before marking implementation tasks complete. Evidence: `verification.md`. [covers=delta-transfer.vectorcdc-evaluation]
