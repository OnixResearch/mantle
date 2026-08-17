# Evidence: resolve-dependency-audit-waivers

## Scope

Task-IDs: I1, I2, I3, I4, V1, V2, V3, V4

Covers:

- r[verification_evidence.dependency_audit_waiver_resolution]
- r[verification_evidence.dependency_audit_upstream_blockers]
- r[verification_evidence.dependency_audit_policy_regression]

## Implementation evidence

### I1 — remaining waiver paths were re-audited

Baseline checked-in policy audit (pueue task 108) ran:

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml

advisories ok, bans ok, licenses ok, sources ok
```

Dependency path probes (pueue task 113) recorded:

```text
paste v1.0.15 -> nickel-lang-core v0.16.1 -> crunch-eval / nickel-lang
proc-macro-error2 v2.0.1 -> getset v0.1.6 -> oci-spec v0.7.1 -> snix-build
quick-xml v0.40.1 -> object_store v0.14.0 -> snix-castore
```

### I2 — tractable waiver retired by dependency feature movement

`postcard` was used from vendored snix crates with `std`, but default features also enabled `heapless-cas`, which selected `heapless 0.7.17` and `atomic-polyfill 1.0.3`. The implementation sets `default-features = false` and keeps `features = ["use-std"]` in:

- `vendor/snix-build/Cargo.toml`
- `vendor/snix-castore/Cargo.toml`
- `vendor/snix-store/Cargo.toml`

Lock refresh (pueue task 122) removed only the obsolete chain:

```text
Removing atomic-polyfill v1.0.3
Removing critical-section v1.2.0
Removing hash32 v0.2.1
Removing heapless v0.7.17
Removing spin v0.9.8
```

Diff check (pueue task 157):

```text
-name = "atomic-polyfill"
-version = "1.0.3"
-name = "critical-section"
-version = "1.2.0"
-name = "hash32"
-version = "0.2.1"
-name = "heapless"
-version = "0.7.17"
-name = "spin"
-version = "0.9.8"
```

`RUSTSEC-2023-0089` was removed from `deny.toml` and from the remaining-waiver inventory.

### I3 — remaining waivers have sharper unblock conditions

`deny.toml` and `docs/dependency-audit.md` now retain only four explicit waivers:

- `RUSTSEC-2024-0436`: `crunch-eval` -> `nickel-lang-core 0.16.1` -> `paste`; remove when Nickel releases a paste-free core or Mantle carries that upstream migration.
- `RUSTSEC-2026-0173`: `vendor/snix-build` -> `oci-spec 0.7.1` -> `getset 0.1.6` -> `proc-macro-error2`; remove when `oci-spec`/`getset` no longer pulls `proc-macro-error2`.
- `RUSTSEC-2026-0194`: `vendor/snix-castore` -> `object_store 0.14.0` -> `quick-xml 0.40.1`; remove when `object_store` permits `quick-xml >= 0.41`.
- `RUSTSEC-2026-0195`: same `object_store 0.14.0` / `quick-xml 0.40.1` cap as `RUSTSEC-2026-0194`.

Upgrade probes showed the local constraints still block the remaining fixed versions:

```text
quick_xml_probe_status=101
error: failed to select a version for the requirement `quick-xml = "^0.40.1"`
candidate versions found which didn't match: 0.41.0
required by package `object_store v0.14.0`

oci_spec_probe_status=101
error: failed to select a version for the requirement `oci-spec = "^0.7.0"`
candidate versions found which didn't match: 0.10.0
required by package `snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)`

nickel_probe_status=0
Locking 0 packages to latest compatible versions
```

Removing each retained waiver from a temporary policy still fails the advisory check (pueue task 148):

```text
RUSTSEC-2024-0436 status=1 -> paste v1.0.15 -> nickel-lang-core v0.16.1
RUSTSEC-2026-0173 status=1 -> proc-macro-error2 v2.0.1 -> getset v0.1.6 -> oci-spec v0.7.1
RUSTSEC-2026-0194 status=1 -> quick-xml v0.40.1 -> object_store v0.14.0
RUSTSEC-2026-0195 status=1 -> quick-xml v0.40.1 -> object_store v0.14.0
```

### I4 — audit evidence policy guard added

`scripts/check-dependency-audit-evidence.rs` validates that dependency-audit evidence includes the checked-in policy flag and success summary before it can be cited.

Self-test (pueue task 119):

```text
cargo -Zscript scripts/check-dependency-audit-evidence.rs --self-test

dependency audit evidence checker self-test passed
```

## Verification evidence

### V1 — checked-in policy audit

Updated audit command (pueue task 134):

```text
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml

advisories ok, bans ok, licenses ok, sources ok
```

The updated transcript `/tmp/mantle-dependency-waivers-cargo-deny-after-postcard-20260703.txt` contains no `RUSTSEC-2023-0089` or `atomic-polyfill` matches.

### V2 — non-authoritative audit transcript is rejected

Evidence guard validation (pueue task 179):

```text
cargo -Zscript scripts/check-dependency-audit-evidence.rs cairn/changes/resolve-dependency-audit-waivers/evidence/validation.md

dependency audit evidence policy check passed
```

Negative raw-transcript check (same pueue task):

```text
cargo -Zscript scripts/check-dependency-audit-evidence.rs /tmp/mantle-dependency-waivers-cargo-deny-after-postcard-20260703.txt

negative_raw_transcript_status=1
error: missing audit command: cargo-deny check
missing checked-in policy flag: --config deny.toml
```

### V3 — focused compile checks

Affected vendored snix crates compiled after the `postcard` feature change (pueue task 137):

```text
cargo check -p snix-build -p snix-castore -p snix-store --locked

Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.68s
```

### V4 — final validation

Format, guard, whitespace, and Cairn gates before task completion (pueue task 189):

```text
cargo fmt -p mantle --check
cargo -Zscript scripts/check-dependency-audit-evidence.rs --self-test
cargo -Zscript scripts/check-dependency-audit-evidence.rs cairn/changes/resolve-dependency-audit-waivers/evidence/validation.md
git diff --check
```

```json
{
  "validate": { "valid": true, "issues": 0, "specs_validated": 17, "changes": 1 },
  "proposal": { "valid": true, "verdict": "PASS", "receipt_hash": "5f66f6482743434e417cd0202d127fa639c86a838a7d6add64786caec6971a32" },
  "design": { "valid": true, "verdict": "PASS", "receipt_hash": "f6ceff42d9953f46fd3b6d74fede81cf120b4899e71b8bfd1bb6787c5b58e06e" },
  "tasks": { "valid": true, "verdict": "PASS", "receipt_hash": "5c2c31bade54fe1d138fffc2b5f32c673fbaf38b21d7cd043f34d42b6b474063" }
}
```

Final post-task-edit validation and gates (pueue task 13, compact transcript in `/tmp/mantle-dependency-waivers-final-gates-20260703`):

```json
{
  "validate": { "valid": true, "issues": 0, "specs_validated": 17, "changes": 1 },
  "proposal": { "valid": true, "verdict": "PASS", "receipt_hash": "69ba464ba28b4fcf7ab3469844adb7471fbc6146463a4056609219581bbd3c6e" },
  "design": { "valid": true, "verdict": "PASS", "receipt_hash": "d0c20f209e310225637274fe0cf0b756eb846454e032c55becab1bc97ba048c3" },
  "tasks": { "valid": true, "verdict": "PASS", "receipt_hash": "9bddc16f08b513312dd5f7cbafcfeedcdd40a5baf70a091e4e30ea1dfab76090" }
}
```

Executed `cairn sync --execute` did not materialize the full-spec-shaped delta, so the three accepted requirements were manually merged into `cairn/specs/verification-evidence/spec.md`.

Manual sync validation (pueue task 15):

```text
validate valid=true issues=0 specs_validated=17 changes=1
requirement counts:
dependency_audit_waiver_resolution 1
dependency_audit_upstream_blockers 1
dependency_audit_policy_regression 1
```

Pending archive and post-archive validation.

## Post-archive validation

Archive execution (pueue task 17):

```json
{
  "dry_run": false,
  "mutated": true,
  "reasons": [],
  "receipt_hash": "8abf172809c0e72f374ce9394a649f1648507122f8104f7c5661dd9e2f1f2d90"
}
```

Post-archive validation (pueue task 18):

```json
{
  "valid": true,
  "issues": 0,
  "specs_validated": 16,
  "changes": 0
}
```
