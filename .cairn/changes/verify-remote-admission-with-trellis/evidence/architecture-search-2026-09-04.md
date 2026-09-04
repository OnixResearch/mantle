# Architecture search: Trellis remote admission

## Goal

Connect Mantle remote-attempt decisions to the exact Trellis fenced-attempt model without adding Trellis runtime authority or a product dependency.

Completion needs these artifacts:

- A pure, fail-closed projection over the closed Mantle fact set.
- A bounded complete parity matrix from the exact Trellis executable model.
- Exact source, fixture, policy, verifier, proof, Kamacite, and Valence identities.
- Positive, negative, mutation, drift, and claim-boundary validation.
- Current Mantle and Trellis lifecycle evidence.

## False completion cases

These results do not complete the change:

- A matching phase table without fence, event, authority, or linkage coverage.
- A copied Trellis implementation without an exact upstream identity and oracle fixture.
- A proof sidecar that changes runtime admission or output authority.
- A global Trellis bundle that does not name the fenced-attempt model.
- A successful verifier command without assumptions, trusted boundaries, and non-claims.
- A parity result that silently drops an unsupported Mantle combination.

## Audit risks

The main risks are decision-order drift, non-injective identity mapping, progress-field mismatch, profile-version substitution, stale source evidence, and formal overclaims.

## Search budget

This search uses four mechanism families and one adversarial audit round. It uses local repositories and no network request.

The user did not authorize subagents. The serial lenses are correlated and do not count as independent reviews.

Allowed outcomes are `validated`, `blocked`, `exhausted`, or `user-decision-required`.

## Approach registry

### Family: Runtime dependency

- **Mechanism:** Add the Trellis crate as a normal Mantle dependency.
- **Claim:** Runtime code calls the verified executable model directly.
- **Artifact:** A Cargo dependency and direct conversion layer.
- **Evidence:** The change explicitly excludes Trellis from ordinary runtime dependencies.
- **Gap strength:** stronger
- **Blocker:** This route transfers proof-model code into the runtime dependency graph.
- **Next check:** None. The requirement rejects this mechanism.
- **State:** falsified

### Family: Source copy

- **Mechanism:** Copy the Trellis implementation into Mantle and compare two local functions.
- **Claim:** The copied model gives direct in-process parity.
- **Artifact:** Duplicate model source under Mantle.
- **Evidence:** A copy can drift from Trellis while local tests remain green.
- **Gap strength:** equivalent
- **Blocker:** The copy does not establish identity with the verified upstream source.
- **Next check:** None. A pinned oracle is stronger and smaller.
- **State:** falsified

### Family: Runtime proof parsing

- **Mechanism:** Parse Verus source, proof logs, Kamacite bytes, or Valence reports during remote admission.
- **Claim:** Each runtime decision consults formal evidence.
- **Artifact:** Runtime parsers and proof-presence gates.
- **Evidence:** ADR 0021 keeps these payloads opaque and outside runtime authority.
- **Gap strength:** stronger
- **Blocker:** This route changes the authority model and duplicates upstream semantics.
- **Next check:** None. The accepted boundary rejects this mechanism.
- **State:** falsified

### Family: Pinned evidence oracle

- **Mechanism:** Project Mantle facts into finite equality classes and compare them with a fixture produced by the exact Trellis executable model.
- **Claim:** Supported normalized cases agree, while every unsupported case rejects before proof coverage.
- **Artifact:** A pure projection, complete matrix fixture, source manifest, and evidence-only validator.
- **Evidence:** Trellis commit `8de4b24aa2d66cc2e6ec966d686df023492265d3` owns the archived proof and executable model.
- **Gap strength:** simpler
- **Blocker:** Implementation and adversarial validation are pending.
- **Next check:** Generate the oracle from the pinned Trellis source, then mutate each binding class.
- **State:** active

## Selected route

Use the pinned evidence-oracle route. Keep ordinary remote admission on `plan_remote_attempt_report`.

The projection will represent equality and ordering classes, not raw strings or cryptographic meaning. Unsupported combinations will return typed errors.

The evidence shell will bind the exact upstream revision and the deterministic `git archive` BLAKE3. Runtime code will not read this evidence.
