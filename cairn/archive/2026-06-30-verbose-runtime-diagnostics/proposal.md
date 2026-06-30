# Proposal: Verbose runtime diagnostics

## Summary

Add a small Mantle runtime fingerprint that appears when operators ask for verbose or talkative diagnostics. The fingerprint should name the Mantle binary version and the command's important runtime context, including logical store prefix, physical store directory, state directory, substitution mode when applicable, and hermeticity mode when applicable. Default quiet and machine-readable modes must remain quiet unless verbosity is explicitly requested.

## Motivation

Determinate Nix 3.12.0 now prints the Nix version at the talkative log level because version identity is often decisive when debugging. Mantle has the same need, but with additional context that affects behavior: the default logical prefix is `/mantle/store`, the physical `--store` path may differ, state can be redirected, strict/practical hermeticity changes failure mode, and substitution/trust settings affect cache decisions.

Today an operator can infer some of this from CLI flags, but logs and copied bug reports often omit enough context to reproduce the failure. A concise verbose fingerprint makes support, evidence review, and regression triage easier without polluting normal output.

## Scope

- Define when Mantle emits a runtime fingerprint: explicit `--verbose`, explicit diagnostic log levels, or commands that intentionally request diagnostic output.
- Define fingerprint fields with stable names for human logs and structured JSON diagnostics when available.
- Keep default output quiet, especially `--json` stdout and non-verbose stderr.
- Add positive tests for verbose output on representative build/store commands.
- Add negative tests proving default and JSON modes are not polluted by the fingerprint.

## Non-goals

- No broad logging redesign.
- No dependency on Nix logging conventions or `nix -vv` flag behavior.
- No disclosure of secrets, trusted-private-key paths, bearer tokens, environment values, or full command arguments that may contain sensitive data.
- No guarantee that the runtime fingerprint proves build correctness, reproducibility, or release readiness.

## Reference

Determinate Nix 3.12.0 added Nix version printing at the talkative log level to make debug output self-identifying. Mantle should adopt that diagnostic value with Mantle-specific runtime context and stricter quiet-mode behavior.

Source reviewed: <https://determinate.systems/blog/changelog-determinate-nix-3120/>

## Target Spec Domains

- `operator-diagnostics` for verbose runtime fingerprint and quiet machine-output behavior.
