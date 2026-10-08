# Design: Adopt data-only dependency exports

## Goal and scope

Define, ahead of demand, the dependency interface a Mantle package layer will
use: typed exports records, no behavior injection, and a typed build-system
registry. Implementation is gated on an admission decision naming the first
consumer. This change is a design prior, not an implementation package.

Planning success means a native change package whose requirements and gate
task make premature implementation impossible by rule.

## Current behavior

`builders/mk_derivation.ncl` composes PATH entries and passes string phases;
dependencies influence builds through whatever recipe shell reads. There is
no package layer yet; the mantlepkgs catalog family evaluates nixpkgs as a
producer action rather than defining Mantle-native package semantics. Nothing
conflicts with the proposed contract; nothing implements it either.

The external reference ran a blind test of five builder API shapes against
people and LLMs writing packages; explicit build systems with a plain phase
list won every round. Its exports rule: one `exports.json` per output derived
from the tree; prepare renders `CPPFLAGS`, `LDFLAGS`, `PKG_CONFIG_PATH`,
`CMAKE_PREFIX_PATH` from records; `exports = false` marks outputs nothing
links against (`evidence/repkgs-review.md`).

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Status quo | String phases, PATH composition | Rejected as the layer's future: uncheckable options, implicit behavior | Blind-test result recorded |
| Setup-hook style | Dependencies inject shell into consumers | Rejected: behavior injection is the specific failure mode | No-behavior requirement |
| Exports as data | Typed records rendered by consumer prepare | Selected direction | Rendering fixtures |
| Full module system | Per-package fixpoints and option modules | Rejected: measured 10x evaluation cost in the reference; Mantle has no such demand | Evaluation-cost prior |

## Contract and component ownership

- Policy and eval: the exports record and registry contracts as typed Nickel
  contracts; evaluation-time validation semantics owned by `crunch-eval`
  integration points.
- Core: rendering of consumer search paths from records is pure planning;
  shell only writes environment.
- No new components before admission; this package owns the contract text
  and the gate.

## Decisions

### Decision: Design prior with an implementation gate

**Choice:** All implementation tasks are blocked behind a recorded admission
decision.

**Rationale:** The workspace rule rejects infrastructure without concrete
demand. Writing the contract now is cheap because the external evidence is
fresh; implementing it without a consumer would violate the demand rule.

### Decision: Nickel, not a new language

**Choice:** The registry and records are Nickel contracts.

**Rationale:** The reference's Nushell choice served its static-seed goals;
Mantle's evaluation layer is Nickel and already enforces closed contracts.

## Risks / Trade-offs

- A contract written before its consumer can guess wrong; the admission gate
  task requires re-review of this design against the real consumer surface.
- Environment defaults with placeholders add one expansion rule; the
  relocatable-outputs change owns path-relativity mechanics and both must
  agree.
