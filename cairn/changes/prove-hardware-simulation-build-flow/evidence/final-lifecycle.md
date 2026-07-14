# Hardware simulation final lifecycle evidence

Date: 2026-07-14

Relevant implementation commits:

- `9d3b24c1` — resolve declared dynamic inputs before hashing units
- `59a5019a` — prove hardware behavior through generic dynamic plans
- `ecbe0865` — admit shared results before ordinary cache reuse
- `7487e96c` — account for shared-result transfer evidence

This transcript supersedes the implementation checkpoint. All implementation and verification tasks in `tasks.md` are supported by the evidence below. The change remains active: this work did not sync, archive, or push it.

## Proven identities and boundary

- Profile: `mantle-hardware-profile://blake3/29ae88cd1abea071db01d724167876f4caef643344cce36e1d0b6e634ffb5c0f`
- Cohort: `mantle-hardware-cohort://blake3/672eb14a9e5494aba76794c35013b656b9d8525d15107ed8ac5f63b0ac78609c`
- Cohort closure: 53 sorted store roots, BLAKE3 `32e93e2530eb0b3afcf3fd7062c1543441e02b5ba027a161e76631dcf31f31ed`
- Generated sources: 16 normalized files, 190,187 bytes, BLAKE3 `dc3f36f05ec76104bc500f00764b06345df810bd8c76bb99dc776da11fbac552`
- Hardware plan: BLAKE3 `7df6ae3962ca54dba61ecd6caeb494c63da8361092f9a58888db83c9bc4c0e62`
- Generic `mantle-plan-v1`: BLAKE3 `b1e30af6fd30c080a993e3d4505cd602bc962c484764ad4e3aa5591f82c202df`
- Plan shape: one generation action, nine compile actions, one link action, two smoke actions, and three dynamic roots
- Evidence ref: `mantle-hardware-evidence://blake3/9c581d319c676243a347de04a9da02d3b4f09540fadfa74f0cd4304268adba94`
- Seed boundary: `nix-produced-open-source-tool-cohort`; no source-bootstrap claim

Typed hardware semantics remain in `crunch-hardware-simulation-core`; bounded Git/filesystem/tool operations remain in `crunch-hardware-simulation`; scheduler, store, admission, publication, and dynamic-plan execution remain generic. The shared-result ordering repair applies to every input-addressed action and contains no HDL, Verilator, compile, link, or smoke branch.

The exact cohort closure is checked in at `tests/fixtures/hardware-simulation/tool-closure-paths.txt`. The four-run typed bundle is `evidence/hardware-evidence.json`; the runtime action/result summary is `evidence/hardware-runtime-summary.json`.

## Fresh generation, compile, link, and smoke

Strict bubblewrap generation ran Verilator 5.046 twice and removed only timestamp bookkeeping before comparison. Both normalized trees had the generated-source identity above. The generic worker accepted all twelve dynamic units:

```text
accepted native dynamic plan output ... plan_digest=b1e30af6fd30c080a993e3d4505cd602bc962c484764ad4e3aa5591f82c202df accepted_units=12
```

The fresh run completed the producer, nine independent object builds, explicit `link.simulator`, and both smoke units:

```text
worker streaming finished completed=13 succeeded=4 failed=0 roots=4
```

The action-result report contains thirteen successful publications. The four report roots are the producer, simulator link root, and two smoke roots. Fresh proof artifacts are rooted at `/home/brittonr/.local/state/mantle-hardware-proof-20260714-fresh`; `fresh.json` has BLAKE3 `bd30045f55c0106b36423831fb4556d73e8b819a96b8b01dd5138d743c8ef016`.

The report records practical-mode closure-resolution degradation for Nix-produced cohort paths lacking local PathInfo. This does not erase each action's native strict-sandbox policy and does not promote the Nix-produced tool roots to source-bootstrap evidence.

## Typed smoke success and negative model

Passing outputs are bounded `mantle-hardware-smoke-result-v1` JSON:

```text
zero-plus-zero: input=0,0 expected=0 observed=0 verdict=pass exit=0
one-plus-two:  input=1,2 expected=3 observed=3 verdict=pass exit=0
```

They bind expected/observed values, process outcome, simulator/action/profile/cohort/source refs, one-byte stdout refs, zero-byte stderr refs, and required non-claims.

Wrong-reference revision `d2cc93ee610863e22c73d4a1579f43d36501f006` produced source BLAKE3 `dde9803907bb90e0f5406ae8e83ee2dffbb10e1e1f5f2079a9b49f84c9fc74ca`, generated BLAKE3 `76cb603e2b7c0bf267e263b7a6c392e3a6d975eb537bb558190edeb570955798`, and generic plan BLAKE3 `88254966975a7583db8421f60c1bc5a935264fea46cb5c09fb6f53db10d139c2`.

Its producer, nine compile units, and link unit succeeded. Both smoke actions failed closed:

```text
reference mismatch expected=1 observed=0
reference mismatch expected=4 observed=3
worker streaming finished completed=13 succeeded=2 failed=2 roots=4
```

The report has eleven successful publications—producer, nine compiles, and link—and no successful smoke publication or passing result. `wrong-model.json` has BLAKE3 `5b53220a51cd7896ebafad502585de416ccf8006abbd38375917398d1eb8f9b2`.

## Clean-client full shared hit

The canonical publisher state was exported as an HTTP binary/action-result cache containing canonical indexes and records, signed PathInfo, NAR/object content, action receipts, sandbox/network/producer/publication policy identities, reference-scan refs, and signatures. The hardware producer file was unchanged.

The final client command asserted that these paths did not exist before creating them:

- state: `/home/brittonr/.local/state/mantle-hardware-proof-20260714-full-shared/state-final`
- physical store: `/home/brittonr/.local/state/mantle-hardware-proof-20260714-full-shared/store-final`
- temp: `/home/brittonr/.local/state/mantle-hardware-proof-20260714-full-shared/tmp/final`

It then ran `mantle build --json --verbose --log-level info` with the HTTP substituter, its public key in both the substituter trust query and `--trusted-public-keys`, `--nix-compat`, the declared bubblewrap executable, and the unchanged producer.

The checked runtime summary validates all thirteen selected records and records:

```text
action reports:                              13
admitted HTTP shared results:                13
records with receipt/policy/reference facts: 13
outputs admitted:                            13
logical NAR bytes transferred:               791928
executor calls:                              0
worker completed actions:                    0
cached roots:                                4
failed roots:                                0
```

Every runtime report has `disposition = reused`, `selected_source_class = http`, one admitted candidate, no candidate diagnostics, signer `crunch-britton-desktop-1`, a selected result ref, and transfer evidence. The stderr transcript contains thirteen `shared action result admitted, skipping executor` records and no `Starting bwrap build` record.

Durable report identities:

- `full-shared-final.json`: BLAKE3 `c1f488ba51b3cd92cdbed53d08aee403b21632b50d5d46afeaee60654c5b8865`
- `full-shared-final.stderr`: BLAKE3 `f71b5af6e0ed914eee84e4a8a5755a1a8d0fb1105bbb9eb7147373d803e03099`

An immediate repeat again admitted all thirteen HTTP-indexed results, made zero executor calls, transferred zero logical NAR bytes, and reused 791,928 logical NAR bytes from local content. Its report and stderr BLAKE3 digests are `600aba5accef49b23375846e6bd980673bd2c2411b80de765612413c8c83380b` and `20d9b4f6ca66b17d99af203248b14c26b579def1bc01e9c5e538e06f77fa8fa5`.

Logical NAR bytes come from admitted signed PathInfo. They are deterministic work/transfer facts, not compressed wire-byte or elapsed-time claims.

## Fail-closed shared-result behavior

A hardware client run without the CLI trusted public key discovered the same thirteen HTTP records but reported thirteen `action-result-record-signature-untrusted` misses. It did not relabel them as shared hits: all thirteen actions executed, and the report recorded `built_total=4`, `cached_total=0`, and `failed_total=0` for the four roots.

The focused generic negative matrix additionally retains:

- stale action, source, and tool identity rejection;
- incomplete PathInfo/object rejection;
- bad record-signature rejection;
- conflicting output-set rejection without source-order selection;
- poisoned-index and interrupted-publication rejection;
- failed-build non-publication;
- bounded logical NAR-byte overflow rejection.

A copied result index without PathInfo, object completeness, receipt, policy, and reference-scan facts still fails closed. Shared action-result admission now runs before ordinary PathInfo cache fallback so input-addressed actions can produce honest admission evidence; ordinary signed cache lookup remains the fallback after a shared miss.

## Four-run invalidation and work-reduction evidence

`evidence/hardware-evidence.json` was built by the checked pure core and validates this stage matrix:

| Run | Generation | Compile | Link | Smoke | Transfer/reuse facts |
|---|---:|---:|---:|---:|---|
| fresh | 1 executed | 9 executed | 1 executed | 2 executed | 0 transferred, 0 reused |
| selected-source-change | 1 executed/invalidated | 9 executed/invalidated | 1 executed/invalidated | 2 executed/invalidated | 0 transferred, 0 reused |
| unrelated-source-change | 1 reused | 9 reused | 1 reused | 2 reused | 0 transferred, 791,928 logical NAR bytes reused |
| full-shared-hit | 1 reused | 9 reused | 1 reused | 2 reused | 791,928 logical NAR bytes transferred and reused |

Selected-reference variant:

- Git revision: `48c284fac87a9350de026902e134c16b6a66711c`
- Source BLAKE3: `6e118d493647fd9df22209d4ade2ac64875d7de67f3adc3f3158bf6f19fc6a48`
- Generated BLAKE3: `89b0e371765dd866a3e2ecaf73731c7ab7d656f0452ad8719528f258901ad908`
- Profile ref: `mantle-hardware-profile://blake3/f4382d6761f63b25ff72f420fedca27655521d5294f432a777cb49fa4c5cc7b5`
- Generic plan BLAKE3: `b3002c019b017ec80b86db300eaaaf4e6ef27dcb0cec059cf7ecf9400d64eb1e`
- Result: thirteen successful executions and passing smoke roots

The selected source participates in profile, generation, and downstream action identity, so the complete selected graph changed. `selected-change.json` has BLAKE3 `345f0cc225bf1aea2cf664f0bc004a1502b667ed0083d1659a7b9262ed395aff`.

The unrelated fixture changed to source BLAKE3 `f05b14cd045c859bfac619d57a341dcea1e6df7d9da3fbe6a9da9cc761fda577` while selected profile and plan identities remained fixed. The final local admission run reused all thirteen results with zero executor calls; `unrelated-shared-final.json` has BLAKE3 `a95d7c5e7ab6e1dc77208ea9bf6e705a0836eff1fdeb0bd307f0404e5a4141af`.

No elapsed measurement appears in the typed bundle. `elapsed_is_gating` is false for every run.

## Focused validation

Exact commands and test-result lines are committed in `evidence/focused-tests.txt`:

```text
hardware core:        9 passed; 0 failed
hardware shell:       8 passed; 0 failed
examples inventory:  10 passed; 0 failed
action-result core:   6 passed; 0 failed
build action-result:  8 passed; 0 failed
native dynamic plans: 9 passed; 0 failed
store action-result: 15 passed; 0 failed
machine contracts:    6 passed; 0 failed
```

`evidence/quality-checks.txt` records exit zero for:

- the hardware planning example check;
- focused first-party formatting;
- hardware core/shell Clippy across all targets;
- action-result core/build/store library Clippy;
- Mantle binary and hardware example Clippy.

All Nix-wrapped commands used `--option secret-key-files ''`. `CARGO_INCREMENTAL=0` was used for the final focused matrix after one shared incremental-cache operation failed with a missing transient `dep-graph.part.bin`; the isolated rerun passed and no source weakening was made.

## Cairn lifecycle checks

Exact current outputs are committed in `evidence/lifecycle-gates.txt`:

```text
cairn validate --root .
  valid=true; changes=7; specs_validated=30; issues=[]

cairn tracey coverage --root .
  traceability coverage ok: 140/140 referenced (profile mantle-default)

cairn gate proposal prove-hardware-simulation-build-flow --root .
  PASS; receipt_hash=360fba5b3f20c9bd5d904d97b54f487b66914006bca038735861338bc367c0d0

cairn gate design prove-hardware-simulation-build-flow --root .
  PASS; receipt_hash=91c7dcbb3a42a9b82f056393dca96a76a301667b10f5c0c0867af3e6d28195e0

cairn gate tasks prove-hardware-simulation-build-flow --root .
  PASS; receipt_hash=2924e0b10bf377df4cdb58543b5e36c49ea9c007dbc91876ceb773652ab10c65
```

The local built Cairn binary was used because the sibling Cairn flake remains affected by mutable path-input evaluation. The change was not synced or archived.

## Integrated main validation

After integration, the focused hardware core/shell and generic action-result,
build, store, machine-contract, example-build, formatting, and lifecycle rails
passed again. The all-examples inventory test alone reported the preserved
pre-existing untracked user file `examples/cowsay.ncl` as uncatalogued. The
clean tracked implementation worktree passed that same inventory test with ten
passing cases. The user file was not modified, deleted, or catalogued because
committing a catalog entry for an untracked file would make the tracked tree
depend on data it does not own.

## Known unrelated limitation and non-claims

The pinned toolchain still lacks `wasm32-unknown-unknown`; no wasm validation is claimed. Broad validation still includes the pre-existing `benchmark_runtime_boundary_stays_out_of_library_path` failure caused by a Cargo dev-dependency layout assertion; this change does not weaken or relabel that failure.

This evidence does not claim commercial simulator or license-server support, FPGA/ASIC correctness, physical design, timing closure, production remote-farm throughput, DVCon benchmark reproduction, elapsed-time speedup, source-bootstrap provenance for the tool cohort, release readiness, or wasm validation.
