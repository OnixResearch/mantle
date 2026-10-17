# Evidence: cargo-dyndrv source review

## Source

- Article: "cargo-dyndrv: A Beginning", Obsidian Systems, 2026-09-16,
  https://blog.obsidian.systems/cargo-dyndrv-a-beginning/
- Repository: https://github.com/obsidiansystems/cargo-dyndrv
- Reviewed revision: `d5245b85f82627fcaf041e2c57c8aeb7d57d20fe` (2026-09-16),
  read over HTTPS during the 2026-09-25 session. No code was copied.

## Mechanism mapped by this change

- The article's consumers reach a generated derivation's output through
  `builtins.outputOf`, which records a multi-level output reference in the
  consumer's input set at evaluation time.
- `cargo-dyndrv/src/main.rs` ends by calling `store::submit_wrapper` with the
  `ln` tool for every root: it creates an intermediate derivation with the
  required output name and symlinks the generated root's output into it,
  because dynamic derivations impose strict output-name rules.

## Adaptation boundary

Mantle consumers reference plan roots by producer derivation, declared plan
output, root unit id, and unit output. The consumer's identity binds that
reference at evaluation, and the worker binds the concrete unit after plan
acceptance. Roots are addressed by unit id, so no naming wrapper derivation is
needed, and no Nix dynamic-output wire format is involved.
