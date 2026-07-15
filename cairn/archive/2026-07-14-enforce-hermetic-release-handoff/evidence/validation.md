# Validation evidence

Recorded: 2026-07-12

## Focused positive and negative tests

All commands used isolated Cargo target and temp roots under `/tmp`.

```text
$ cargo test -q -p crunch-release-core
184 passed; 0 failed

$ cargo test -q -p crunch-bootstrap-core
70 passed; 0 failed

$ cargo test -q -p mantle --bin mantle cairn_handoff
3 passed; 0 failed

$ cargo test -q -p mantle --bin mantle onix_profile_
2 passed; 0 failed

$ cargo test -q -p mantle --bin mantle source_root_provider
23 passed; 0 failed

$ cargo test -q -p mantle --bin mantle bootstrap_source_root::
25 passed; 0 failed

$ cargo test -q -p mantle --bin mantle bootstrap_capabilities_is_discoverable
1 passed; 0 failed

$ cargo test -q -p mantle --bin mantle flake_check_workflow_has_one
1 passed; 0 failed

$ cargo test -q -p mantle --test release_cli cairn_handoff
2 passed; 0 failed

$ cargo test -q -p mantle --test release_cli onix_release_profile
3 passed; 0 failed
```

The covered negative cases include fabricated declared digests, tampered
bundle-local bytes, wrong role/schema pairs, empty required handoffs, duplicate
artifact/policy paths, receipt reuse after a release-manifest projection change,
authenticated-status promotion, missing Onix handoff evidence, and missing
strict deterministic evidence. Existing deterministic-core coverage in the
184-test release-core suite rejects impure mode, degraded audits, failed
isolation, missing perturbations/normalization controls, and output mismatch.

Changed-file formatting check:

```text
$ rustfmt --check --edition 2024 --config skip_children=true <changed Rust files>
PASS
```

## Production capability report

```text
$ mantle --json bootstrap capabilities
{
  "schema": "mantle-source-root-capability-report-v1",
  "observations": {
    "platform_supported": true,
    "host_c_compiler_available": true,
    "host_make_available": true,
    "host_tar_available": true
  },
  "operations": [
    {
      "operation": "bootstrap-source-root-materialization",
      "status": "supported-host-assisted",
      "capability_class": "host-assisted-source-materialization",
      "command": ["mantle", "bootstrap", "--source-root", "<source-root-manifest.json>"]
    },
    {
      "operation": "self-build-source-root",
      "status": "unsupported",
      "command": null,
      "blockers": ["Mantle self-build has no executable full-source source-root provider chain"]
    }
  ]
}
```

Both operation records carry host-influence notes and the non-claim that this is
not a full-source bootstrap.

## Authentication-dependency integration validation

The focused post-dependency rail passed:

```text
$ cargo test -p crunch-release-core
207 passed; 0 failed

$ cargo test -p crunch-release-core cairn_handoff
5 passed; 0 failed

$ cargo test -p mantle --bin mantle cairn_handoff
4 passed; 0 failed

$ cargo test -p mantle --bin mantle cairn_release_handoff
8 passed; 0 failed

$ cargo test -p mantle --test release_cli cairn_handoff
2 passed; 0 failed
```

Production-path coverage now includes exact dependency measurement during
planning and assembly, publication-plan inclusion, bundle-local remeasurement,
receipt binding, and rejection of tampered dependency bytes. Changed Rust files
pass the repository's focused `rustfmt --edition 2024 --config
skip_children=true` check.

## Cairn lifecycle validation

`cairn validate --root .` and the proposal, design, and tasks gates pass after
the dependency integration. No sync or archive command was run because the
final flake task remains blocked.

## Required flake-check attempts

The exact command evaluated 777 checks but the host could not publish build
results because its configured signing key is absent:

```text
$ nix flake check
error: opening file "/run/secrets/vars/nix-signing-key/key": No such file or directory
```

The bounded local retry disabled only unavailable host signing and remote
builders. It reached the enforced pre-existing product blocker:

```text
$ nix flake check --option secret-key-files '' --option builders ''
bootstrap blocker inventory: 40 findings across 4 classes,
396 evidence-backed suppressions, 0 promotion claims, enforce=true
FAIL: bootstrap blocker inventory is not clean; expected 0 findings and 0 promotion claims
```

This is exact closeout evidence, not a passing flake-check claim. The final
lifecycle task remains unchecked.

## External authentication dependency

See `external-authentication-dependency.md`. Cairn's
`authenticate-stack-provenance-inputs` package is archived at revision
`f4a1f8df0d430c1b9431358a388ac1d3c1a823ec`. Mantle now requires the exact
reviewed dependency receipt, measures it before staging, includes it in the
atomic publication plan, remeasures it during bundle verification, and binds it
into the validation receipt. Missing, stale, fabricated, and tampered dependency
identities fail focused core and production-path tests.

The status `archive-authentication-prerequisite-bound-v1` remains bounded:
Mantle does not independently re-run producer signatures or claim Cairn,
producer, source, build, or release correctness.

## Bootstrap inventory closeout rerun

On 2026-07-14 the bootstrap inventory blocker was repaired without removing
fail-closed diagnostic behavior or changing the clean-gate requirement. The
enforced local rail and the Nix check now report 0 findings, 434
evidence-backed suppressions, and 0 promotion claims. The corrected first-party
Nix format check also passes, and a fresh `bootstrap/gcc.ncl` validation passes
all warmups, the build, and host-leakage scanning after generated manpage
placeholders were removed from the bootstrap output contract.

The mandatory flake task remains blocked at the next independent frontier:
both exact and bounded `nix flake check` stop at the explicitly deny-level
Tiger Style consumer check with 418 violations across multiple first-party
crates. No lint level, package scope, or Tiger Style revision was weakened.
See `bootstrap-blocker-inventory-closeout.md` for commands, bounded claims, and
the closeout decision.

## Strict clock-syscall enforcement repair

On 2026-07-14 a follow-up audit found that deterministic release runs declared
clock-free observed effects but only normalized `SOURCE_DATE_EPOCH` and timezone
inputs. The proof sandbox now loads a BLAKE3-identified classic-BPF policy
through bubblewrap's inherited `--seccomp` file descriptor. The policy returns
`EPERM` for the bounded Linux clock, timer, and interval-timer syscall set and
allows unrelated syscalls. Filter construction is a deterministic core;
temporary-file creation, descriptor inheritance, and process launch remain in
the shell. The profile identity and isolation check set bind the enforced
policy before pass evidence is emitted.

Pre-change baselines:

```text
$ cargo test -p mantle --bin mantle release_reproducibility::tests::
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1477 filtered out

$ cargo test -p crunch-release-core
test result: ok. 207 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Focused post-change evidence:

```text
$ MANTLE_TEST_REAL_BWRAP=/nix/store/g7svy17fhkg2cq3q4lfzzc0mmsl3d8hq-bubblewrap-0.11.2/bin/bwrap \
    cargo test -p mantle --bin mantle 'proof_clock_seccomp::' -- --nocapture
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1489 filtered out

$ cargo test -p mantle --test release_cli \
    release_reproduce_writes_deterministic_proof_from_repeated_clean_runs \
    -- --exact --nocapture
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 139 filtered out

$ cargo test -p crunch-release-core
test result: ok. 207 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p mantle --bin mantle 'release_reproducibility::tests::'
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1482 filtered out

$ cargo -Zscript scripts/check-real-release-determinism-receipt.rs --self-test
real release determinism receipt checker self-test passed

$ rustfmt --check --edition 2024 <changed Rust files> && git diff --check
PASS
```

The kernel subprocess fixture proves a raw `clock_gettime` syscall receives
`EPERM` while `getpid` remains allowed. The real-bubblewrap fixture proves that
the exact generated policy survives descriptor inheritance and is applied to
the sandbox child. The production CLI fixture proves repeated clean runs still
produce a strict self-rebuild match and now carry `denies-clock-syscalls`
isolation evidence. This is bounded syscall-enforcement evidence; it does not
claim interception of non-syscall hardware or vDSO time sources, compiler
correctness, or universal reproducibility.

## Final flake and lifecycle closeout

Recorded: 2026-07-14

The independent Tiger Style and sequential package-test frontiers were repaired
without demoting lint levels, shrinking the first-party package scope, removing
negative tests, weakening Cargo checksum validation, or promoting host
observations into product claims.

The package rail completed successfully after exercising the complete Mantle
package test suite:

```text
$ nix build path:$PWD#checks.x86_64-linux.crunch --no-link -L \
    --option secret-key-files '' --option builders ''
PASS
```

The bounded full flake command then built and passed all 13 checks:

```text
$ nix flake check path:$PWD -L \
    --option secret-key-files '' --option builders ''
running 13 flake checks...
all checks passed!
```

A same-tree exact host-configured rerun completed from the realized check set
without requesting the absent signing key:

```text
$ nix flake check path:$PWD -L
running 0 flake checks...
all checks passed!
```

The exact rerun proves that the current realized check graph is accepted by the
host configuration. It does not prove that this host can publish a newly built
uncached output while its configured signing-key path is absent, and it does
not promote remote-builder availability into a Mantle claim.

Fresh lifecycle validation used the canonical Cairn binary and policy rather
than Mantle's stale generated policy:

```text
$ /home/brittonr/git/OnixResearch/cairn/target/debug/cairn validate \
    --root . \
    --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
valid: true; issues: []

$ /home/brittonr/git/OnixResearch/cairn/target/debug/cairn gate proposal \
    enforce-hermetic-release-handoff --root . --policy <canonical-policy>
verdict: PASS; valid: true; issues: []

$ /home/brittonr/git/OnixResearch/cairn/target/debug/cairn gate design \
    enforce-hermetic-release-handoff --root . --policy <canonical-policy>
verdict: PASS; valid: true; issues: []

$ /home/brittonr/git/OnixResearch/cairn/target/debug/cairn gate tasks \
    enforce-hermetic-release-handoff --root . --policy <canonical-policy>
verdict: PASS; valid: true; issues: []
```

The direct first-party quality rails also passed on the closeout tree:

```text
$ ./scripts/check-first-party-tigerstyle.sh
Finished `dev` profile

$ ./scripts/check-first-party-clippy.sh
Finished `dev` profile

$ cargo fmt --check <workspace.metadata.tigerstyle.default_scope>
PASS

$ nixfmt --check flake.nix
PASS

$ cargo test --release -p mantle --bin mantle -- --test-threads=1
test result: ok. 1536 passed; 0 failed; 0 ignored; 0 measured
```

The root libtest rail is deliberately serialized because subprocess-heavy
fixtures exercise executable replacement and isolated environment seams; the
Nix package and nextest rails independently cover their configured parallel
execution shapes.

The initial sync dry-run correctly failed closed because both change specs were
legacy full accepted-spec projections. After converting them to explicit
`ADDED` operations, release provenance still failed closed on the existing
`final_validation` ID. That operation was reviewed as `MODIFIED`, preserving
its requirement identity while replacing its earlier narrower fixture text.
The final sync dry-run reported `blocked: false` with two actions, and execution
reported `mutated: true`, `blocked: false`, no reasons, and receipt hash
`7fbfa1b9389e6db934d0b4ab543fe0f38048f67607ec3975c9170f0456a01949`.
The accepted specs were then inspected for every added ID and the modified
`final_validation` text before post-sync validation.

The previously archived authentication dependency remains an explicit bounded
input. This closeout does not claim Cairn producer correctness, universal
reproducibility, full-source bootstrap closure, unavailable remote-builder
execution, or uncached host publication through an absent signing key.

## Post-archive validation

The exact post-archive command was:

```text
$ /home/brittonr/git/OnixResearch/cairn/target/debug/cairn validate \
    --root . \
    --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Its complete unedited JSON output is committed beside this file as
`post-archive-validation.json`. It reports `valid: true`, empty repository,
spec, substance, and cross-repository issue lists, 28 accepted specs validated,
and one remaining active change: the intentionally blocked bounded
KernelScript experiment.
