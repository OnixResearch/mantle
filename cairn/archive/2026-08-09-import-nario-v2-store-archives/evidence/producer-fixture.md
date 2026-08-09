# Pinned Nario v2 producer fixture

## Authority

- Repository: `DeterminateSystems/nix-src`
- Tag: `v3.12.0`
- Revision: `9512828397f684d0f732ea76b7631f69a0db34f7`
- Built producer: `/nix/store/2qky6r4y8lwl02yg10nxr3hqxb979w3l-nix-3.12.0/bin/nix`
- Positive archive: `fixtures/nario-v2/positive-single.nario`

## Fresh producer parity

The command exported the current fixture store path again. `cmp` returned success before the hash and list commands ran.

```text
$ producer --version
warning: unknown experimental feature 'wasm-builtin'
nix (Determinate Nix 3.12.0) 2.32.1

$ producer nario export --format 2 STORE_PATH > FRESH
warning: unknown experimental feature 'wasm-builtin'

$ cmp fixtures/nario-v2/positive-single.nario FRESH

$ producer hash file --type blake3 FRESH
warning: unknown experimental feature 'wasm-builtin'
blake3-BxoedO19Ca08+g5nyM5WdBc7/yiRBqwY4ML661BlUng=

$ producer nario list < FRESH
warning: unknown experimental feature 'wasm-builtin'
/nix/store/j3wdfhfzn69xrn6lkk7sm210yx8fp0k7-payload.txt: 144 bytes
```

The negative corpus consists of deterministic mutations of these exact producer bytes. `scripts/check-nario-v2-fixtures.rs` checks corpus parity and mutation distinctness.
