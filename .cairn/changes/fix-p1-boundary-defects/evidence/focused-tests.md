# Focused evidence: P1 boundary fixes

## Focused regressions (passed)

All commands used `CARGO_TARGET_DIR=/tmp/mantle-fix-target2
CARGO_INCREMENTAL=0` inside `nix develop`.

| Command filter | Result |
| --- | --- |
| `cargo test -p nix-compat --lib signing_keys` | 4 passed; 0 failed |
| `cargo test -p snix-build --lib bwrap::capture::tests::` | 2 passed; 0 failed |
| `cargo test -p snix-build --lib bwrap::tests::initialize_` | 2 passed; 0 failed |
| `cargo test -p crunch-store --lib gc::tests::unsigned_retained_pathinfo_survives_real_gc` | 1 passed; 0 failed |
| `cargo test -p crunch-build --lib fetcher::tests:: -- --test-threads=1` | 100 passed; 0 failed |
| `cargo test -p crunch-build --lib worker::tests::cancellable_build_service_panic` | 1 passed; 0 failed |
| `cargo test -p mantle --bin mantle signing_key::tests::` | 12 passed; 0 failed |

Total: 122 focused tests passed, 0 failed.

Additional signing-key path coverage:

| Test | Result |
| --- | --- |
| `integration_build::casita_explicit_external_signing_key_outside_policy_cannot_publish` | 1 passed; 0 failed |
| `integration_build::casita_external_config_signer_requires_destination_policy_for_build_and_read` | 1 passed; 0 failed |
| `integration_build::casita_external_signer_policy_removal_rejects_fresh_lookup_verify_gc` | 1 passed; 0 failed |

The private-key fixtures in these tests now set mode 0600; the loader
correctly rejects the prior 0644 fixtures before reading.

Follow-on security boundaries:

| Test filter | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle signing_key::tests::` | 13 passed; 0 failed |
| `cargo test -p crunch-pipeline --lib private_build_work_root` | 3 passed; 0 failed |
| `cargo test -p crunch-build --lib fetcher::tests::extract_tar_strips_setuid_setgid_and_sticky_bits` | 1 passed; 0 failed |

The key test includes eight concurrent first-run callers converging on one
mode-0600 key; the pipeline tests cover creation, loose-mode rejection, and
symlinked-parent rejection. Tar mode fixtures cover 04755, 02755, 01777, and
04644. `mantle doctor` now includes a `work-root` check reporting the selected
sandbox root and free space at its nearest existing ancestor.

Executable fixed-output migration:

| Test filter | Result |
| --- | --- |
| `cargo test -p crunch-build --lib fetcher::tests::parse_fetch_executable` | 2 passed; 0 failed |
| `cargo test -p crunch-build --lib fod::tests::` | 21 passed; 0 failed |
| `cargo test -p crunch-build --lib fetch_build_service::tests::` | 26 passed; 0 failed |
| `cargo test -p mantle --bin mantle foreign_graph_compiler::tests::compiler_lowers_ordered_executable_download_to_mantle_fetch_facts` | 1 passed; 0 failed |

Flat executable declarations now fail with `ExecutableRequiresNar`; flat
executable nodes fail with `FodFlatExecutable`; NAR-addressed executable
verification passes and its digest differs from the non-executable node.

Remote admission trust:

```text
cargo test -p crunch-store --lib remote_substitution_ -- --test-threads=1
```

Observed: 17 passed; 0 failed. The new
`remote_substitution_rejects_untrusted_full_pathinfo_before_publication`
regression proves an untrusted nonempty signature list is rejected before
local PathInfo persistence or export, while a trusted candidate still
substitutes.

Protected-exec non-claim (#79, partial):

- Docs and ADR 0164 now state that continue-based user notification
  re-resolves the pathname after inspection, so audit events are observations
  of inspected paths under a cooperative-tracee assumption, not proof of the
  executed bytes.
- Audit files now use `mantle-protected-exec-audit-v2` with a required
  `non_claims` field; the provider rejects a missing, wrong, or legacy schema.
- Receipt-schema regression:
  `cargo test -p mantle --lib protected_exec::tests::protected_exec_audit_receipt_requires_check_to_use_non_claim`
  passed (1 passed; 0 failed). The full `protected_exec::tests` suite passed
  36 tests; the provider suite passed 29 tests.
- A deterministic rename-swap fixture
  (`seccomp_supervisor_continue_race_executes_replaced_bytes`) holds the
  supervisor after hashing, replaces the admitted path with `/bin/false`, and
  confirms the kernel executes the replaced bytes while the audit event records
  the inspected `/bin/true` digest. This records the documented limit rather
  than claiming prevention.

Formatting: `rustfmt --edition 2024 --check` on the edited first-party leaf
files passed. Vendored files are outside the first-party fmt selection and
retain their existing formatting.

Scoped Clippy:

```text
CARGO_TARGET_DIR=/tmp/mantle-fix-target2 CARGO_INCREMENTAL=0 \
  nix develop -c cargo clippy -p crunch-build -p crunch-store -p mantle \
  --all-targets --no-deps -- -D warnings
```

Observed: `Finished dev profile` with no warnings.

Tiger Style:

```text
nix run .#tigerstyle -- check
```

Observed: the `crunch-tigerstyle` derivation could not be built because the
configured remote builder had no connected, non-draining `x86_64-linux`
worker. A direct `nix develop -c cargo-tigerstyle check` then reached
`crunch-remote-core` and failed on pre-existing findings there
(`has_capacity` unit suffix, `features_valid` predicate, `has_no_whitespace`
negated predicate) before reporting a result for the touched packages. These
are P2 #78 findings, not results introduced by this change; the scoped Clippy
and rustfmt results above cover the touched first-party files.

Edited test targets also compile:

```text
cargo test -p mantle --test store_archive_cli --no-run
cargo test -p mantle --test store_gc_cli --no-run
cargo test -p mantle --test attest_cli --no-run
cargo test -p mantle --test kernel_bundle_oci_registry_cli --no-run
cargo test -p mantle --test release_cli --no-run
```

Observed: all five test executables built.

## Pre-existing snix-build cancellation regression

Command (modified tree):

```text
CARGO_TARGET_DIR=/tmp/mantle-fix-target2 CARGO_INCREMENTAL=0 \
  nix develop -c cargo test -p snix-build --lib \
  buildservice::bwrap::tests::dropping_cancellable_build_future_reaps_escaped_child \
  -- --test-threads=1 --nocapture
```

Observed: `FAILED` at
`vendor/snix-build/src/buildservice/bwrap.rs:1216` with
`dropped cancellable future allowed escaped builder output`.

Control: the same test was run after restoring `vendor/snix-build` to `HEAD`
(`git checkout -- vendor/snix-build`) with a separate target directory. It
failed identically at the same assertion. The failure is therefore present
before this change; it is not accepted as fixed by this change.

The suite also leaves
`buildservice::bwrap::tests::aborting_caller_then_panicking_supervisor_retains_owned_teardown`
running beyond 60 seconds under the same environment. That test is listed as a
separate pre-existing liveness blocker.

## Flake check evaluation

Command:

```text
nix flake check --no-build -L
```

Observed: evaluation reaches `packages.x86_64-linux.spacewasm-reference-bundle`
and fails with `error: path
'/nix/store/1r4xh8sqinzjy3lj6yralgfn1k35khxj-mantle-spacewasm-bundler-source.drv'
is not valid`. This is recorded as a blocker for a full `nix flake check`
outcome; the SSH artifact-source drift that blocked earlier evaluation is no
longer reported.

## Cairn validation

Command:

```text
nix develop -c cairn validate --root .
```

Observed: exit 0 with no `fix-p1-boundary-defects` validation messages after
the change package and requirement references were added.
