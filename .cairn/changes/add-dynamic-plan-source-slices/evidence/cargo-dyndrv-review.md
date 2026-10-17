# Evidence: cargo-dyndrv source review

## Source

- Article: "cargo-dyndrv: A Beginning", Obsidian Systems, 2026-09-16,
  https://blog.obsidian.systems/cargo-dyndrv-a-beginning/
- Repository: https://github.com/obsidiansystems/cargo-dyndrv
- Reviewed revision: `d5245b85f82627fcaf041e2c57c8aeb7d57d20fe` (2026-09-16),
  read over HTTPS during the 2026-09-25 session. No code was copied.

## Mechanism mapped by this change

- `cargo-dyndrv/src/main.rs` adds each crate root to the store as its own
  object (`store::add_to_store_nar(crate_root, "<name>-src")`) before it
  writes that crate's derivation.
- Inside a derivation, that call goes through `builder-rpc-v0`: a restricted
  Nix daemon socket that accepts only store-object creation (`AddToStore`,
  `AddToStoreScanning`) and `SubmitOutput`.
- The effect is one source object per crate, so editing one crate leaves the
  other crates' derivations unchanged.

## Adaptation boundary

Mantle keeps store creation out of the sandbox. The producer writes slice
declarations into its plan, and the worker admits declared subtrees of the
producer's own outputs after the producer completes, under named limits and
with an expected digest. No daemon protocol, socket, or recursive store access
is exposed to builders.
