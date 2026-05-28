# Design

## Functional core

Add two bounded proc-macro classification repairs:

1. Native manifest parsing accepts both `[lib] proc-macro = true` and Cargo-normalized `[lib] proc_macro = true` for the same proc-macro library flag.
2. Cargo unit target classification treats `crate_types` containing `proc-macro` as a host proc-macro only when the target kind is lib/rlib-shaped.

Use the Cargo target classifier in unit derivation selection, host artifact discovery, and selected host-unit key extraction so the same Cargo target object cannot be treated as a target lib in one path and a host proc-macro in another.

## Imperative shell

No new shell behavior. The existing topology executor already handles proc-macro host artifacts once planning classifies the unit as host.

## Risk

Do not classify arbitrary crate types or bin-shaped targets as proc macros. Only the manifest proc-macro flag or the literal `proc-macro` crate type on lib/rlib-shaped Cargo targets may produce a proc-macro host unit. This keeps unsupported target-kind handling fail-closed.
