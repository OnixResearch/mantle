Task-ID: V3
Covers: Full-source bootstrap claim requires evidence

# Source-built proof deferral

Status: deferred to `openspec/changes/live-bootstrap-source-chain/`.

The future proof task must record all fields required by
`bootstrap.fullsource.claim.evidence`:

- provider kind (`source-root` or StageX-class source-built provider, not legacy)
- manifest digest
- provider output digest
- proof bundle digest
- stage1/stage2 byte-identical result
- docs update separating remaining trusted roots from eliminated binary-provider trust

This repair slice intentionally does not claim those fields exist. Until the
successor records fresh transcripts, full-source bootstrap status remains
blocked.
