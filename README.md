# crunch

## Development

```bash
nix develop          # enter devshell
cargo check          # verify compilation
cargo nextest run    # run tests
nix flake check      # full CI (build + test + clippy + fmt)
```

## Verification

```bash
verus verus/example_spec.rs
```
