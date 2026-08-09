# Verification: stabilize the Mantle operator command contract

Date: 2026-08-08

## Baseline

Before implementation, the focused error tests passed:

```text
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 2282 filtered out; finished in 0.00s
```

The operator integration baseline had two real plan-mode state-mutation failures:

```text
assertion `left == right` failed: plan must not mutate empty state dir
  left: 4
 right: 0
assertion `left == right` failed: plan preflight must not mutate state dir
  left: 4
 right: 0
test result: FAILED. 13 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
```

The branding baseline also reported the unclassified stale prose in
`docs/mantle-naming.md:11`.

## Accepted catalog

The checked descriptor snapshot contains 169 executable public command paths.
Synthetic recursive Clap `help` subcommands are excluded. Public `--help` and
`-h` flags remain in each applicable descriptor.

The typed Nickel inventory contains 194 surfaces. It includes commands,
project files, environment inputs, machine schemas, and exact compatibility
identifiers. Each surface records an owner, role, supported operations,
support tier, mutation class, network class, compatibility state, migration
reference, and removal gate where applicable.

The deterministic catalog records this entry identity:

```text
schema: mantle-operator-command-catalog-v1
identity_blake3: 743dd080162620dc069e3d8756606359ae7c9835b739c106006469d6b10a68f6
entries: 169
```

Checked artifact file identities:

```text
blake3-niuDd5jlQIYq0H6FAnmatotmvEdo8aSW+E1KYushJwY=  config/operator-command-descriptors.json
blake3-f6ph2V1Zo8Z7MAJ8I2WwwRpvLbSfF0hO/Jgubf7W+S0=  config/operator-surfaces.ncl
blake3-SyVnL5MS0vJmWBW+rJiA56pwnteh4d4GPZtqEscEums=  config/operator-surfaces.json
blake3-w1kirgqs1/DNqDTbEkaOJm8FlXYSvSs8hjc1Wm94jiU=  config/operator-command-catalog.json
blake3-oc1DcfVB8SymcMIY2ro15lJ0VqMRLhaeco11yF21eF0=  docs/generated/operator-command-reference.md
blake3-lUgQecr2rfCglIJruiivCrWC8W3FXDk2aNrbpS5Ce38=  docs/generated/canonical-operator-workflow.md
```

## Focused tests

Command:

```text
nix develop -c cargo test -p mantle --lib operator_contract:: -- --nocapture
```

Result:

```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 154 filtered out; finished in 0.00s
```

Command:

```text
nix develop -c cargo test -p mantle --bin mantle errors:: -- --nocapture
```

Result:

```text
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 2303 filtered out; finished in 0.00s
```

Command:

```text
nix develop -c cargo test -p mantle --test operator_diagnostics -- --nocapture
```

Result:

```text
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
```

This result includes the repaired empty-state and missing-store plan cases.
Plan mode no longer creates the four store-state entries found in the baseline.

Focused parser command:

```text
nix develop -c cargo test -p mantle --bin mantle build_cli_ -- --nocapture
```

Result:

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 2309 filtered out; finished in 0.01s
```

Focused build-plan unit command:

```text
nix develop -c cargo test -p mantle --bin mantle build_plan:: -- --nocapture
```

Result:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2322 filtered out; finished in 0.00s
```

## Contract and quality rails

Command:

```text
nix develop -c ./scripts/check-operator-command-contract.sh
```

Result:

```text
operator command contract generator self-test: PASS
operator command contract: PASS (commands=169)
```

The rail compares the live Clap descriptor export with the checked snapshot,
runs the generator positive and negative self-test, typechecks and exports the
Nickel inventory, and checks the catalog and generated documentation.

Branding commands:

```text
nix develop -c sh -c 'rustc scripts/check-stale-branding.rs -o /tmp/mantle-check-stale-branding && /tmp/mantle-check-stale-branding'
nix develop -c sh -c 'rustc scripts/check-stale-branding.rs -o /tmp/mantle-check-stale-branding && /tmp/mantle-check-stale-branding --self-test'
```

Result:

```text
stale branding check passed
stale branding checker self-test passed
```

Focused first-party Clippy command:

```text
nix develop -c cargo clippy -p mantle --lib --bins --test operator_diagnostics --no-deps -- -D warnings
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 43.49s
```

Changed Rust files passed direct Rustfmt checks, including `src/main.rs` with
child-module formatting disabled. `git diff --check` passed.

The required broad command was also run:

```text
nix develop -c cargo fmt --check -p mantle
```

It retained the pre-existing formatting difference only in
`src/source_built_fixed_point_shell.rs`. Exact first-parent review passed:

```text
git diff --exit-code origin/main -- src/source_built_fixed_point_shell.rs
```

The candidate does not modify that excluded fixed-point surface. No broad
formatting-success claim is made.

## Compatibility and non-claims

This change preserves existing wire and read compatibility. It does not rename
legacy project files, environment inputs, schemas, store prefixes, Nickel
functions, workflow identities, or non-claim tokens. It documents the evidence
required before a later read-only or historical-only transition.

This change does not prove a source-built fixed point, full-source closure,
bootstrap completion, vendored Clippy success, or release readiness. It does
not remove any compatibility surface.

## Cairn validation

The legacy-layout Cairn revision was required because current Cairn discovers
`.cairn/`, while this repository uses `cairn/`.

Pre-archive validation result:

```json
{
  "change_issues": [],
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_findings": [],
  "spec_issues": [],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}
```

Pre-archive gates:

```text
proposal: valid=true, verdict=PASS, issues=[]
design: valid=true, verdict=PASS, issues=[]
tasks: valid=true, verdict=PASS, issues=[]
```

The complete command transcript is retained in the session evidence as
`cairn-prearchive.log`.

After all task boxes were checked, validation again returned `valid: true` with
empty findings and issues. The tasks gate returned:

```json
{
  "input_hash": "01a75c982c0d5a57b5a8c3928653c9ac7db0e34b9d62dfc1b4ec83cd82ddff95",
  "issues": [],
  "valid": true,
  "verdict": "PASS"
}
```
