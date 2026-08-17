# Requested-digest shell guard

## Implemented boundary

`crates/crunch-store/src/path_identity.rs` now owns a pure requested-digest comparator.

`crates/crunch-store/src/handle.rs` applies this comparator immediately after an alternative `PathInfoService` returns `PathInfo`. The guard runs before local PathInfo persistence, sidecar creation, success reporting, output maps, export, root registration, or advisory-hit publication.

The stable error class is:

```text
remote-pathinfo-request-identity-mismatch
```

The Snix HTTP service remains the first guard. This first-party guard is independent of that service implementation.

## Positive and negative coverage

The pure comparator accepts equal store-path digests. It rejects a different digest and retains the requested and observed paths in the error.

The mutation-counting substitution test uses an alternative service that returns signed `PathInfo` for another digest. The test proves:

- one remote lookup occurs;
- zero local or remote PathInfo writes occur;
- no output node, built output, or success report appears;
- no sidecar, export, retained root, or advisory entry appears; and
- the shell returns the stable mismatch error.

The delta transport does not use the alternative `PathInfoService` boundary. It retains its separate exact-path check in `persist_and_export_signed_output`.

## Validation

Pre-change command:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib try_substitute_remote
```

Result:

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 335 filtered out; finished in 0.03s
```

Post-change focused commands:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib mismatched_remote_pathinfo_causes_no_substitution_mutations
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib requested_path_digest
```

Results:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 340 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 339 filtered out; finished in 0.00s
```

Full package command:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib
```

Result:

```text
test result: ok. 341 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
```

These checks also passed:

```text
CARGO_TARGET_DIR=/tmp/mantle-backport-snix-i4-target SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo clippy -p crunch-store --lib --no-deps -- -D warnings
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo fmt --check -p crunch-store -v
git diff --check
```

A shared-target Clippy retry failed during an incremental-cache rename with `No such file or directory`. It emitted no source diagnostic. The isolated-target command above passed.

Lifecycle commands:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks backport-snix-correctness-fixes --root .
```

Results:

```text
validate: "valid": true
tasks gate: "verdict": "PASS"
tasks gate receipt: c52134a7e5ce8c9d69e59279d6e3ecc79879c20cd1c94ca23c9a853cf25dd91c
```
