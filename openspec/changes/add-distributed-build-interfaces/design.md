# Design: Add distributed build interfaces

## Context

Crunch's useful distribution unit is the derivation/output, not a Rust source file or Cargo crate. Rust-specific tools such as `sccache` and `sccache-dist` are useful externally, but they do not model Crunch's whole derivation graph, store identity, PathInfo persistence, bootstrap packages, fixed-output derivations, or sandboxed builder receipts. Enterprise build systems such as Buck2 and Bazel prove the value of remote realization and hermetic request keys, but their graph model and policy should not be embedded directly into Crunch.

Crunch should instead expose narrow interfaces around its existing local pipeline while preserving the Nix-like invariant:

> Crunch distributes and caches derivation realizations, not arbitrary build-system actions.

The resulting flow is:

1. Derive deterministic realization keys from Crunch-native derivation realization facts.
2. Ask configured resolvers for trusted existing artifacts.
3. Dispatch ready derivation goals through a realization policy that can choose local sandbox realization now and remote realization later.
4. Verify every returned output through the same PathInfo/castore/finalization path.
5. Publish verified artifacts through configured publishers.

## Goals / Non-Goals

**Goals:**

- Keep local-only realizations as the default and reference behavior.
- Make realization-cache and realizer providers pluggable adapters, not scheduler constants.
- Define Crunch-native keys over derivation, inputs, toolchain paths, sandbox policy, platform, and store prefix facts.
- Preserve substitute lookup before rebuild or remote realization.
- Require output verification and logs/receipts for every non-local result.
- Keep policy data-driven so future configs can select providers without recompiling Crunch.

**Non-Goals:**

- No concrete remote realization protocol in this baseline.
- No mandatory dependency on S3, Redis, REAPI, BuildBuddy, EngFlow, Buck2, Bazel, `sccache`, or Cargo.
- No Rust crate-level distributed compilation inside Crunch.
- No default network access or remote realization.
- No bypass of PathInfo, castore, signature, or final output verification.

## Decisions

### 1. Realization keys are Crunch-native and provider-neutral

**Choice:** Introduce a pure key-derivation seam that computes realization keys from normalized Crunch derivation realization facts. The key model must include the derivation identity, declared input closure identities, builder/args/env after normalization, platform/system, sandbox/hermeticity mode, store prefix/store-dir semantics, relevant toolchain paths, and any realizer profile inputs that can affect output.

**Rationale:** Cache hits are only safe if the key reflects the facts that influence output. Keeping this pure and provider-neutral allows tests to exercise determinism without network services.

**Alternative:** Reuse `sccache` hashes or hard-code Nix-style narinfo paths as the only key. Rejected because those are either Rust compiler specific or artifact-address specific rather than a complete Crunch derivation-realization key.

### 2. Artifact lookup and publication use resolver/publisher traits

**Choice:** Model artifact reuse through resolver and publisher interfaces. Local PathInfo lookup, existing binary substitution, directory cache export, and future remote caches become implementations of those interfaces.

**Rationale:** Crunch already has binary cache/substitution logic, but distributed build work needs a general seam that can ask multiple sources without coupling the scheduler to URL formats or storage vendors.

**Alternative:** Add more flags directly to the realization command for each cache backend. Rejected because that makes provider selection a CLI concern and encourages hard-coded backend behavior.

### 3. Realization selection is policy over realizer adapters

**Choice:** The scheduler asks a realization policy to select a realizer for each ready derivation goal. The initial required realizer is the existing local sandbox realizer. Remote realization is a future adapter that must implement the same contract: materialize inputs, run builder under declared policy, return logs, output metadata, and verifiable content references.

**Rationale:** This keeps the goal scheduler focused on dependency readiness and deduplication while making local/remote placement swappable.

**Alternative:** Fork a separate distributed scheduler. Rejected because Crunch's lazy goal model already owns the correct readiness and waiter semantics.

### 4. Remote realization returns candidates, not trusted success

**Choice:** A remote realizer result is only a candidate until Crunch verifies returned content and metadata through the same finalization path used by local realizations or an explicitly equivalent verifier interface.

**Rationale:** Remote workers are outside the local trust boundary. Verification prevents protocol adapters from becoming implicit authorities.

**Alternative:** Trust the remote service's success receipt as PathInfo. Rejected because it would let provider-specific assertions bypass Crunch's store model.

### 5. Configuration names capabilities, not concrete dependencies

**Choice:** Configuration should select logical resolver/publisher/realizer profiles and capability requirements. Concrete backends live behind adapter registration and feature gates.

**Rationale:** The user explicitly wants seams and interfaces, not hard-coded dependencies. This also lets default builds exclude optional remote libraries.

**Alternative:** Add first-class flags such as `--s3-cache`, `--redis-cache`, or `--reapi-endpoint` in the first implementation. Rejected because those flags prematurely freeze provider choices.

## Interface Sketch

Names are illustrative; implementation may refine module placement.

```rust
pub trait RealizationKeyDeriver {
    fn derive_key(&self, request: &RealizationKeyRequest) -> Result<RealizationKey, RealizationKeyError>;
}

pub trait ArtifactResolver {
    async fn resolve(&self, key: &RealizationKey, request: &ResolveRequest)
        -> Result<ResolveOutcome, ResolveError>;
}

pub trait ArtifactPublisher {
    async fn publish(&self, key: &RealizationKey, artifact: &VerifiedRealizationArtifact)
        -> Result<PublishOutcome, PublishError>;
}

pub trait DerivationRealizer {
    async fn execute(&self, request: RealizationRequest)
        -> Result<RealizationOutcome, RealizationError>;
}

pub trait RealizationPolicy {
    fn choose_realizer(&self, goal: &ReadyGoal, capabilities: &RealizerCapabilities)
        -> RealizerSelection;
}
```

Important boundaries:

- `RealizationKeyDeriver` is pure and deterministic.
- `ArtifactResolver` may read local or remote stores but must not mutate realization state directly.
- `ArtifactPublisher` only accepts verified artifacts.
- `DerivationRealizer` may be local or remote, but it never bypasses output verification.
- `RealizationPolicy` is data-driven and testable without concrete backends.

## Risks / Trade-offs

**Over-broad first slice** → Mitigate by landing only interface contracts, local adapters, and no-op/default policies first.

**False cache hits** → Mitigate with conservative key inputs, golden tests, and negative tests that perturb env, inputs, platform, sandbox mode, and toolchain paths.

**Provider lock-in** → Mitigate by keeping concrete protocols behind optional adapters and forbidding backend-specific behavior in core scheduler tests.

**Remote trust confusion** → Mitigate by treating remote results as untrusted candidates until PathInfo/castore verification succeeds.

**Configuration sprawl** → Mitigate by defining capability/profile data structures before adding backend-specific config fields.

## Validation Plan

- Validate this OpenSpec strictly before implementation.
- Add deterministic key-derivation tests before enabling any cache lookup beyond existing PathInfo/substitution.
- Add policy-selection tests proving default local-only behavior, explicit remote opt-in, remote-unavailable fallback, and no implicit network.
- Add trait contract tests with in-memory fake resolver/publisher/realizer implementations before real remote adapters.
- Add integration evidence that a cache hit skips realization, a cache miss falls through to local realization, and a remote realization candidate is rejected when verification fails.
