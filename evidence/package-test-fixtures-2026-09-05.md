# Package fixture repair: blocked full gate

## Disposition

The fixture repair branch is reviewable, but the default package remains blocked.
Do not integrate this branch into `main` or admit it downstream as a passing package.
No production evaluator, ptrace supervisor, frontend admission, or trust policy changed.
No existing Cairn task was checked or archived.

Base source: `079f02853a84114b33d8e91583c7456c16f50613`.
Latest tested source: `4a433bf1c0662f423b6668cb1afe57070613d92c`.
Source commits `026d4d6`, `52aed79`, and `4a433bf` retain each repair stage.

## Repaired scope

- Ptrace fixtures select the explicit package shell without fallback on invalid input.
  Parent and descendant commands bind the same executable, including spaces and quotes.
- Package, nextest, and development inputs declare Python for the existing patch test.
- OCI fixture assertions match the existing unsupported-kind diagnostic and require no store mutation.
- Fault-injection cases use a separate debug executable. Production release guards remain unchanged.
- Documentation assertions match the existing signer, NAR, and provenance wording.
- Cargo checks use `--no-fail-fast`. Any failed target still fails the package.

## Native package observations

The exact local Git source ran with a 60-minute cap, two cores, and one job:

```text
nix build 'git+file:///home/brittonr/git/OnixResearch/mantle?ref=fix/package-test-fixtures-20260905&rev=4a433bf1c0662f423b6668cb1afe57070613d92c#default' --no-link --print-out-paths --builders '' --cores 2 -j 1 -L
```

The run finished in 38 minutes and 30 seconds. Nix returned one after builder
status `101`. It did not time out. Cargo continued through the selected package
targets and doc tests. Its final summary reports nine failed targets.

Selected passing target summaries (abbreviated):

```text
library: test result: ok. 188 passed; 0 failed; 0 ignored
crunch: test result: ok. 2560 passed; 0 failed; 72 ignored
mantle: test result: ok. 2560 passed; 0 failed; 72 ignored
evaluator_budget_cli: test result: ok. 15 passed; 0 failed; 0 ignored
example_projects: test result: ok. 28 passed; 0 failed; 5 ignored
```

The evaluator target includes a passing
`release_binary_ignores_fault_injection_environment` control.
The repaired documentation fixture also passes. Ignored tests are not execution
proof. Nested subprocess reports are not additional target totals.

## Remaining failure inventory

| Target | Failed tests | Observed blocker |
| --- | ---: | --- |
| `examples_inventory` | 1 | Catalog omits `distributed_eval_assess.rs` and `picolibc_compare.rs`. |
| `foreign_import_cli` | 3 | A fixture has inconsistent output/environment paths. Two expected diagnostic classes differ from current admission results. |
| `integration_build` | 1 | The base-only source layer selection is absent from the observed selection list. |
| `machine_schema_contracts` | 1 | The inventory contains 33 contracted surfaces. The test expects 21. |
| `offline_build_runbook_docs` | 1 | The runbook no longer contains an expected exact non-claim sentence. |
| `operator_diagnostics` | 1 | Checked operator inventory lacks the required `platform_profiles` field. |
| `remote_transfer_production` | 1 | Status reports `capture-root-unavailable` where the fixture expects `captured`. |
| `removed_system_cli` | 2 | The scan exceeds its fixed file bound. A rejection uses an eager-evaluation diagnostic instead of the expected deserialization text. |
| `store_gc_cli` | 1 | The base-layer info query returns `no PathInfo matching 'cli-base'`. |

These observations do not establish every root cause. In particular, a missing
selection or changed receipt cannot be repaired by weakening its assertion.
The next change needs a scoped contract review and focused baselines for these
inventory, admission, store-layer, and receipt boundaries.

## Other checks

Focused tests passed: 14 ptrace, 9 OCI, 7 frontend-store, and 1 OpenSSL patch.
The evaluator debug integration target passed all 14 tests. The corrected
example documentation fixture passed its focused test. Strict Clippy passed
for the changed binary/test scope. Leaf Rust formatting, Nix formatting, and
`git diff --check` passed. The pinned Nix Tiger Style check passed at the latest
tested source.

A separate host-debug all-target diagnostic reached its 15-minute timeout.
It recorded seccomp observer/deep-descendant and invalid-artifact failures in
unchanged files. The native release suites passed those seccomp cases.
This report does not claim a passing host-debug all-target matrix.

## Retained evidence identities

Raw logs remain under the initiating Neural Stream repository's persistent
`.pi/drains/2026-09-05-mantle-repair/`. These BLAKE3 values bind exact raw bytes:

| File | BLAKE3 |
| --- | --- |
| `package-build-complete.log` | `29645dd4783a0a9c33e390f8c109e6bd90a22cf8a50c201151975137a6c56a21` |
| `focused-checks.log` | `57349cdbc8c2a8d47a88f0d196c543aea030e3c9055ddd15930b40eb21fa3116` |
| `evaluator-fixture-checks.log` | `b0046e6e7c811e14d137b68c6bac15a8059363be0dd325c9849f11a5635b9a1f` |
| `example-final-checks.log` | `3467b00ec7fa0abe2a60f744f15f11236e59c9f20c001cc45b42d50c4d2c6c97` |
| `tigerstyle-complete.log` | `be2e04fb710c77e703d29b4f803b9576038a53889b18231b247256214932e9e1` |
| `clippy.log` | `6c086be9ee8621c025406ca6baad78386530966b3a5169e93173f44e9161e048` |
| `all-root-tests.log` | `d05a9415ad7eea040c9cacd19274dc4032082f539dd158e2aee5ddc038ed1cc4` |

A log identity does not authenticate an actor or grant release authority.
This work does not prove package success, compiler correctness, reproducibility,
remote trust repair, production promotion, or downstream materialization.
