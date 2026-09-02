## Why

The strict repository Tiger Style gate now reaches the Mantle root library. It
reports 36 findings across `src/operator_contract.rs`, `src/protected_exec.rs`,
`src/protected_exec_seccomp.rs`, `src/errors.rs`, and `src/bootstrap.rs`.

Mantle must clear every remaining Tiger Style finding without allowances,
warning budgets, finding baselines, reduced targets, or weaker enforcement. The
repair must preserve operator-contract wire compatibility, remediation order,
bootstrap fetch meaning, protected-execution authority, fail-closed supervision,
audit identity, error-envelope compatibility, and public behavior. The one
checker-required public count normalization must use `u32` and checked caller
conversion instead of retaining platform-dependent `usize`.

## What Changes

- Make optional operator-contract wire defaults explicit and preserve their
  accepted legacy shape.
- Decompose contract validation and remediation classification along existing
  invariant and policy-family boundaries.
- Make protected-execution limits, predicates, arithmetic, and source admission
  explicit without widening executable authority.
- Keep seccomp and ptrace supervision fail-closed while using fixed-width public
  counts and handled response failures.
- Replace production serialization panics with deterministic typed or bounded
  fallback behavior.
- Split bootstrap seed fetching along existing service, request, and export
  phases.
- Repeat the complete repository Tiger Style gate until it reports no findings.

## Impact

- **Files:** the five current root-library files, any newly exposed first-party
  source, focused tests, one architecture record, and lifecycle evidence.
- **Testing:** pre-change and post-change root tests, complete repository Tiger
  Style, strict first-party Clippy, formatting, all-target caller checks, Nix
  evaluation, full Nix checks, Cairn validation, Tracey coverage, and gates.
