# Verus Specifications

This directory contains [Verus](https://verus-lang.github.io/verus/guide/) formal
verification specs for this project.

Each `*_spec.rs` file defines invariants, preconditions, and postconditions that
are machine-checked by the Verus verifier.

## Running verification

```bash
# If verus is in your devShell:
verus verus/example_spec.rs
```
