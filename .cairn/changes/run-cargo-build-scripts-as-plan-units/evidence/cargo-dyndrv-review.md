# Evidence: cargo-dyndrv source review

## Source

- Article: "cargo-dyndrv: A Beginning", Obsidian Systems, 2026-09-16,
  https://blog.obsidian.systems/cargo-dyndrv-a-beginning/
- Repository: https://github.com/obsidiansystems/cargo-dyndrv
- Reviewed revision: `d5245b85f82627fcaf041e2c57c8aeb7d57d20fe` (2026-09-16),
  read over HTTPS during the 2026-09-25 session. No code was copied.

## Mechanism mapped by this change

- `target-env` runs `rustc --print=cfg` and `rustc --print=host-tuple` in its
  own derivation and writes the results as environment variables to a file;
  the article places it in a separate derivation to maximize caching.
- `build-wrap` runs a compiled build script and saves its directives into
  files. `cargo-dyndrv/src/main.rs` gives each execution derivation `out`
  (`OUT_DIR`) and `flags` outputs with `args-immediate`, `args-transitive`,
  `env`, and `metadata` files.
- `env-wrap` applies environment files and then runs rustc or `build-wrap`.
  Dependent rustc calls load `@flags/args-immediate` and
  `@flags/args-transitive` argument files.
- Execution derivations receive `HOST_CC`, `HOST_CXX`, `CC`, `CXX`,
  `CARGO_FEATURE_*`, `OPT_LEVEL`, and a PATH that includes the C compilers.
- `extern.json` maps package ids to `inputs`, `env`, and `path`. `writeExtern`
  computes `inputs` from the declared strings with `exportReferencesGraph`,
  because a generated derivation receives no evaluator string context.

## Adaptation boundary

Mantle keeps the unit shape (target facts, execution units with `out` and
`flags`, argument files) but uses one Mantle-built helper with modes instead
of three tools, enumerates supported directives, and fails closed on others.
Native inputs are typed role-named Nickel declarations resolved by the
producer, not inferred from string references.
