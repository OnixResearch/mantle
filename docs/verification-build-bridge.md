# Octet/Trellis identity handoff bridge (issue 14)

## Status and authority

The bridge consumes externally owned proof profiles, exact receipts, producer
reports and executable job metadata. ADR 0034 remains a hard gate: no
Mantle-authored theorem policy, proof registry or accepted proof artifact is
introduced. Completion requires the integration owner to exercise the real
producer job; the recorded fixture alone is not issue-14 acceptance.

`crunch-verification-bridge-core` is the no_std identity consumer;
`crunch-verification-bridge` owns bounded filesystem reads and publication.
`mantle-verification-bridge` exposes the latter without invoking a compiler,
verifier, subprocess, network client, Cargo, Nix, or ambient sibling checkout.
It reads only an explicitly selected root. Inputs should be immutable store
outputs or private fixture directories; concurrent hostile filesystem mutation
is outside this adapter's support boundary. Symbolic links in the root or
selected artifact paths are rejected.

The report deliberately separates:

- Ordinary Mantle derivation success: packaging executed, or a signed cached
  output was reused. This is reported by the existing build/store machinery.
- External evidence availability: selected source/profile/provider/generated
  artifact bytes match the named external links. The exact original receipt
  and profile are copied without reserialization.
- External proof acceptance: **not evaluated by Mantle**. A producer's recorded
  `supported`/`passed` fields are observations, not fresh verification, theorem
  admission, or downstream release eligibility. A `failed`/`blocked` outcome
  cannot produce a successful bridge observation.

Octet retains execution profiles, theorem admission and receipt validation.
Trellis retains its theorem campaign and Verus evidence. Valence retains
external evidence acceptance; Cairn retains lifecycle readiness. Existing
release opaque-sidecar and Trellis role-pair checks are unchanged.

## Inventory before schema design

The inspected immutable upstream revisions are recorded in
`tests/fixtures/verification-bridge/upstream.json`:

| Owner | Immutable source | Existing contracts consumed |
|---|---|---|
| Octet | `4a540dff99aa2040e267e42bbafe09144f276572` | `docs/aeneas-consumer-contract.md`; generated `octet-aeneas-consumer-profile/v1`; `octet-formal-proof-envelope/v1` with `octet-aeneas-consumer-receipt-binding/v1` |
| Trellis | `22d51fb8c1f57155f6c0c4acc4e14902ebb6b4f3` | `docs/proof-artifact-manifest.md`; existing `trellis.proof-artifact-manifest.v1` reference boundary; representative pilot `trellis-aeneas-cross-verifier-profile/v1` and `trellis-aeneas-proof-receipt/v1` |

Octet profile identity:
`b3:62bb3c01a2037eef1410e35d3902d73fb5e7439e8856d721d0823e14c0af4879`.
Trellis pilot profile identity:
`b3:eaf8005e31e201b073ba9cdde36e563ad31f96d597b7337c6a03825211519c75`.
Both reference Octet provider profile
`b3:cd11b49d1acede2c6a91fa26f0b7a6b4b28db4007b4601a65b3269706a0ce625`.
The Trellis pilot separately pins Octet revision
`890f424cd7b1b2015f39c0eb5747f1bd1e576028`; inspecting newer Octet source does
not advance that recorded provider pin.

No new external DTO is substituted for those contracts. Mantle's local
`PackagingInputs`/`Observation` records describe packaging expectations and
byte-link checks only; they are not external execution manifests.

The core takes owned `ObservationRequest { inputs, profile, receipt, materials }`
values, with each `Material` owning its path and bytes. `observe_octet` and
`observe_trellis` return `ObservedHandoff`, retaining those exact buffers with
the observation rather than cloning payloads. `identity(Vec<u8>)` similarly
returns `IdentifiedBytes { bytes, identity }`. The adapter's
`packaging_inputs(Producer, Vec<u8>, &str)` returns `ProfileInputs { inputs,
profile }`, and `consume` takes owned `PackagingInputs`; `consume_selected`
threads the initially read profile through the check without reopening it.
These local owned plans do not change any external wire schema or authority.
`relative_path(String)` returns the admitted owned path or a rejection, allowing
the adapter to retain path allocations. `export(Handoff, &Path)` returns the
same handoff after publication so callers can inspect the observation without
cloning material names or payloads.

`octet` retains the deliberately weaker selected-stage identity check used by
the recorded fixture. `octet-verified` additionally requires Octet's exact
`producer-artifact-verification.json`, its `valid` status and empty diagnostics,
the selected envelope's matching report row, and every payload named by both
report and envelope. Mantle hashes/links these bytes; it does not reimplement
the owner's theorem admission, checker semantics or report authenticity policy.
This observes external acceptance, rather than turning build success into proof.

The provider-profile locator is derived from the selected receipt: it is the
single `artifact_identities` row whose `blake3` equals
`consumer.provider_profile_identity` (compared with the `b3:` prefix ignored)
and whose locator's final component is `provider-profile.json`. Zero matching
rows, more than one, or an unsafe relative locator fail closed, so a
rail-named proof directory binds without a fixture-specific constant. The
shell's material plan and the core check share this one function.

The producer tool graph admits exactly the two wire shapes the producer job
declares: the legacy `nix derivation show` map of full store paths with
`inputDrvs`/`inputSrcs`, and the `version: 4` `derivations` map keyed by store
basenames, each node carrying `version: 4` and `inputs.drvs` values with string
`outputs` and an empty `dynamicOutputs` object. The root is located by the final
component of `root_derivation`, every dependency must sit inside the closure,
and mixed, unsupported, or non-empty dynamic-output shapes fail closed with
stable error variants. `tool_graph_identity` stays the digest of the exact
graph bytes.

Both rules locate and parse producer-authored bytes. Neither grants proof
authority: Mantle still does not admit theorems, evaluate proof truth, or
reimplement the producer's verification policy.

The Trellis adapter checks exact pilot profile/source/artifact identities,
byte sizes, and the opaque Verus receipt payload identity. It does not parse
or accept Verus proof semantics. Trellis's existing primitive manifest is not
converted into a new Mantle proof registry.

The association digest includes the selected immutable source revision,
source/profile/provider identities, exact receipt digest, measured producer
**tool-contract** and options identities, and checked artifact locators.
A tool-contract identity is not a constructed tool closure. Old receipts
cannot satisfy changed expected source/profile/tool/options identities.
The source revision is Mantle's source-acquisition observation, not an
invented source-revision field inside the original receipt.

Bounds: 1 MiB per JSON document, 8 MiB per material, 128 materials and 32 MiB
aggregate material bytes. The separately bound recursive producer tool graph
allows 256 MiB and 32,768 derivations. Relative paths reject escapes and symlinks.

## Real fixture preparation and exercise

Preparation copies exact Git object bytes, not the mutable worktrees. It
retains upstream license notices and refuses to overwrite a destination:

```sh
python3 scripts/prepare-verification-bridge.py \
  --octet ../octet --trellis ../trellis --out /tmp/bridge-fixture
```

The checked fixture was prepared with that command's implementation against
the exact revisions above. Preparation itself does not run a proof or validate
its outcome.

After workspace registration and compilation by the integration owner:

```sh
cargo test -p crunch-verification-bridge --test boundaries

target/debug/mantle-verification-bridge octet \
  tests/fixtures/verification-bridge/octet profile.json receipt.json \
  4a540dff99aa2040e267e42bbafe09144f276572 /tmp/octet-handoff

target/debug/mantle-verification-bridge trellis \
  tests/fixtures/verification-bridge/trellis profile.json receipt.json \
  22d51fb8c1f57155f6c0c4acc4e14902ebb6b4f3 /tmp/trellis-handoff

python3 scripts/prove-verification-bridge.py \
  --mantle target/debug/mantle \
  --bridge target/debug/mantle-verification-bridge \
  --b3sum "$(command -v b3sum)" --out /tmp/bridge-packaging-proof
```

The driver builds ordinary typed Mantle derivations from BLAKE3-pinned
`file://` acquisition inputs, separately packages the already observed owner
bytes in strict sandboxes, checks cold construction and warm-cache reuse,
and reconsumes the exported store output with `PATH=/no-ambient-tools`.
It saves stdout/stderr and selected result identities. It does not run an
external proof or construct a verification tool closure. Its generated Nickel
is a test packaging recipe, **not** a producer-authored verification job.
Existing store signatures/cache admission retain their authority; the bridge
adds no trust grant or cache implementation.

Boundary regressions cover source/profile/tool/options mismatch, missing and
tampered payloads, path escape, unknown receipt version, failed/blocked
external outcomes, symlink substitution, and exact-byte export/reconsumption.
Passing those regressions would demonstrate the adapter's boundary, not the
complete issue-14 acceptance. No current test/build/proof results are claimed
in this document; integration must exercise these commands.

## Executable owner job and remaining exercised acceptance

The owner-authorized Octet additions now supply the previously missing job:

- `standards/proof-rails/aeneas-lean/mantle-job.ncl` defines
  `octet-mantle-producer-job/v1`, selects existing consumer/provider identities,
  the exact reviewed rail/argv, resource bounds, denied-network execution and
  output contract. It explicitly declares that this Lean rail uses no SMT solver.
- `nix/mantle-aeneas-job.nix` is an ordinary producer derivation that executes
  actual `cargo-octet prove` and `cargo-octet artifact verify`; it preserves the
  full artifact tree, original report/envelope bytes and source. It is not an
  expected-receipt copier.
- `tools/mantle-producer/export.py` performs acquisition-only export from an
  explicit immutable committed flake reference. It freezes Nix's complete
  recursive derivation graph, original flake metadata/lock, exact root/revision
  and every declared source object. No proof executes during acquisition.

After committing those upstream changes, prepare the real source-bound handoff:

```sh
python3 ../octet/tools/mantle-producer/export.py \
  --nix /absolute/path/to/nix --nickel /absolute/path/to/nickel \
  --root ../octet --flake 'git+file:///absolute/path/to/octet?rev=COMMITTED_REVISION' \
  --out /tmp/octet-producer-export
```

The immutable revision must actually contain the owner additions; an older
recorded fixture revision cannot substitute. Acquisition's complete ordinary
Nix graph is an input to the existing foreign graph/source/pipeline boundaries,
not a second Mantle proof-job execution engine. Tool bootstrap/binary inputs are
declared and do not establish seed-free or source-only tool construction.

Once the real producer output exists, consume and export it:

```sh
target/debug/mantle-verification-bridge octet-job \
  /path/to/actual/producer-output /tmp/octet-producer-export \
  COMMITTED_REVISION /tmp/octet-real-handoff
```

`consume_octet_job` requires full verified-mode linkage, matches the owner job's
profile/provider/rail, checks its network-denied declaration, matches the
immutable flake revision and requires a complete recursive tool graph. The
association binds the exact job, graph, flake metadata/lock and root derivation.
`export` preserves these original metadata bytes. Replay uses the exported
directory as both explicit roots; no ambient Octet/Nix/tool lookup occurs.
Neither metadata parsing nor report linkage grants cache/signature authority.

The integration owner must still exercise actual Mantle construction and
signed-cache/export replay, changed source/tool/profile identities, stale,
missing/schema/tamper/external-rejection negatives and no-network execution.
The existing foreign cache-preserve route is cache-only: importing a prebuilt
Nix closure is not evidence that Mantle source-built the tool graph. Likewise
the fixture packaging proof is not real producer execution. Record the actual
route, provenance, supported graph fragment and limits rather than silently
substituting either weaker result. No current build/proof completion is claimed.

## Trellis Verus producer job (`trellis-job`)

`trellis-job` binds a Trellis-owned Verus producer job without reusing the
Aeneas pilot schemas:

```sh
target/debug/mantle-verification-bridge trellis-job \
  /path/to/trellis-producer-output /tmp/trellis-producer-export \
  COMMITTED_REVISION /tmp/trellis-real-handoff
```

`OUTPUT '-'` inspects without exporting, as in the other modes.
`consume_trellis_job` reads `profile.json`, the receipt at the path the
profile's `verus.receipt_path` names, `producer-report.json`, and every
material the profile names (source paths, artifact `locator`s, the receipt,
`proof-cards.json`, `evidence.json`) under the existing document, material,
aggregate-byte, count and no-symlink bounds. Packaging identities reuse the
Trellis derivation from `consumer.provider`, `consumer.source.paths[0]`,
`provider_tools` and `extraction`; the profile's own bytes remain the profile
identity.

`observe_trellis_job` requires schema versions
`trellis-verus-producer-profile/v1` (profile), `octet-verus-proof-receipt/v1`
(Verus proof receipt) and `trellis-verus-producer-report/v1` (Terrane's own
report). It checks the profile digest and provider identity, receipt
`status == "passed"` and non-empty `checked_scope`, every profile source path
and artifact locator as an exact material with its recorded byte count, every
receipt `source_bindings` row against a profile source path by digest and path
suffix (the receipt binds repository-relative paths, the profile the
output-relative copies), the selected source path against the packaging source
identity, `provider_tools`/`extraction` digests, each profile artifact against
a receipt artifact row (`stdout_artifact`, `stderr_artifact`, comparing every
field the receipt records and ignoring the profile's `role`), and the profile's
`verus.receipt_identity`/`verus.receipt_path` against the selected receipt
bytes. The producer report's exact bytes become
`external_report_identity`; it is not interpreted. `proof_acceptance` stays
`not-evaluated-by-mantle`.

`bind_trellis_job` shares one job binder with `octet-job`. The Trellis job must
declare `trellis-mantle-producer-job/v1`, owner `Trellis`, matching
profile/provider identities, `execution.rail_id` equal to the receipt's
`checked_scope`, `execution.network_allowed == false`, a flake revision equal to
the packaging source revision, and a complete valid v4 or legacy tool graph
rooted at `root_derivation`. `check_tool_graph` is unchanged. The association
binds the job, graph, root derivation, flake metadata/lock and the producer
report. A failed/blocked receipt, a wrong job schema, owner, rail, network
declaration or revision, tampered bytes, a missing report/card/evidence
document, and an absent root or incomplete closure all fail closed.

`export` is unchanged, so the exported handoff carries the exact producer
`profile.json`, receipt, report, cards, evidence, source and artifact bytes
with the job metadata; replay points both explicit roots at that directory and
performs no ambient Terrane/Nix/Verus lookup. ADR 0168 records the decision.

Synthetic boundary tests live in
`crates/crunch-verification-bridge/tests/trellis_job.rs`. They build the output
contract in temporary directories and cover the positive observation, export
replay, and every negative above. They are not the real Terrane producer run;
the realized output is exercised separately in the WP8 stage-2 handoff.
