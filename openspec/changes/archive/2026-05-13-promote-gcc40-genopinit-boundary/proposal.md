## Why

GCC 4.0 still has a grouped generated-source boundary for `genopinit`, `genoutput`, and `genattrtab`. After splitting `genpeep`, the next smallest bounded increment is to isolate `genopinit` so the optab-initializer generator boundary is named and checked.

## What Changes

- Split `genopinit` from the grouped generated-source wrapper in `bootstrap/gcc-4.0.ncl`.
- Emit a checked empty-opinit source boundary with stable marker/symbol text.
- Reject legacy generic generated-source labels in `genopinit` output.
- Refresh GCC 4.0 placeholder inventory and parity regression coverage.

## Non-Goals

- Proving native `genopinit` correctness.
- Completing GCC 4.0 native/self-hosted compiler correctness.
- Changing parity classification from evidence-backed partial.

## Verification

Run Nickel/shell shape checks, the repo-local GCC 4.0 build, bootstrap parity tests, parity report, OpenSpec validation, and `git diff --check`.
