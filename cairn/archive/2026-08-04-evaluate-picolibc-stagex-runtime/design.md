## Context

Mantle has an authenticated static native-musl boundary inside StageX. The current materializer compiles 746 self-host sources and 19 predecessor sources.

The Picolibc x86_64 Linux profile uses static linking and its own Linux operating-system library. Upstream uses Meson and Ninja for configuration and construction.

Picolibc can fit an early static runtime role. It does not match the final dynamic musl provider contract.

A successful ordinary build also does not prove TinyCC compatibility. It does not prove a smaller protected execution graph.

## Goals

- Measure one pinned Picolibc release against the current StageX native-musl boundary.
- Preserve exact source, tool, configuration, output, and license identities.
- Reuse the current positive and negative libc behavior contract.
- Produce one deterministic decision with explicit evidence and non-claims.
- Keep all provider and parity state unchanged.

## Non-goals

- Replace the final musl provider.
- Add a bare-metal target or frontend package policy.
- Admit Meson or Ninja into protected StageX execution.
- Prove Picolibc, musl, TinyCC, the compiler, or the kernel correct.
- Claim release eligibility or full-bootstrap completion.

## Decisions

### Decision: keep the experiment diagnostic-only

**Choice:** Put the Picolibc build in a research-only diagnostic path. Do not add it to the StageX lineage or normalized provider graph.

The diagnostic can use ordinary Mantle sandbox inputs. It cannot update parity rows, provider metadata, accepted output digests, or release claims.

**Rationale:** Feasibility evidence must exist before an authority change. A successful build is not provider-admission evidence.

### Decision: bind complete source and license authority

**Choice:** Pin one release, acquisition URL, fixed-output hash, source BLAKE3, selected Linux profile, and upstream configuration file.

Record BLAKE3 identities for Meson, Ninja, the compiler, the linker, and all generated configuration inputs. Deny network access during realization.

Classify the library, test, and helper licenses. Preserve required notices and identify which license classes enter runtime outputs.

**Rationale:** Picolibc contains different license classes. A repository URL or release tag cannot identify the complete build authority.

### Decision: use the upstream build only for feasibility evidence

**Choice:** Build the pinned x86_64 Linux static profile with explicit Meson and Ninja inputs. Capture the exact compiled source closure and generated files.

Classify Meson and Ninja as diagnostic-only tools. A protected candidate plan must remove these roles or admit them through a separate change.

**Rationale:** The upstream route gives the fastest honest feasibility result. It cannot silently widen the protected StageX execution graph.

### Decision: compare behavior and isolated output identity

**Choice:** Run the current StageX libc positive behavior cases against Picolibc. Run malformed-source, host-libc-dependency, and tamper rejection cases too.

Build twice with fresh state, output, and scratch roots. Compare BLAKE3 identities for all selected runtime artifacts and normalized reports.

**Rationale:** Compile success and output size do not establish runtime suitability or deterministic construction.

### Decision: classify outcomes in a pure Rust core

**Choice:** Implement report normalization and outcome selection as pure functions. Keep file reads, process execution, and report writes in a thin shell.

The classifier uses three outcomes:

| Outcome | Meaning |
|---|---|
| `candidate` | All behavior and identity checks pass. The route uses fewer compiled units and source rewrites. A protected plan adds no executable roles. |
| `rejected` | Evidence proves behavior failure, nondeterminism, host-libc dependence, or no reduction against the baseline. |
| `blocked` | Required source, tool, license, behavior, identity, or protected-route evidence is missing or inconclusive. |

Every comparison field has a named unit and bound. The report records both observed values and the decision reasons.

**Rationale:** A deterministic classifier prevents a favorable narrative from overriding missing or contrary evidence.

### Decision: require a separate change for adoption

**Choice:** A `candidate` outcome authorizes only a later Cairn proposal and ADR review. It does not change bootstrap execution or provider selection.

A `rejected` or `blocked` outcome preserves its exact blocker and smallest next action. It does not fabricate completion.

**Rationale:** Research evidence and provider authority have different review and proof requirements.

### Decision: record the external reference only after use

**Choice:** If implementation consumes Picolibc source or tests, add the repository to the README `## References` section.

The ADR and oracle checkpoint must name the question, inspected evidence, decision, owner, next action, and non-claims.

**Rationale:** The repository reference must correspond to a concrete, recorded use.

## Risks / Trade-offs

- Mature build tools can hide TinyCC or protected-route incompatibility. The classifier reports this result as `blocked`.
- Translation-unit count is not a semantic complexity measure. The report also records rewrites, tools, behavior, and authority roles.
- The Picolibc tests can exercise different contracts than musl. Only shared declared behavior enters parity.
- Mixed upstream licenses add review work. The source and output manifests keep each license class explicit.
- The native-musl baseline can change during the experiment. The report binds the exact baseline commit and evidence identities.
