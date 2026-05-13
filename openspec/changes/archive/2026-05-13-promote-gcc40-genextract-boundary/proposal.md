## Why

GCC 4.0 still has a grouped generated-source boundary for `genextract`, `genpeep`, `genopinit`, `genoutput`, and `genattrtab`. After splitting `genrecog`, the next smallest bootstrap increment is to isolate `genextract` so its bridge output is checked and named instead of hidden behind the generic grouped source wrapper.

## What Changes

- Split `genextract` from the grouped generated-source wrapper in `bootstrap/gcc-4.0.ncl`.
- Emit a checked empty-extraction source boundary with stable marker/symbol text.
- Reject legacy generic generated-source labels in `genextract` output.
- Refresh GCC 4.0 placeholder inventory and parity regression coverage.

## Non-Goals

- Proving native `genextract` correctness.
- Completing GCC 4.0 native/self-hosted compiler correctness.
- Changing parity classification from evidence-backed partial.

## Verification

Run Nickel/shell shape checks, the repo-local GCC 4.0 build, bootstrap parity tests, parity report, OpenSpec validation, and `git diff --check`.
