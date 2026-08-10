# Bounded architecture search

## Success contract

Goal: admit concrete `/nix/store/*.drv` files through one reviewed `nix-derivation` adapter without changing native Mantle derivation behavior.

Completion evidence requires exact package-source parity, one production import site, bounded positive and negative projection tests, unchanged public schemas, and current Cairn gates.

False completion includes compilation alone, test-only dependency use, broad `nix-compat` replacement, silent unsupported-form flattening, Guix cutover, or a passing process with stale package evidence.

Audit risks are hash-domain confusion, out-of-band name drift, non-UTF-8 data loss, unbounded dynamic inputs, incomplete closure publication, and direct dependency bypasses.

The search used three serial lenses because subagent consent was absent. It used the existing call graph, exact upstream and package source, and adversarial fixtures. The round budget was three implementation and audit rounds.

Allowed outcomes were validated, blocked, exhausted, or user-decision-required.

## Approach registry

### Narrow production adapter

- Family: compatibility adapter
- Mechanism: one pure Mantle module wraps `nix-derivation` and returns Mantle-owned values
- Claim: closes the Nix metadata boundary without changing native Mantle identity
- Artifact: `src/nix_derivation_adapter.rs`
- Gap strength: equivalent to the requested boundary
- State: validated candidate

### Test-only oracle

- Family: dual-run oracle
- Mechanism: keep the old production parser and compare selected fixtures with the new crate
- Claim: detects disagreement but does not close the production boundary
- Gap strength: weaker
- Blocker: production still uses the adapted native parser for original Nix identity
- State: rejected as final design; retained only as test evidence

### Broad `nix-compat` replacement

- Family: dependency replacement
- Mechanism: replace native derivation, store, wire, and compatibility code with the new crate
- Claim: one derivation implementation
- Gap strength: stronger than the requested goal
- Blocker: the crate has no store, daemon, NAR, castore, or configurable-prefix implementation
- State: falsified

## Adversarial audit

The first adapter test found that the synthetic fixed-output fixture used an invalid all-zero output path.
The reviewed crate recalculated the Nix path as `n3i8ai7ldvbjgjdhdd0rl4sqcs14iqyx-hello-source`.
The fixture and its dependent root were corrected instead of weakening semantic projection.

Negative tests cover malformed input, invalid UTF-8 text, arbitrary environment bytes, name mismatch, floating output, deferred output, impure output, unsupported fixed methods, versioned syntax, dynamic inputs, invalid paths, and limit failures.

The surviving design keeps Guix prefix rewriting on the old path and rejects every currently unrepresentable Nix form before graph publication.

## Non-claims

The search and checks do not prove arbitrary Nix compatibility, evaluator parity, build success, output correctness, store trust, reproducibility, or release eligibility.
