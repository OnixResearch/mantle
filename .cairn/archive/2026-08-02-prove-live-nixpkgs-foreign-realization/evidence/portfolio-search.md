# Portfolio search contract

## Goal

Prove one fresh live `nixpkgs#hello` graph reaches Mantle `realized` and a
bounded provenance audit without a Nix frontend during consumption.

## Completion evidence

- Fresh host-Nix drv resolution and recursive derivation JSON.
- Mantle producer, validation, executable plan, and import receipt.
- Trusted cache closure facts under exact `/nix/store` paths.
- A normal scheduler build report with no builder execution.
- Realization and provenance receipts with valid self-digests.
- Exact rerun reuse and fresh-state cache hydration evidence.
- Positive and negative automated tests and passing Cairn gates.

## False completion

Planning alone, host-path copying, unsigned cache content, local build fallback,
Nix commands during consumption, fixture-only results, and a failed audit called
passing are not completion.

## Audit risks

The audit must test signature authority, NAR identity, closure completeness,
store-prefix ambiguity, source-bundle bypass, scheduler fallback, hidden foreign
references, limits, stale receipts, and unsupported claim promotion.

## Budget

The search allows three mechanism families, two implementation rounds, one
advisory model review, repository and Nix cache sources, and deterministic Rust,
Nickel, Nix, and Cairn validators.

Allowed terminal states are validated, blocked with an exact cache or code
boundary, exhausted, or user-decision-required.

## Approach registry

| Family | Mechanism | State |
|---|---|---|
| preserved-cache-path | Keep exact `/nix/store` paths and make the plan cache-only | active |
| content-rebinding | Rebind signed Nix NAR content to `/mantle/store` | falsified by embedded references |
| producer-cache-conversion | Export a new Mantle-native cache from host Nix | blocked by the same path problem and extra authority |

The advisory model timed out. Its output is not evidence.
