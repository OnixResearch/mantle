# Current blocker

Clean probe before this change:

- Receipt: `target/mantle-self-rust-plan-probe-after-27794c0b-clean/receipt.json`.
- HEAD: `27794c0b4ff5e7448c40674e854d0ba2d8082af0`.
- `topology_execution=blocked`.
- `ring@0.17.14` metadata run: `success`.
- Blocker class: `rustc-failed`.
- Blocker: `crates/crunch-project/src/error.rs` fails because `thiserror::Error` expansion cannot find `thiserror::__private` and `as_dyn_error`.

Inspection of the clean receipt shows `thiserror@2.0.18` consumes host artifact `thiserror-impl@1.0.69` instead of `thiserror-impl@2.0.18`.
