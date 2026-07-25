## Context

The StageX CLI path validates lineage input and classifies evidence, but it does not construct a provider. The source graph already contains hex0, M0/M1/hex2/kaem, Mes, TinyCC, musl, GNU tools, GCC, and final provider stages. The missing boundary is trustworthy orchestration and evidence: current Nickel builders can name `/bin/sh` and `/bin/busybox`, while the protected self-build contract rejects host-bwrap and executable fallback for StageX completion.

## Decisions

### Decision: separate environmental execution authority from produced lineage

**Choice:** The receipt must enumerate kernel and initial Mantle-orchestrator assumptions separately from seed bytes and produced tools. No host executable bytes may enter claimed outputs or stage PATH unless they are the audited seed, a declared stage output, or an explicitly BLAKE3-bound pre-protection launcher retired before the claimed transition.

**Rationale:** Treating the host kernel as an assumption is honest; silently treating host shell or compiler output as source-built is not.

### Decision: establish a one-way protected transition

**Choice:** Materialization first verifies the seed and source authority, builds/selects the StageX transition runner and sandbox tools, then installs protected execution. After the transition, executable access is allowlisted by absolute path and digest, fallback events are forbidden, and the process cannot return to ambient discovery.

**Rationale:** A monotonic transition makes the audit reviewable and prevents late host-tool escape.

### Decision: derive receipts from observed stage execution

**Choice:** The final receipt is produced from stage reports and observed protected-exec events, not from a checked-in template. It records real audited-seed, manifest, stage-graph, source-state, provider, output, and audit BLAKE3 values and `lineage_receipt_status = complete`.

**Rationale:** Digest-shaped scaffold values are useful for parser tests but are not evidence.

### Decision: publish only after independent revalidation

**Choice:** Materialize under a private staging directory, validate the provider contract and lineage receipt independently, then publish create-new. Any source, stage, output, fallback, or audit mismatch leaves the existing scaffold and selected provider unchanged.

**Rationale:** Partial output must never become bootstrap authority.

## Risks / Trade-offs

- Early stages may genuinely require host shell/sandbox help; the claim must either construct and transition away from it or retain an explicit blocker.
- Seccomp user-notification support is platform-specific; unsupported kernels fail closed for StageX proof.
- The full lineage is long-running and needs resumable authenticated stage outputs without accepting stale evidence.