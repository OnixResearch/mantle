# Evidence: cargo-dyndrv source review

## Source

- Article: "cargo-dyndrv: A Beginning", Artemis Tosini, Obsidian Systems,
  2026-09-16, https://blog.obsidian.systems/cargo-dyndrv-a-beginning/
- Repository: https://github.com/obsidiansystems/cargo-dyndrv
- Reviewed revision: `d5245b85f82627fcaf041e2c57c8aeb7d57d20fe` (2026-09-16),
  read over HTTPS during the 2026-09-25 session: `README.md`,
  `cargo-dyndrv/src/main.rs`, and `cargo-dyndrv/src/unit_graph.rs`. No code
  was copied.

## Mechanisms observed

- Planning: `unit_graph.rs` runs Cargo with `-Z unstable-options --unit-graph`
  and rejects unit-graph versions other than 1. `main.rs` joins units with
  `cargo metadata` packages by package id.
- One derivation per unit: build units become rustc derivations;
  `run-custom-build` units become build-script execution derivations with
  `out` and `flags` outputs.
- Sources: each crate root is added to the store as `<name>-src` from inside
  the builder through `builder-rpc-v0`, a restricted daemon socket.
- Outputs: every generated output is floating content-addressed, and
  dependents reference them through downstream placeholders.
- Library dependencies: `--extern` points at `.rmeta` for library consumers
  and `.rlib` for binaries and proc macros; every transitive dependency is a
  declared input with its own `-L dependency=` flag.
- Metadata: `-C metadata` comes from `std::hash::DefaultHasher` over
  dependency derivation paths and the unit description.
- Build scripts: `target-env`, `build-wrap`, and `env-wrap` helpers plus rustc
  `@` argument files carry cfg, environment, link, and `links` metadata.
- Native inputs: `extern.json` maps package ids to inputs, environment, and
  PATH entries; `writeExtern` derives inputs with `exportReferencesGraph`.
- Roots: `submit_wrapper` creates an `ln` wrapper derivation per root to
  satisfy dynamic-derivation output naming.
- Future work named by the article: starting dependents after `.rmeta`
  instead of `.rlib`.

## Adaptation boundary

This change adopts the unit model: Cargo plans in a build step and every unit
becomes one derivation. It rejects `.drv` output, the builder store socket,
wrapper derivations for naming, and process-local metadata hashing. Sibling
changes own the remaining mechanisms: `add-dynamic-plan-source-slices` for
per-package sources, `bind-static-inputs-to-dynamic-plan-roots` for static
consumers, `resolve-content-addressed-inputs-before-dispatch` for early
cutoff, and `run-cargo-build-scripts-as-plan-units` for build scripts and
native inputs. Pipelining is a non-goal with a recorded revisit trigger.
