# Hardware simulation real-flow evidence

Date: 2026-07-14
Implementation commit: `59a5019a`
Generic dynamic-input repair commit: `9d3b24c1`

This transcript supersedes the earlier implementation checkpoint for I5–I8 and V2–V3. It records exact proof boundaries and leaves I9, I10, V4, V5, and V6 unchecked where their full-shared hardware dependency is not proven.

## Proven identities

- Profile: `mantle-hardware-profile://blake3/29ae88cd1abea071db01d724167876f4caef643344cce36e1d0b6e634ffb5c0f`
- Cohort: `mantle-hardware-cohort://blake3/672eb14a9e5494aba76794c35013b656b9d8525d15107ed8ac5f63b0ac78609c`
- Cohort closure: 53 sorted store roots, BLAKE3 `32e93e2530eb0b3afcf3fd7062c1543441e02b5ba027a161e76631dcf31f31ed`
- Generated sources: 16 files, 190,187 bytes, BLAKE3 `dc3f36f05ec76104bc500f00764b06345df810bd8c76bb99dc776da11fbac552`
- Hardware plan: BLAKE3 `7df6ae3962ca54dba61ecd6caeb494c63da8361092f9a58888db83c9bc4c0e62`
- Generic `mantle-plan-v1`: BLAKE3 `b1e30af6fd30c080a993e3d4505cd602bc962c484764ad4e3aa5591f82c202df`
- Plan shape: nine compile units, one link unit, two smoke units, three roots, thirteen hardware action nodes including generation
- Seed boundary: `nix-produced-open-source-tool-cohort`; no source-bootstrap claim

The exact cohort closure is checked in at `tests/fixtures/hardware-simulation/tool-closure-paths.txt`. The profile and source fixtures are under `tests/fixtures/hardware-simulation/`.

## I5 / I6 — generation, compile, and link

Strict bubblewrap generation ran Verilator 5.046 twice and removed only timestamp bookkeeping before comparison. Both normalized trees had the generated-source identity above.

The generic worker accepted all twelve dynamic units:

```text
accepted native dynamic plan output ... plan_digest=b1e30af6fd30c080a993e3d4505cd602bc962c484764ad4e3aa5591f82c202df accepted_units=12
```

The fresh run completed the producer, nine independently declared object builds, explicit `link.simulator`, and both smoke units:

```text
worker streaming finished completed=13 succeeded=4 failed=0 roots=4
```

The four report roots are the producer, simulator link root, and two smoke roots. The full action-result report contains thirteen successful publication entries, one for each producer/dynamic unit. The report includes per-root artifact-attestation paths and saved logs.

Fresh proof files:

- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/fresh.json`
- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/fresh.stderr`
- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/state/attestations/`
- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/state/logs/`

The report records practical-mode closure-resolution degradation for Nix-produced cohort paths lacking local PathInfo. It does not erase the strict bubblewrap boundary or promote those paths to source-bootstrap evidence.

## I7 / V3 — typed smoke outputs and wrong model

Passing smoke outputs are bounded `mantle-hardware-smoke-result-v1` JSON. They record matching expected/observed values, exit code zero, typed simulator/action/profile/cohort/source refs, one-byte stdout refs, zero-byte stderr refs, and required non-claims.

```text
zero-plus-zero: input=0,0 expected=0 observed=0 verdict=pass exit=0
one-plus-two:  input=1,2 expected=3 observed=3 verdict=pass exit=0
```

Passing result paths:

- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/store/p1gy579qla2sc17lw7xiqy7izi1d3lgz-hardware-smoke-zero-plus-zero`
- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/store/8ggscg0hk2hvmnl3383z7dhqrcvxsgg1-hardware-smoke-one-plus-two`

Wrong-reference revision `d2cc93ee610863e22c73d4a1579f43d36501f006` produced source BLAKE3 `dde9803907bb90e0f5406ae8e83ee2dffbb10e1e1f5f2079a9b49f84c9fc74ca`, generated BLAKE3 `76cb603e2b7c0bf267e263b7a6c392e3a6d975eb537bb558190edeb570955798`, and generic plan BLAKE3 `88254966975a7583db8421f60c1bc5a935264fea46cb5c09fb6f53db10d139c2`.

Its producer, nine compile units, and link unit succeeded. Both smoke units failed closed:

```text
reference mismatch expected=1 observed=0
reference mismatch expected=4 observed=3
worker streaming finished completed=13 succeeded=2 failed=2 roots=4
```

The wrong-model report contains eleven successful publication entries, exactly producer + nine compile + link. It contains no publication for either failed smoke action and no passing smoke output. The failed roots have saved bounded logs.

Negative proof files:

- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/wrong-model.json`
- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/wrong-model.stderr`

## I8 — generic action-result path

No hardware-specific scheduler, store, publication, admission, or reuse branch was added. Fresh passing producer, compile, link, and smoke actions published through the generic action-result implementation. The wrong-model run demonstrated generic failed-action suppression.

Generic positive and negative rails:

```text
crunch-action-result-core: 6 passed; 0 failed
crunch-build action_result: 8 passed; 0 failed
crunch-store action_result: 14 passed; 0 failed
```

These cover deterministic refs, complete admission facts, bad signatures, stale drift, incomplete objects, conflicts, interrupted publication, poisoned indexes, failed-build non-publication, CA reuse without executor calls, and HTTP clean-client object transfer. They validate the generic mechanism used by the hardware actions; they do not by themselves prove a clean-client hardware full-shared hit.

## Partial I9 / V5 — invalidation and reuse

Selected-reference variant:

- Git revision: `48c284fac87a9350de026902e134c16b6a66711c`
- Source BLAKE3: `6e118d493647fd9df22209d4ade2ac64875d7de67f3adc3f3158bf6f19fc6a48`
- Generated BLAKE3: `89b0e371765dd866a3e2ecaf73731c7ab7d656f0452ad8719528f258901ad908`
- Profile ref: `mantle-hardware-profile://blake3/f4382d6761f63b25ff72f420fedca27655521d5294f432a777cb49fa4c5cc7b5`
- Generic plan BLAKE3: `b3002c019b017ec80b86db300eaaaf4e6ef27dcb0cec059cf7ecf9400d64eb1e`
- Result: thirteen successful builds and passing smoke roots

The selected source participates directly in profile/generation/action identity, so all dependent generation, compile, link, and smoke identities changed.

Unrelated variant:

- Unrelated source BLAKE3: `f05b14cd045c859bfac619d57a341dcea1e6df7d9da3fbe6a9da9cc761fda577`
- Selected profile and plan identities: unchanged
- Result: thirteen `all outputs cached, skipping build` records and `completed=0`

Proof files:

- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/selected-change.json`
- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/selected-change.stderr`
- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/unrelated-change.json`
- `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh/unrelated-change.stderr`

This is count/identity evidence only. Elapsed timestamps are diagnostics and are not gates or speedup claims.

## Exact remaining blocker

I9 requires a hardware-specific clean-client full-shared hit with zero executor calls. The hardware graph is input-addressed, so a client with accepted local/remote PathInfo takes the ordinary cache path before action-result admission. Copying only the action-result index into a clean state failed closed because PathInfo, object completeness, receipt, sandbox policy, and reference-scan facts were absent. Weakening those checks or relabeling a local cache hit as a shared hit would be dishonest.

Therefore:

- I9 remains unchecked despite fresh, selected-change, and unrelated-change evidence.
- I10 remains unchecked because it depends on I9, although the catalog and documentation are implemented.
- V4 remains unchecked because the clean-client compile/smoke hardware reuse clause is not directly proven.
- V5 remains unchecked because it depends on I9, although selected/unrelated invalidation facts are proven.
- V6 remains unchecked because I10, V4, and V5 remain open.

## Focused validation

Commands and current results:

```text
cargo test -p crunch-hardware-simulation-core --lib
  9 passed; 0 failed

cargo test -p crunch-hardware-simulation --lib
  8 passed; 0 failed

cargo test -p mantle --test examples_inventory
  10 passed; 0 failed

cargo test -p crunch-build --lib native_dynamic
  9 passed; 0 failed

cargo test -p crunch-action-result-core --lib
  6 passed; 0 failed

cargo test -p crunch-build --lib action_result
  8 passed; 0 failed

cargo test -p crunch-store --lib action_result
  14 passed; 0 failed

cargo fmt --check -p crunch-hardware-simulation-core -p crunch-hardware-simulation -p mantle
cargo check -p mantle --example hardware_simulation_plan
cargo clippy -p crunch-hardware-simulation-core -p crunch-hardware-simulation --all-targets --no-deps -- -D warnings
cargo clippy -p mantle --example hardware_simulation_plan --no-deps -- -D warnings
  exit 0
```

Every Nix-wrapped command used `--option secret-key-files ''`.

Current Cairn results after task-marker normalization:

```text
cairn validate --root .
  valid=true; changes=7; specs_validated=30; issues=[]

cairn tracey coverage --root .
  traceability coverage ok: 140/140 referenced (profile mantle-default)

cairn gate proposal prove-hardware-simulation-build-flow --root .
  PASS; receipt_hash=24a19593dbefd0dd4b3aba1d1a9bc3e9f834b2878a384d6ae91e74dc9cd6e636

cairn gate design prove-hardware-simulation-build-flow --root .
  PASS; receipt_hash=74a92b7737bb8422a2e6df6e03d69161f1f4473ced9709e5ad42496325b21e69

cairn gate tasks prove-hardware-simulation-build-flow --root .
  PASS; receipt_hash=85723b9f902221134834ebeb720691d372067c1f941541a1bb12ba7562ff6f75
```

Wasm remains a non-claim: the active toolchain lacks `wasm32-unknown-unknown`, so no wasm-target validation was recorded.

## Non-claims

This evidence does not claim commercial simulator or license-server support, FPGA/ASIC correctness, physical design, timing closure, production remote-farm throughput, DVCon benchmark reproduction, elapsed-time speedup, source-bootstrap provenance for the tool cohort, a hardware clean-client full-shared hit, release readiness, or wasm validation.
