## Context

The hydrated full-source-provider proof reaches matching Mantle binaries but installs fetched standalone Rust. The source-built Rust provider proof reaches a Cargo-free fixed point but begins from an older host-assisted source-root native closure. Neither can authorize the missing conjunction. The current real-self-build parity file also predates the content-bound v2 rebuild descriptor and authority-plan contract.

## Decisions

### Decision: construct providers inside the proof authority

**Choice:** The promoted evidence run starts with empty transition, native-provider, Rust-provider, and Mantle output authorities. Its inputs contain authenticated source records and seed/lineage authority, never prebuilt provider directories. Development runs may use receipt-validated caches, but cached provider outputs cannot satisfy this proof.

**Rationale:** Importing or revalidating a previously built provider would prove consumption, not the claimed source-to-Mantle lineage.

### Decision: compose existing mechanisms under a new pure plan and Rust shell

**Choice:** A pure core defines the exact StageX transition, StageX publication, full-source native-provider, full-source Rust-provider, Mantle stage1, and Mantle stage2 sequence. A new Rust shell observes inputs and runs those mechanisms. The existing v1 Cargo-free fixed-point command remains a narrower diagnostic.

**Rationale:** Extending the v1 result could let existing provider-path inputs satisfy a source-construction claim. An external driver cannot own one typed authority plan or durable fail-closed evidence. ADR 0050 records this boundary.

### Decision: use one immutable closure policy for both Mantle stages

**Choice:** Stage1 and stage2 must use the same full-source-bound native/Rust closure policy digest, Cargo-free planner contract, source-state identity, and hermeticity policy. Stage1 Mantle is the stage2 planner/orchestrator; the host Mantle binary cannot plan stage2.

**Rationale:** A fixed point requires the produced tool to repeat the build under equivalent authority.

### Decision: require strict no-fallback execution

**Choice:** Both stages forbid live source acquisition, Cargo invocation, ambient tool discovery, host rustc/linker use, practical hermeticity fallback, protected-exec denial bypass, and undeclared store/checkout reads. Every attempted violation becomes durable failed evidence.

**Rationale:** Success with a fallback event would recreate the trust edge this proof exists to remove.

### Decision: use the v2 content-bound proof contract

**Choice:** Emit a `mantle-deterministic-proof-receipt-v2` receipt containing the canonical source/rebuild descriptor, authority plan, provider/closure identities, stage plans, run roots, approved read identities, effect-policy results, stage output digests, and proof bundle digest.

**Rationale:** The current parity verifier intentionally rejects the historical v1 receipt.

### Decision: make mismatch and failure durable

**Choice:** Each proof attempt writes preflight, provider, closure, stage, audit, stdout/stderr, digest, status, and blocker artifacts before success evaluation. Only a complete matching proof may update `latest` or release aliases.

**Rationale:** Long-running failures are evidence, not disposable logs.

## Risks / Trade-offs

- A clean proof can run for many hours and require substantial disk; preflight must bound both before construction.
- Stage1 may expose native-topology behavior not covered by prior one-shot proofs.
- Matching binaries prove a bounded fixed point, not compiler correctness or independent reproducibility.