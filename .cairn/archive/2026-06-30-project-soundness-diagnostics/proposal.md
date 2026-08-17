# Proposal: Project soundness diagnostics

## Summary

Make `mantle check` and related project diagnostics report stable manifest × lockfile × generated-input soundness classes instead of generic validation failures.

## Motivation

Nixtamal's `check-soundness` is valuable because it names why a manifest and lockfile cannot be trusted together: version mismatch, kind mismatch, invalid context files, unsupported hash algorithms, prefetch failures, and more. Mantle should provide the same operator clarity for its Nickel manifest, JSON lockfile, generated `.mantle/inputs.ncl`, freshness probes, mirrors, patches, trust policy, fetch policy, and retention roots.

Better diagnostics make project workflows safer and improve evidence quality. They also reduce the chance that an agent or operator overclaims project readiness from a partial check.

## Scope

- Define stable project soundness diagnostic classes and JSON rendering.
- Cover schema/version mismatch, manifest parse errors, lockfile parse errors, input kind mismatch, undefined patches, unsupported hash algorithms, mirror issues, stale generated inputs, freshness probe errors, trust policy failures, fetch policy conflicts, retention root issues, and orphaned lock entries.
- Keep default `mantle check` no-network unless an explicit freshness/trust mode documents network behavior.
- Preserve quiet human output and parseable JSON output.

## Non-goals

- No background refresh or automatic repair unless an explicit command is selected.
- No replacing `mantle refresh`; check reports soundness, refresh mutates lock/input state.
- No broad build-success claim from project soundness alone.

## Target Spec Domains

- `operator-diagnostics` for diagnostic rendering and machine-readable behavior.
- `project-workflows` for project soundness semantics.
- `verification-evidence` for bounded readiness claims.
