# Source-closure tree verification — 2026-10-27

## Executed focused provider checks

The checked-in Mantle Nix development shell executed these changed-package
checks, with `CARGO_TARGET_DIR=/dev/shm/aspen-runtime-target-20261026`, two Cargo
jobs, debug information disabled and incremental compilation disabled:

- `cargo test --locked -p mantle-wasm-consumer-verifier --test consumer_fixtures`:
  **26 passed, zero failed or ignored**.
- `cargo check --locked -p mantle-wasm-consumer-verifier --no-default-features`:
  passed.
- `cargo clippy --locked -p mantle-wasm-consumer-verifier --all-targets --no-deps -- -D warnings`:
  passed.
- `cargo fmt --check -p mantle-wasm-consumer-verifier`: passed, no output.
- `cargo test --locked -p mantle --test wasm_component_cli production_cli_executes_pinned_pipeline_and_publishes_rehashable_component_evidence -- --exact`:
  **one passed, zero failed or ignored**, two unselected; actual pinned tools,
  publication, producer remeasurement, complete consumer verification and source
  copy were exercised (57.44 seconds in the successful test execution).

The first four-leg command reached the subsequent CLI build, which initially
failed because the root `cap-fs-ext` dependency had mistakenly been removed.
It was restored, and `cargo update --offline -p mantle` regenerated the lock
without updating package versions. The separate successful production CLI run
above used the restored dependency. The retained
[failed-attempt record](focused-attempt-root-dependency.json) includes the
26-test result and successful no-std/Clippy legs; inherited environment values
are deliberately excluded.

The disposable real-producer observer initially failed to compile because its
`DigestError` conversion used `?` into `Box<dyn Error>`. Explicit conversion
fixed the observation harness; its build subsequently passed. Building that
observer alone is not a claim that its fresh producer-to-Molten handoff ran.

## Immutable bounded-tree prerequisite

Cargo and Nix pin Bounded Tree implementation
`250f3b25d0933c2fa106aa82d1cc17a496716f06`; the reviewed recursive Nix hash is
`sha256-ZS8gQ8abQk5Pstm0VuQiGMmOrZUsK3TIJ3SItgwGoEY=`. The exact source and
owner evidence boundary are documented in
[Bounded Tree adoption](../../../../docs/bounded-tree-adoption.md).

Observed remote publication, not inferred from a local branch:

```text
git ls-remote https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git HEAD refs/heads/main
f73685b4b85ae7446ee3e52eb199ae871cd32ffd HEAD
f73685b4b85ae7446ee3e52eb199ae871cd32ffd refs/heads/main
```

The implementation pin is the tested implementation; the later owner
archive-only commit contains
`.cairn/archive/1970-01-01-avoid-blocking-member-substitution/`.
No sibling override or vendor edit is a published dependency.

## Broader-gate attempts and limits

`./scripts/check-first-party-quality.sh` completed its configure-bridge,
first-party formatting and strict first-party Clippy legs, then failed in the
workspace test leg. Seven `crunch-rustc-wrapper::waits` cases failed at
`WaitCollector::start` with `path must be shorter than SUN_LEN`; that crate
reported 41 passed and seven failed. The task-owned long `TMPDIR` plus the
fixture's nonce/socket path exceeded the host Unix socket path limit.

The failed observation is retained here, not relabeled as a passing workspace
run. The test leg is being rerun with the short owned `TMPDIR=/tmp/mst27`;
source and checks are not weakened. Full Nix checks and fresh public Molten
intake are separate outstanding observations until their result records exist.

Two read-only reviewer-agent attempts failed before reviewing source because
the provider returned a rate-limit error. No independent review result is
claimed. Inline source review covered native-name/BFS framing, opened
capabilities, no-follow/nonblocking opens, bounds, duplicate role/path agreement,
source/copy revalidation, exclusively leased staging cleanup and complete
Structural+Bytes report agreement.

This record grants no runtime admission, actor/extension installation,
source/compiler/component correctness, atomic source snapshots, durability,
release eligibility or shared-runtime cutover authority.

## Nix source-binding correction

The first full `nix flake check --no-write-lock-file --option builders '' --max-jobs 2 --cores 2 -L`
attempt failed in `checks.x86_64-linux.durable-file-publication-adoption`.
The retained [full result](provider-nix-gate.json) records that failure.
The check binds current Cargo/Nix input bytes; all four old receipt digests
differed from directly measured files after this dependency update. Its empty
builder log is not treated as evidence of any other cause.

The existing documented refresh workflow updates only those four typed receipt
and independent-validator bindings, regenerates JSON with pinned Nickel and
remeasures the sidecar. The exact measurements and generated identity are in
[the receipt maintenance history](../../../../docs/durable-file-publication-adoption.md#adoption-receipt-freshness).
This repairs current source linkage, not the historical adoption outcomes.
The replacement Nix gate remains unclaimed until its execution completes.

## Corrected focused verification

After the source-binding refresh, full Nix validation reported eleven Tiger
Style findings in the new shell/tree implementation. The correction retains
the same admitted bounds and framing: explicit saturating bound arithmetic,
an allocation-free parent-component iteration, the `is_tree_allowed` predicate,
and assertions over admitted-plan and exact streaming invariants. No lint
allowance or policy exception was added.

The [corrected focused execution](provider-focused-corrected.json) then passed
all five legs: **26 verifier fixtures**, no-default-features check, package
formatting, strict package Clippy, and **one actual production CLI/pipeline
case** (79.52 seconds; zero failed or ignored). This supersedes neither the
retained failed broader attempts nor their evidence limits. The final full
Nix and short-TMPDIR workspace results remain separate observations.

Two further read-only review requests also failed before source review with
provider rate-limit responses. There is no independent review result. Inline
review additionally checked the cross-version opened-directory bridge,
source-only directory-role restriction, complete-report equality and private
stage cleanup; successful scenarios and denials require executed evidence.

## Executed retained replay and fresh intake

The [native observer execution](development-producer.json) replayed the
unchanged retained counterexample through the current public APIs. Both
Structural and Bytes passed; **37 member observations matched**, including
the directory source closure. Its original bundle identity remains
`93a2e552582646f7c06a299296a33318fb0fbd1913471702a8ecd2efe939d19f`.
Source-only copying returned `Matched`, digest
`e3c55a6434da88070b9c2338efc5b2553da1c2ceae3c469b735f08cd276f20aa`,
and **4,145 content bytes**. No locator, digest, bundle or report was rewritten.

The same observer then ran the real pinned producer pipeline afresh, using its
original `demo:app` fixture and a declared **unadmitted intake-only profile**,
not Molten's supported production cohort. Producer execution and both producer
remeasurement APIs passed; Structural+Bytes again matched all 37 observations,
and the source-only copy matched the same source identity. Fresh bundle identity:
`53a0bbbca34663474b063aec4f6851eab37905eb7654a5421eb415d30a068704`.
The probe profile reference is
`blake3:2f8e9be2c51bf74d0b8b09811720a6d332e0cae546627832fd5481d7cead8a32`.
The expected profile was a real producer request input, never a post-publication
bundle edit. This observation cannot establish production-world compatibility.

The [actual Molten public-API invocation](development-intake.json) then consumed
that unchanged fresh bundle through `VerifiedBundle::load`: **265 exact component
bytes** and **115 exact WIT bytes** were retained, and the complete actual
Structural+Bytes report agreed with every private-snapshot observation. The
private lease was removed before success. Adding an empty source directory then
denied intake and removed its failed lease while preserving the earlier captured
component. `execution_admitted=false`; no runtime was installed or switched.
This execution used a development provider override; final immutable publication,
consumer pinning and the corresponding pinned checks are later delivery facts.

The [publication ZIP](handoff/producer-snapshot.zip) is a replay transport of all
**42 actual files** under original logical paths, not a source-closure archive
substitution. CRC and exact comparison against every published file passed.
Extract it into a fresh private root and retain all bundle locators. The native
[producer report](handoff/execution-report.json),
[consumer report](handoff/consumer-report.json), inert observer source/manifest/
lock captures and intake driver are retained alongside it. Original included
fixture implementation is attributable to this Mantle source cohort; the
observation harness is not an installed product mechanism.

```text
1a883e4dfe8a9aed568662e5a616ddd2627719b009980344bb24d4db6da4495b  producer-snapshot.zip
7a6e2ea1dc224f5896830059e6359804914ac6dbf3af2d1d45baa4a1e5f2c8e8  execution-report.json
e5cc69a315ea66955a3a268c21ee2a5210d28a7749ff911aabcfbe96ae17f0bd  consumer-report.json
8d3ba0eebdb4c9bf0c949cfb313857919499e16a5b6e9805e99e0dc1a3a4099f  development-intake.json
```

## Separate supported-world artifact-policy denial

The observer also supplied the unchanged supported Molten WIT, a real generated
list/result echo guest, and the supported product profile reference
`blake3:605a53191900ec2c8016cacc1e17c2338e9cda5423115dd5bb993e5082975910`.
The real pipeline compiled, componentized, composed and normalized it, but its
unchanged pinned Octet artifact rail denied publication. The retained
[execution report](handoff/product-world-blocked-execution-report.json) and
[exact Octet receipt](handoff/product-world-octet-receipt.json) record two
`disallowed-import` findings for nested `component::preserves-payload::Type(Eq(0))`
references and one `disallowed-proposal` finding for bulk memory at byte 498.
Receipt BLAKE3 is
`15993ccaaaaa190e5bfa87bbf49e696279908b751c0ddd0832156573de26c8c8`.
The guest source is retained in `handoff/product-world-source.zip`. No artifact
profile, admitted WIT, runtime feature set, receipt or policy result was weakened
or reinterpreted. This new production-world prerequisite is distinct from the
repaired source-tree contract; actor/extension adoption stays unproved.

Earlier disposable observer attempts rejected whitespace in the runtime invoke
expression and then a missing checked `wkg.lock`. Their original command/output
records are retained as `witness-attempt-invocation.json` and
`witness-attempt-lock.json`; the observer now preserves the real fixture lock
before calculating its declared source identity. Host kache separately disabled
caching after disk-I/O errors; compilation still completed without caching.

Immediate outcome: the unchanged directory handoff now verifies and copies, and
actual Molten private intake exercises that mechanism. Durable capability:
one bounded provider-owned tree identity/copy boundary reusable by consumer
snapshots. Maintenance owners remain Mantle's producer/verifier maintainers and
Molten's snapshot-intake maintainers. Repeatability evidence is the retained
positive/negative native execution, exact publication bytes and focused tests;
none of these artifacts grants runtime or release authority.

## Native implementation lifecycle validation

The [captured implementation lifecycle run](implementation-lifecycle.json)
executed the pinned Cairn binary through the Mantle development shell.
Repository validation returned `valid=true`, with empty issues; proposal,
design and tasks gates each returned `valid=true`, `verdict=PASS`, with empty
issues. These are structural lifecycle observations, not evidence of runtime
adoption, publication, consumer pinning or arbitrary implementation correctness.
The implementation commit precedes archive; final publication/pinning and
post-archive validation are separately recorded closeout operations.
