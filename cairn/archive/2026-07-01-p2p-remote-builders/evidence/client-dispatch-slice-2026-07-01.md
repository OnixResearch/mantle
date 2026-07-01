# Evidence: remote client dispatch slice

Date: 2026-07-01

## Question

Can `mantle build --builder ... --ticket ...` evaluate/lower locally, send concrete derivation payloads over the stdio remote-build protocol, and import signed remote outputs without reopening the local store before the child builder exits?

## Inspected evidence

- `src/main.rs` adds build flags for remote dispatch: `--builder`, `--ticket <id:secret>`, optional `--builder-program`, repeated `--builder-arg`, repeated `--trusted-builder-key`, and `--remote-build-time-secs`.
- With no `--builder-program`, the client defaults to spawning the current Mantle binary as `remote serve --binding stdio-once --executor local-build`, forwarding the selected state dir, physical store, and logical store prefix.
- `src/remote_build.rs` adds pure client planning helpers for ticket-token parsing, concrete derivation request construction, expected-output extraction, input-ref derivation, stdio command framing, and trusted-output-key validation.
- `crates/crunch-glue` now serializes `CrunchDerivation`/input payloads back to JSON so locally evaluated Nickel derivations can cross the remote concrete request boundary and round-trip into the remote local-build executor.
- The client shell evaluates/lower roots locally, rejects non-empty input refs for now, spawns the stdio child, validates output frames against the original request and trusted builder key names, opens the local store only after the child exits, and imports admitted signed outputs through `StoreHandle`.
- The store-open order was smoke-tested: opening the local import store before spawning the child caused `directories.redb` lock contention; the current shell opens/imports after each child exits.

## Decision

Progress slice accepted, not full drain. The CLI now has a real stdio remote-dispatch path for derivations that already have declared output paths and no missing input refs. Full drain still requires CAS/source input upload, content-addressed provisional request support, production P2P transport, and first-class build-report integration for remote substitutions.

## Owner

Mantle remote-build owner.

## Next action

1. Materialize non-empty input refs into the remote store instead of rejecting them in the client shell.
2. Teach client request planning to support content-addressed derivations whose expected final output paths are not known before execution.
3. Promote the remote client report into the stable top-level `crunch-build-report-v1` JSON schema.

## Validation

```text
cargo test -p crunch-glue

test result: ok. 78 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests crunch_glue: ok. 0 passed; 0 failed.
```

```text
cargo test -p mantle --bin mantle remote_build::

test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 966 filtered out; finished in 0.01s
```

```text
cargo test -p mantle --bin mantle build_cli_accepts_remote_builder_ticket_dispatch_flags

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1021 filtered out; finished in 0.00s
```

```text
cargo test -p mantle --test remote_stdio_cli

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

```text
cargo build -p mantle

Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.37s
```

```text
manual smoke: /home/brittonr/.cargo-target/debug/mantle --json --store <tmp>/store --state-dir <tmp>/state build <tmp>/input-addressed.ncl --builder local-builder --ticket ticket-1:secret-1 --remote-build-time-secs 60

Result: passed after adding bwrap to PATH. Output was a `mantle-remote-client-build-v1` JSON report with one imported output and a delta transfer report signed by `crunch-britton-desktop-1`.
```

```text
git diff --check
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal p2p-remote-builders --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design p2p-remote-builders --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks p2p-remote-builders --root .

final gate output: stage=tasks valid=true verdict=PASS
```
