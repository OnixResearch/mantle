Task-ID: V1
Covers: r[compiled-eval.backend-boundary], r[compiled-eval.private-backend-seam], r[compiled-eval.cranelift-prototype-subset]
Command: cargo test -p crunch-eval --lib
   Compiling syn v2.0.117
   Compiling regex-syntax v0.8.10
   Compiling aho-corasick v1.1.4
   Compiling serde_core v1.0.228
   Compiling petgraph v0.7.1
   Compiling malachite-base v0.6.1
   Compiling wide v0.7.33
   Compiling winnow v0.7.15
   Compiling toml_edit v0.25.10+spec-1.1.0
   Compiling blake3 v1.8.2
   Compiling nom v8.0.0
   Compiling rustix v1.1.4
   Compiling proc-macro-crate v3.5.0
   Compiling regex-automata v0.4.14
   Compiling serde_json v1.0.149
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling indexmap v2.13.1
   Compiling serde_spanned v1.1.1
   Compiling tempfile v3.27.0
   Compiling darling_core v0.23.0
   Compiling logos-codegen v0.15.1
   Compiling proc-macro2-diagnostics v0.10.1
   Compiling serde_derive v1.0.228
   Compiling futures-macro v0.3.32
   Compiling curve25519-dalek-derive v0.1.1
   Compiling thiserror-impl v2.0.18
   Compiling num_enum_derive v0.7.6
   Compiling tokio-macros v2.7.0
   Compiling tracing-attributes v0.1.31
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/crunch/crunch/vendor/nix-compat-derive)
   Compiling toml v0.9.12+spec-1.1.0
   Compiling ouroboros_macro v0.18.5
   Compiling curve25519-dalek v4.1.3
   Compiling tokio v1.51.0
   Compiling futures-util v0.3.32
   Compiling num_enum v0.7.6
   Compiling tracing v0.1.44
   Compiling thiserror v2.0.18
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling ouroboros v0.18.5
   Compiling logos-derive v0.15.1
   Compiling logos v0.15.1
   Compiling ed25519-dalek v2.2.0
   Compiling darling_macro v0.23.0
   Compiling darling v0.23.0
   Compiling serde_with_macros v3.18.0
   Compiling serde v1.0.228
   Compiling lalrpop-util v0.22.2
   Compiling regex v1.12.3
   Compiling serde_with v3.18.0
   Compiling lalrpop v0.22.2
   Compiling malachite-nz v0.6.1
   Compiling codespan-reporting v0.13.1
   Compiling nickel-lang-vector v0.1.0
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/crunch/crunch/crates/crunch-attestation-core)
   Compiling serde_yaml v0.9.34+deprecated
   Compiling codespan v0.13.1
   Compiling bstr v1.12.1
   Compiling futures-executor v0.3.32
   Compiling futures v0.3.32
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/crunch/crunch/crates/crunch-attestation)
   Compiling nix-compat v0.1.0 (/home/brittonr/git/crunch/crunch/vendor/nix-compat)
   Compiling crunch-glue v0.1.0 (/home/brittonr/git/crunch/crunch/crates/crunch-glue)
   Compiling nickel-lang-parser v0.1.1
   Compiling nickel-lang-core v0.16.1
   Compiling malachite-q v0.6.1
   Compiling malachite-float v0.6.1
   Compiling malachite v0.6.1

Fresh closeout attempt result: BLOCKED
- pueue task 17 ran for 9m11s and was killed after `cargo test -p crunch-eval --lib` made no visible progress past Nickel crate compilation under the isolated `target/closeout-compiled-eval` target directory.
- Earlier pueue tasks 14/15 failed because the inherited global `~/.cargo-target` was root-owned; retry used writable `CARGO_TARGET_DIR`.
- Next best checked-in evidence retained from the stale worktree: `evidence/cranelift-prototype-tests.txt` records `cargo test -p crunch-eval --lib --features cranelift-proto` -> `test result: ok. 41 passed`, `cargo test -p crunch-eval --lib` -> `test result: ok. 35 passed`, and `cargo test -p crunch --test examples_eval` -> `test result: ok. 4 passed`.

Fresh retry without rustc wrappers: 2026-04-27T12:46Z
Command: cargo test -p crunch-eval --lib
   Compiling nickel-lang-core v0.16.1
