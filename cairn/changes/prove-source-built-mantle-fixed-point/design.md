## Context

The hydrated full-source-provider proof reaches matching Mantle binaries but installs fetched standalone Rust. The source-built Rust provider proof reaches a Cargo-free fixed point but begins from an older host-assisted source-root native closure. Neither can authorize the missing conjunction. The current real-self-build parity file also predates the content-bound v2 rebuild descriptor and authority-plan contract.

## Decisions

### Decision: construct providers inside the proof authority

**Choice:** A cold promoted evidence run starts with empty transition, native-provider, Rust-provider, and Mantle output authorities. Its inputs contain authenticated source records and seed/lineage authority, never unbound provider directories. A later promoted attempt may restore one immutable provider checkpoint only after its four stage receipts, stage-specific sources, recipe projection, policies, resources, predecessor outputs, semantic outputs, execution evidence, and payload identities revalidate. Development cache entries remain ineligible. Profile refresh replaces the Mantle source and checked vendor records together. Proof preflight validates their lock, package, and file-checksum closure before StageX. ADR 0080 records this pairing.

**Rationale:** Raw provider import proves only consumption. A promoted checkpoint composes the original execution evidence and identifies every restored stage without claiming current execution.

### Decision: compose existing mechanisms under a new pure plan and Rust shell

**Choice:** A pure core defines the exact StageX transition, StageX publication, full-source native-provider, full-source Rust-provider, Mantle stage1, and Mantle stage2 sequence. A new Rust shell observes inputs and runs those mechanisms. The existing v1 Cargo-free fixed-point command remains a narrower diagnostic.

**Rationale:** Extending the v1 result could let existing provider-path inputs satisfy a source-construction claim. An external driver cannot own one typed authority plan or durable fail-closed evidence. ADR 0050 records this boundary.

### Decision: run the first promoted proof on Leviathan

**Choice:** The first promoted V2 evidence run executes the complete six-stage proof on Leviathan (`leviathan.cymric-daggertooth.ts.net`). Host Mantle orchestration, StageX transition and publication, native and Rust provider construction, stage1, and stage2 all execute on that host. The operator transfers the exact source tree and authenticated profile before the attempt. Leviathan's pueue daemon launches and retains the run. Mantle `--builder`, Nix remote-action dispatch, and split-host stage execution are forbidden for this run.

The Tailscale DNS name is an operator routing label, not execution authority. Proof authority continues to come from authenticated source and policy identities, explicit executable paths, bounded host observations, the action trust plan, and planned-versus-observed reconciliation. SSH, rsync, and pueue remain outside the proof action graph as pre-launch transfer and control mechanisms.

**Rationale:** Leviathan provides substantially more CPU, memory, and disk than the operator workstation. Running the complete process there uses those resources while preserving the proof's local-only execution rule.

### Decision: use one immutable closure policy for both Mantle stages

**Choice:** Stage1 and stage2 must use the same full-source-bound native/Rust closure policy digest, Cargo-free planner contract, source-state identity, and hermeticity policy. Stage1 Mantle is the stage2 planner/orchestrator; the host Mantle binary cannot plan stage2.

**Rationale:** A fixed point requires the produced tool to repeat the build under equivalent authority.

### Decision: make remapped Cargo manifests readable through rustc's working directory

**Choice:** Deterministic non-custom Rust units receive `/proc/self/cwd/<package-relative-path>` as `CARGO_MANIFEST_DIR`. Rustc already runs from the admitted source root. Custom-build compiler and child environments keep their physical package roots. The `/mantle/release/source` rustc remap remains unchanged. ADR 0081 records this lowering.

**Rationale:** Compile-time macros can read the selected `Cargo.toml` through an absolute stable path. Physical paths are forbidden because rustc does not remap arbitrary `env!` output.

### Decision: require strict no-fallback execution

**Choice:** Both stages forbid live source acquisition, Cargo invocation, ambient tool discovery, host rustc/linker use, practical hermeticity fallback, protected-exec denial bypass, and undeclared store/checkout reads. Every attempted violation becomes durable failed evidence.

**Rationale:** Success with a fallback event would recreate the trust edge this proof exists to remove.

### Decision: plan the complete action trust graph before execution

**Choice:** The pure proof core derives one root-scoped action trust plan from the selected proof root. Each reachable action names its broad stage, producer actions, fixed or produced executable authorities, input authorities, outputs, local-only execution rule, event-count bounds, and resource limits. An incomplete adapter or action list blocks execution.

Provider construction emits complete stage-local plans before each provider stage. A promoted checkpoint binds those plans and their reconciliations. A later restored attempt composes them with stage1 and stage2 plans before any current build action. The final root plan cannot depend on an unbuilt output or retrofit authority from observations.

**Rationale:** A six-stage summary cannot expose an undeclared child tool or missing producer edge before a long proof starts. A complete pre-execution list provides a cheap failure point without weakening runtime enforcement. Staged provider plans break the produced-rustc planning cycle while keeping every original action under prior authority.

### Decision: bind generated executables to producers, not paths

**Choice:** A fixed executable carries a reviewed BLAKE3 identity. A generated executable carries its producer action and output identity, then receives an observed BLAKE3 before execution. A store path, output prefix, executable name, or generated-directory location cannot grant authority by itself.

Rust unit actions use the unit graph for producer edges. Compile actions use fixed toolchain authority. Build-script executions use the compile action and declared output identity. Receipt-bound aliases use the BusyBox shell identity from the full-source Rust binding, not ambient `/bin/sh`.

**Rationale:** Path classification can mislabel copied, stale, or attacker-selected files as generated output. The producer relationship and observed content identity preserve the existing Mantle trust model.

### Decision: reconcile planned and observed execution

**Choice:** The proof shell maps current or checkpoint-bound protected-exec and build records back to planned actions. It rejects unknown events, missing required evidence, digest drift, producer drift, count-bound violations, remote execution, and cache-only completion without a promoted checkpoint receipt. Restored stages retain their original execution-evidence digest and checkpoint identity. The operator trust report is a view over these bound records, not a separate authority source. ADR 0070 records this boundary.

**Rationale:** Static review and runtime interception cover different failure modes. Their explicit reconciliation makes the useful action-audit shape visible without replacing seccomp evidence.

### Decision: use the v2 content-bound proof contract

**Choice:** Emit a `mantle-deterministic-proof-receipt-v2` receipt containing the canonical source/rebuild descriptor, authority plan, provider/closure identities, stage plans, action-trust plan, observed execution reconciliation, run roots, approved read identities, effect-policy results, stage output digests, and proof bundle digest. The rebuild source closure contains one aggregate source-authority root plus every validated source-role leaf. The root digest equals the receipt source digest.

**Rationale:** The current parity verifier intentionally rejects the historical v1 receipt. The new action-trust identities must also be content-bound rather than inferred from producer status. The aggregate root binds the complete source authority while the leaves keep each source role reviewable. ADR 0083 records this boundary.

### Decision: separate durable evidence from working scratch

**Choice:** The final proof-bundle digest covers a declared durable projection. It retains the complete StageX execution tree, provider outputs, source authority, stage outputs, receipts, audits, transcripts, and action-trust evidence. It excludes only Cargo-free execution intermediates, proof home and temporary directories, native store state, and Rust-provider scratch. The StageX tree uses a bounded, no-follow observation digest that records opaque symlink target bytes without granting source authority. Unknown unreadable content still fails closed. ADR 0079 records this boundary.

**Rationale:** The V26 working root contained more than 2.5 million files and links plus intentionally unreadable overlay work directories. Those bytes are useful attempt diagnostics, not durable proof evidence. Applying source-admission rules to the complete StageX evidence tree also conflicts with ADR 0052 because its negative fixtures intentionally contain absolute and escaping links.

### Decision: make mismatch and failure durable

**Choice:** Each proof attempt writes preflight, provider, closure, stage, audit, stdout/stderr, digest, status, and blocker artifacts before success evaluation. Only a complete matching proof may update `latest` or release aliases.

**Rationale:** Long-running failures are evidence, not disposable logs.

## Risks / Trade-offs

- A clean proof can run for many hours and require substantial disk; preflight must bound both before construction.
- Leviathan cannot currently authenticate to GitHub, so the operator must transfer a fresh source tree and retain exact post-transfer parity evidence.
- The Leviathan route name cannot grant proof authority. Host preflight must record the observed system, architecture, kernel, resource bounds, and explicit sandbox tools.
- Native-provider and Rust-unit adapters can expose incomplete action descriptions. The proof must stop instead of emitting a partial report.
- Native derivation planning now covers 88 unique actions and 568 bounded scheduler observations across the provider and host-tool roots. Rust provider and Rust-unit child process interception remains unfinished.
- Generated build scripts require producer-linked authority and a digest observation before execution.
- The complete action list and observations can be large, so schemas and event counts need explicit limits.
- Stage1 may expose native-topology behavior not covered by prior one-shot proofs.
- Matching binaries prove a bounded fixed point, not compiler correctness or independent reproducibility.