# Baseline evidence

## Question

Why does `artifact-auth-radicle-cutover` fail on current `origin/main`?

## Command

```text
nix build .#checks.x86_64-linux.artifact-auth-radicle-cutover --no-link -L
```

## Result

```text
this derivation will be built:
  /nix/store/vs5l8b32qv53szxvy0sm7yk99n7a600p-mantle-artifact-auth-radicle-cutover.drv
cannot build on 'ssh-ng://root@10.10.10.1': error: failed to start SSH connection to '10.10.10.1'
building '/nix/store/vs5l8b32qv53szxvy0sm7yk99n7a600p-mantle-artifact-auth-radicle-cutover.drv'...
error: Cannot build '/nix/store/vs5l8b32qv53szxvy0sm7yk99n7a600p-mantle-artifact-auth-radicle-cutover.drv'.
       Reason: builder failed with exit code 1.
       Output paths:
         /nix/store/2w21wxijm90b9c37i2bgzgs5897cr1wb-mantle-artifact-auth-radicle-cutover
```

The remote-builder warning is non-blocking. The local derivation fails.

## Observed BLAKE3 values

```text
496123770c3e21dea731262771046424fc3b534f7279004d3ca52b05bc8a7c79  crates/crunch-action-result-core/Cargo.toml
9c72bc4929f9f836c0b767d6b604b5e6d3fb97e4f1b61505dbc28238b107b6ce  crates/crunch-build/Cargo.toml
19daded89bc2d1e15bb2f4d44b827a9f2a7b3be8909e0bbd7d06efeb0fc6d535  Cargo.lock
35f225f391fabe20bc597a3120c2058cd9e5389754492b43ec3e092e1597d05e  flake.nix
44015a40ba7858db1fa8df4f2bcca11499b04b90be1f8eb8ae3c353009203274  flake.lock
dd35d8ca96c8a7beb159d65afc78608eefc5b8818bb7e3f1ee4e467ea2adde97  evidence/radicle/artifact-auth-cutover-v1.json
```

The receipt expects historical `flake.nix` BLAKE3 `495ef4816453fc6ab58617dde209735e55953485c8c4c45d600d42c14b0560b3`. All other live file bindings match.

## Decision

Keep the historical receipt immutable. Replace only the live whole-file `flake.nix` comparison with scoped current source validation.

## Owner

Mantle owns the Nix gate and current source-agreement validation.

## Next action

Implement the scoped positive and negative source-declaration checks.
