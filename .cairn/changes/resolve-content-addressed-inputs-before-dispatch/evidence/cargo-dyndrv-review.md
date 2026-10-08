# Evidence: cargo-dyndrv source review

## Source

- Article: "cargo-dyndrv: A Beginning", Obsidian Systems, 2026-09-16,
  https://blog.obsidian.systems/cargo-dyndrv-a-beginning/
- Repository: https://github.com/obsidiansystems/cargo-dyndrv
- Reviewed revision: `d5245b85f82627fcaf041e2c57c8aeb7d57d20fe` (2026-09-16),
  read over HTTPS during the 2026-09-25 session. No code was copied.

## Mechanism mapped by this change

- Every generated derivation declares its outputs as floating
  content-addressed (`DerivationOutput::CAFloating` with NAR SHA-256) in
  `cargo-dyndrv/src/main.rs`.
- Dependents refer to those outputs through downstream placeholders
  (`Placeholder::ca_output(drv_path, output)`), which Nix replaces with
  realized paths when it resolves the dependent before building it.
- The repository README states the purpose: content addressing "can reduce
  rebuilds when a crate's source code or build flags change but the resulting
  binary is identical". The reference requires the Nix `ca-derivations`
  experimental feature for this.

## Adaptation boundary

Mantle resolves derivations with CA inputs before dispatch and keys reuse on
the resolved identity, with signed realisation records under Mantle's PathInfo
trust policy. It does not adopt Nix realisation wire formats or downstream
placeholder encodings; plan placeholders keep Mantle's `{{mantle-...}}`
grammar.
