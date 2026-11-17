# 0159. Canonicalize primary sources before recipe lowering

Status: Accepted

## Context

Leviathan cold task 1714 failed closed on the `snix-build` build-script compiler
in the source-built fixed-point workflow. Its source-only unit plan retained
`crates/crunch-pipeline/../crunch-build/../crunch-rustc-wrapper/../crunch-rust-cache/../crunch-store/../../vendor/snix-build/build.rs`.
The authored compiler correctly consumed the typed Mantle-source member
`vendor/snix-build/build.rs`. Runtime admission compares the independently
planned primary operand with that exact source-role/member input; it does not
resolve parent-directory aliases to make the comparison pass.

The original inert selector inspector normalized the plan operand before its
comparison and therefore missed the mismatch. Retained-data task 1717 checks
all 2,098 primary-source obligations without that normalization: 126 are
invalid and none are uncovered. It reproduces the actual failed unit, confirms
both operands refer to the same physical file, and rejects their lexical
mismatch. The failure is producer path composition, not missing source authority.

The current consumer is the source-built cold fixed-point proof. The target
outcome is exact agreement between source-only planning, recipe lowering and
fresh runtime admission, before any provider or Mantle stage executes.

## Decision Drivers

- Preserve exact source-role/member admission and independent runtime planning.
- Fix all native primary target kinds and nested path dependencies at their producer.
- Keep filesystem observation in the Rust shell, outside the functional core.
- Expose complete primary-source inspection coverage, including failed matches.
- Regenerate changed-source cohorts rather than accepting old plans or reviews.

## Decision

`push_native_target` resolves each readable target file canonically before
constructing `NativeTargetCandidate`. Missing or non-file targets retain the
existing source-failure path. Build scripts, libraries, proc macros and binaries
use this common producer; downstream lowering receives the admitted physical
source spelling instead of accumulated manifest parent segments.

Runtime `validate_primary_source`, typed input admission, source capture,
compiler-source hashing and semantic-unit equality remain unchanged. No path
exception, hash exclusion, relaxed comparison or runtime canonicalization is
introduced.

The inert selector inspector keeps package-ownership normalization separate
from exact primary operands. Its primary comparison preserves parent segments,
resolves the declared typed input's source role and member prefix, and mirrors
runtime lexical path equality. Every selected compiler contributes a primary
obligation; absent provenance is uncovered and a mismatched operand is invalid.
The inspector also binds the exact runtime consumer's source bytes.

## Consequences

- The source-only Rust planner and Mantle proof-inspection adapters own this
  boundary. The durable capability is repeatable primary-source agreement over
  complete source-owned unit plans, including nested path dependencies.
- Regression evidence covers build-script and library parent paths, actual
  retained failed-unit data, typed-input boundaries and the inspector's
  rejection of a same-file noncanonical plan mutation.
- Changed planner and inspector bytes require a new published source snapshot,
  separate absent-target qualifications, complete source-only regeneration and
  fresh full current-data independent reviews before another protected attempt.
- Developer tests and inert inspection are not provider execution, a fixed
  point, a verified v2 receipt, native-row qualification or release authority.
