## Context

The StageX CLI path validates lineage input, but it does not construct a provider. The protected transition now reaches self-hosted TinyCC, native musl, GNU tools, and authenticated binutils. This is enough for an intermediate provider. Final GCC stages and provider admission remain separate claims.

## Decisions

### Decision: separate environmental execution authority from produced lineage

**Choice:** The receipt must enumerate kernel and initial Mantle-orchestrator assumptions separately from seed bytes and produced tools. No host executable bytes may enter claimed outputs or stage PATH unless they are the audited seed, a declared stage output, or an explicitly BLAKE3-bound pre-protection launcher retired before the claimed transition.

**Rationale:** Treating the host kernel as an assumption is honest; silently treating host shell or compiler output as source-built is not.

### Decision: establish a one-way protected transition

**Choice:** Materialization first verifies the seed and source authority, builds/selects the StageX transition runner and sandbox tools, then installs protected execution. After the transition, executable access is allowlisted by absolute path and digest, fallback events are forbidden, and the process cannot return to ambient discovery.

**Rationale:** A monotonic transition makes the audit reviewable and prevents late host-tool escape.

### Decision: derive receipts from observed stage execution

**Choice:** The final receipt is produced from stage reports and observed protected-exec events, not from a checked-in template. It records real audited-seed, manifest, stage-graph, source-state, provider, output, transition-report, audit, provider-validation-report, final-bundle, and domain-separated receipt-payload BLAKE3 values and `lineage_receipt_status = complete`.

**Rationale:** Digest-shaped scaffold values are useful for parser tests but are not evidence.

### Decision: use explicit publication inputs

**Choice:** Require an absolute lineage-manifest path, an absolute complete transition root, and an absent absolute output path. Do not infer transition artifacts from the checkout, environment, output path, or store.

**Rationale:** Explicit authority prevents stale-root discovery and keeps publication reviewable.

### Decision: publish only after independent revalidation

**Choice:** Materialize under a private create-new staging directory, validate the provider contract and lineage receipt independently, then publish with Linux no-replace rename. Revalidate the emitted provider after rename.

**Rationale:** Partial output must never become bootstrap authority, and an existing destination must never be removed or overwritten.

### Decision: publish the smallest honest intermediate provider

**Choice:** Publish the self-hosted TinyCC compiler and runtime, native-musl headers and static libraries, 11 target-prefixed binutils tools, binutils headers and linker scripts, provider metadata, validation evidence, and the complete receipt.

**Rationale:** These four provider roles close the bounded StageX boundary without promoting the result to final GCC admission or compiler correctness.

## Risks / Trade-offs

- Early stages may genuinely require host shell/sandbox help; the claim must either construct and transition away from it or retain an explicit blocker.
- Seccomp user-notification support is platform-specific; unsupported kernels fail closed for StageX proof.
- The full protected transition is long-running. Publication imports one complete transition root and rejects stale or partial evidence.
- Some report-only stage observations have no standalone artifact bytes. The receipt uses an exact allowlist and a domain-separated projection over the artifact ID, plan digest, and complete report digest. Live provider components always use observed file or tree BLAKE3 identities.
- Stage reports bind each stage's complete declared authorization set only after every declared path-plus-digest identity appears in a kernel-intercepted `execve` or `execveat` decision in the separately bound raw audit. One raw event can satisfy equivalent authorization IDs, but an unused identity fails publication. This does not prove successful process completion.