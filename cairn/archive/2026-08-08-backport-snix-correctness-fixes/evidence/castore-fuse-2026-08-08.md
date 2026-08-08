# Castore and FUSE correctness

## Implemented boundaries

The pure castore directory-size core now sums one contribution per child. A directory child contributes its own size plus one entry. A file or symlink contributes one entry. Checked arithmetic preserves the construction invariant.

FUSE directory entries now use `DT_DIR`, `DT_REG`, and `DT_LNK`. They no longer use `S_IF*` stat mode values. Root and node attributes now use the adopted nonzero link count.

## Coverage

Tests cover an empty directory, a mixed directory, the duplicate-contribution regression, overflow rejection, all supported FUSE node kinds, explicit rejection of `S_IF*` values, and nonzero root and node link counts.

## Validation

```text
cargo test -p snix-castore --lib
254 passed; 0 failed; 1 ignored

cargo test -p snix-castore --lib --features fs
260 passed; 0 failed; 1 ignored

cargo check -p snix-castore --features fs
Finished successfully.

cargo fmt --check -p snix-castore -v
Finished successfully.

git diff --check
Finished successfully.
```
