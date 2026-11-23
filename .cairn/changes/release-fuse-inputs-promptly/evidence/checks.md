# Checks for release-fuse-inputs-promptly

Environment (from the follow-ups spec, with the FUSE wrapper path added):

```
export PATH=/run/wrappers/bin:/nix/store/mvj6y8k3xs35cj7kq937xi44q4hl3r93-bubblewrap-0.12.0/bin:/nix/store/2msw5nclhpsm80z65dqmjiv8kkras989-clang-wrapper-21.1.8/bin:$PATH
export RUSTC_WRAPPER= SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/datapool/mantle-store/followups-20261122/mantle-target
export TMPDIR=/datapool/mantle-store/followups-20261122/mantle-tmp
```

`cargo test -p snix-castore` needs the crate's `fuse` feature; `--features fs` compiles the FUSE
module out and selects 0 tests.

## Diagnosis inputs

- `stall-diag-README.md` — the stall diagnosis, including the holder: the Nix 2.36.0 daemon's saved
  sandbox mount namespace (`sandboxMountNamespace`).
- `stall-2-eu-stack.txt` — the `eu-stack` sample of the blocked tokio worker and the four
  `fuse_server` threads.
- `conn78.txt` — `/sys/fs/fuse/connections/78` disappearing at 19:06:11.708.

## Formatting

`rustfmt.log`:

```
$ rustfmt --check --edition 2024 vendor/snix-castore/src/fs/fuse/mod.rs vendor/snix-castore/src/fs/fuse/tests.rs
EXIT=0
```

## Regression test

`fuse-regression-with-wake.log` — the new test with the fix:

```
test fs::fuse::tests::unmount_with_held_descriptor ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 290 filtered out; finished in 0.05s

EXIT=0
```

`fuse-regression-without-wake.log` — the same test with the `wake()` call temporarily removed (the
old code), recorded before the call was restored:

```
thread 'fs::fuse::tests::unmount_with_held_descriptor' (3116645) panicked at vendor/snix-castore/src/fs/fuse/tests.rs:252:13:
unmount did not return within 10s while a descriptor into the mount was open
test fs::fuse::tests::unmount_with_held_descriptor ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 290 filtered out; finished in 10.17s

EXIT=101
```

The test returned from its 10 s bound instead of hanging: the timeout branch drops the held
descriptor, joins the unmount thread, and then fails.

## Existing FUSE tests

`fuse-suite.log`:

```
running 24 tests
...
test fs::fuse::tests::unmount_with_held_descriptor ... ok
test fs::fuse::tests::mount ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 267 filtered out; finished in 0.21s

EXIT=0
```

## Route and foreign-import CLI suites

`cli-suites.log`:

```
$ cargo test -p mantle --test cache_inputs_build_route --test foreign_import_cli
...
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.77s
...
test result: ok. 18 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.61s

EXIT=0
```

The ignored test requires `MANTLE_TEST_FIX_BACKEND_BINARY`, `MANTLE_TEST_NIX_INSTANTIATE_BINARY`, and
a reachable Nix daemon; it is ignored on this host for the same reason as before the change.

## Mantle binary

`mantle-build.log`:

```
$ cargo build -p mantle --bin mantle
   Compiling mantle v0.1.0 (/home/brittonr/wt/mantle-followups)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 21s
EXIT=0
```

## Cairn and diff checks

`cairn-validate.log`, `gate-proposal.log`, `gate-design.log`, `gate-tasks.log`, `diff-check.log`:

```
$ cairn validate --root .
  "issues": [],
EXIT=0

$ cairn gate proposal release-fuse-inputs-promptly --root .
  "issues": [],
  "verdict": "PASS",
EXIT=0

$ cairn gate design release-fuse-inputs-promptly --root .
  "issues": [],
  "verdict": "PASS",
EXIT=0

$ cairn gate tasks release-fuse-inputs-promptly --root .
  "issues": [],
  "verdict": "PASS",
EXIT=0

$ git diff --check
EXIT=0
```

`cairn validate` reports an empty `issues` array (0 issues); each gate reports
`"verdict": "PASS"` with an empty `issues` array and exits 0.
