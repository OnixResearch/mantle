# Design

## Functional core

Add a pure helper that returns the dependency names accepted for a host artifact:

- proc macros: Rust crate-name spelling of the target name.
- custom builds: Rust crate-name spelling plus Cargo's `build_script_main` and `build_script_build` aliases.
- other host kinds: Rust crate-name spelling only.

`bind_all_host_artifacts(...)` uses that bounded name set when deciding which dependency placeholders are satisfied by a consumed host artifact.

## Imperative shell

No new shell behavior. The execution scheduler already produces host artifacts first; binding just needs to recognize all Cargo spellings for the same build-script producer.

## Risk

The alias set is intentionally fixed. Do not wildcard every same-package dependency name; that could mask real dependency graph errors.
