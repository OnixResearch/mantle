# Toolchain pin evidence

## Direct Cargo/rustup entry point

```text
cargo -Vv
cargo 1.96.0-nightly (a357df4c2 2026-04-03)
commit-date: 2026-04-03

rustc -Vv
rustc 1.96.0-nightly (c75612477 2026-04-07)
commit-hash: c756124775121dea0e640652c5ee3c89e3dd0eb4
commit-date: 2026-04-07
```

## Checked-in Nix shell

```text
nix develop -c rustc -Vv
rustc 1.96.0-nightly (c75612477 2026-04-07)
commit-hash: c756124775121dea0e640652c5ee3c89e3dd0eb4
commit-date: 2026-04-07

nix develop -c cargo -Vv
cargo 1.96.0-nightly (a357df4c2 2026-04-03)
commit-hash: a357df4c26fc14514e66aae2a269456b5545c7db
```

Direct and Nix entry points agree on the primary compiler version and commit.

## Pinned workspace check

```text
CARGO_TARGET_DIR=/tmp/mantle-pinned-target nix develop -c cargo check \
  --locked --offline -p crunch-store -p crunch-build -p crunch-pipeline \
  -p crunch-eval -p crunch-glue --lib
```

Observed: `Finished dev profile` with no errors. A bare host Cargo invocation
resolves the same compiler but cannot link because the dev-shell linker is not
on the bare host PATH; that is the documented tool environment, not a
toolchain-version mismatch.
