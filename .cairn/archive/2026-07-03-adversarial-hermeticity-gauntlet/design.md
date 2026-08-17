## Context

Hermeticity claims are most valuable when they survive hostile inputs. Mantle already has strict/practical modes, protected exec, reference scanning, and sandbox policy, but the confirmation track needs explicit perturbation cases and deterministic reporting.

## Decisions

### 1. Perturbations are declared before execution

**Choice:** The gauntlet profile enumerates host-tool, environment, network, timestamp, locale, umask, temp path, store-path, and randomness perturbations up front.

**Rationale:** Declared axes make coverage reviewable and prevent ad hoc negative tests from being overgeneralized.

### 2. Strict mode must fail closed

**Choice:** Strict hermeticity cells must either produce clean evidence or fail before admissible reproducibility evidence is emitted.

**Rationale:** A strict report that merely warns would weaken release/global reproducibility claims.

### 3. Practical mode records degradation

**Choice:** Practical-mode cells may continue, but their report must carry audit events that block strict reproducibility admission.

**Rationale:** Practical builds remain useful for diagnostics while preserving strict evidence semantics.

## Risks / Trade-offs

- Some perturbations are platform-specific; unsupported perturbations must be explicit non-claims.
- Negative tests must avoid damaging the developer host and run only in isolated scratch roots.
