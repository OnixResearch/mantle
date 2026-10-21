# Immutable Cargo-planned build bundles

`cargo-build-bundle` composes Mantle's existing source bundle, `mantle-plan-v2`,
portable receipt bundle, and signed Mantle-native store archive. It is not a
resolver, new cache format, trust transport, or alternate scheduler. This is an
opt-in Cargo-planned Linux handoff, not universal Cargo support. Implementation
and a proof driver are not acceptance evidence: retain an actual successful
campaign report and raw commands from the final immutable source before claiming
issue #4 acceptance. Issue #15's baseline remains a separate release prerequisite.

## Supported fragment and compatibility

Version 1 supports `x86_64-linux`, host and target
`x86_64-unknown-linux-gnu`, native sandboxes, network denied, and the producer's
existing bounded Cargo-unit lane. The logical prefix is explicitly `/nix/store`
or `/mantle/store` and must match the client's independently supplied compatibility
file exactly. Physical store placement may differ. Arbitrary logical-prefix,
platform, ABI, profile, feature, policy or toolchain relocation is rejected.

The bound graph is the producer's already exported graph. Replay MUST NOT rerun
Cargo, metadata, the unit-graph oracle, or rust-plan. The producer's `out`, `plan`
and `sources` outputs must all be included as signed cached roots in the archive.
The ordinary worker re-admits that graph and its source slices. A missing cached
producer output is an error, not permission to reconstruct planning state.

The current generic limits are 4,194,304 plan bytes, 4,096 units, 256 source
slices and 256 direct inputs per unit. The outer manifest is at most 8 MiB, with
4,096 entries per collection, 16,384 bytes per string, nesting depth 16 and
1 TiB per transport. Existing source, receipt and archive limits still apply.
Small parsed planning/configuration payloads are capped at 64 MiB. A source
bundle is not copied into a second Cargo-specific source transport.

Unknown fields in the outer mandatory records, unknown mandatory schema/version,
planner receipt versions other than 1, Cargo unit-graph versions other than 1,
policy widening, incompatible limits, wrong prefix/target and noncanonical or
changed manifest bytes fail before client state exists. Receipt signatures,
revocation, validity windows, policy identity, matching output facts and evidence
completeness retain the existing receipt checker's semantics. Bundled public keys
are not automatically trusted. No private keys, credentials or Cargo home are
exported. A diagnostic incomplete receipt remains visibly incomplete; bundle
verification does not upgrade it to a correctness or release receipt.

## Bound inputs

The strict outer `mantle-cargo-build-bundle-v1` manifest binds:

- source-manifest, lock, receipt-bundle and canonical native-plan BLAKE3 identities;
- exact Cargo package identities to immutable source records, registry/Git
  package identities (including coordinates/version) to their verified records,
  and optional signed vendor-store root;
- wire unit IDs, roots, direct unit dependency edges, selected per-unit features,
  and package identities; only generic `target-facts` helper units have a null
  package identity;
- explicit rustc, Cargo, toolchain, producer, planner, helper and linker paths,
  required signed store roots and the producer's three output identities;
- host/target/system, profile, selected features and no-default-features switch,
  exact producer environment, native-declaration schema 1, role declarations and
  the existing core's resolved native inputs (not a second substitution rule);
- source/plan/receipt schema versions, prefix, admission limits, sandbox/network
  policy identities and separately supplied public trust-policy identity;
- full producer revision, raw oracle/evidence payload identities and the retained
  versioned planner receipt. Replay reports preserve this provenance rather than
  pretending that exported-graph replay is fresh planning.

Seven named artifacts are required: `source`, `receipts`, `plan`, `goals`,
`store`, `lock`, `config`. Each is a `{file, blake3, bytes}` record with a safe
relative transport filename. `source` and `receipts` are the existing formats;
`store` is the existing signed Mantle-native archive, not Nario v2. `goals` is
fully evaluated named derivation JSON with exactly one explicit producer goal
and the app observation consumer. No Nickel imports or exporter paths are
needed to decode it. Every converted builder/source and each native plan store
input must be covered by `required_store_paths` and present in the archive.

Five mandatory evidence keys are retained byte-for-byte from producer `out`:

| Key | Producer filename |
| --- | --- |
| `cargo-metadata` | `cargo-metadata.json` |
| `cargo-unit-graph` | `cargo-unit-graph.json` |
| `rust-plan-receipt` | `rust-plan-receipt.json` |
| `cargo-unit-plan-evidence` | `cargo-unit-plan-evidence.json` |
| `cargo-unit-bindings` | `cargo-unit-bindings.json` |

`provenance.oracle_inputs` binds the same key to its raw artifact BLAKE3. The
producer's binding record supplies exact wire-ID/package/feature mappings; the
bundle does not reconstruct the producer's host/target instance mapping.
Evidence files reconstructed as a directory must match the signed producer `out`
NAR. The transported plan file and configuration file must independently match
their signed store-root NARs. This prevents substituting a different plan beside
a valid cached producer. Configuration must be a regular store-root file, not a
path below a store directory, for this version's exact-file binding.

The store archive includes the full closure of all required roots. Every archive
path must have a corresponding portable receipt output record. Verification
checks the complete archive closure through the existing receipt checker, not
just the app output. Include source-ref evidence as usual: it is matched against
the already validated source manifest before imports. A verified vendor produced
by lock-driven acquisition enters through the existing verified vendor root and
source-state identities; a foreign acquisition report is not authority.
An optional acquisition-report artifact can retain the existing signed plan-output
and vendor-output attestations, canonical plan identity, parent derivation and
lock/selected-table facts. Keep it outside the five-file producer `out` evidence
directory; it remains inspectable evidence, never an external admission token.

## Producer preparation and canonical export

1. Build the bounded fixture with explicitly pinned source, lock, tools, native
   inputs and offline vendor state, using the existing Cargo producer/worker.
   Retain its actual build report and the five raw producer evidence files.
2. Freeze the evaluated producer and consumer derivations as JSON. Preserve the
   complete declaration, including inputs, dynamic-plan output declarations,
   addressing mode, policy environment and nested plan-output selector.
3. Export the existing source bundle and a signed store archive containing
   producer outputs, tool/native/source closure and unit outputs. **Do not include
   the observation consumer output in the proof archive**: clean replay must run
   the app rather than return a cached observation.
4. Export the existing receipt bundle for **all** archive paths, with explicit
   public verification keys and policy/window/revocation identities. Never use
   trust-unsigned, fabricate receipts or copy a publisher result database.
5. Supply the manifest facts and a JSON map of each artifact/evidence key to its
   actual producer-local payload path. Local filenames are export inputs only;
   they do not enter canonical identity. `bundle_blake3` is empty in an unsigned
   pre-export manifest; export canonicalizes and reports the final identity.

```sh
mantle --json cargo-build-bundle export \
  --manifest declared-manifest.json --inputs explicit-transports.json \
  --to handoff --compatibility client-compatibility.json \
  --policy-hash "$PUBLIC_TRUST_POLICY_BLAKE3" \
  --valid-at-unix-s "$VERIFICATION_TIME" --trusted-public-key "$TRUSTED_PUBLIC_KEY"
```

Export stages a fresh sibling directory, verifies all existing components and
publishes it without replacing an existing destination. There is no implicit
fetch. Identity uses compact typed JSON with ordered maps and normalized set-like
lists; it excludes timestamps and producer filesystem paths. Identical declared
inputs produce identical canonical manifest bytes, independent of source payload
locations. Changed tool/native identities cannot retain the old manifest digest.

## Clean compatible client

Obtain the final bundle identity and public compatibility/trust policy through
an independent authenticated channel. `inspect` reads metadata only; it does not
claim payload presence, plan admission, trust or execution.

```sh
mantle --json cargo-build-bundle inspect \
  --from handoff --compatibility client-compatibility.json
mantle --json cargo-build-bundle verify \
  --from handoff --compatibility client-compatibility.json \
  --expected-bundle-blake3 "$EXPECTED_BUNDLE_BLAKE3" \
  --policy-hash "$PUBLIC_TRUST_POLICY_BLAKE3" \
  --valid-at-unix-s "$VERIFICATION_TIME" --trusted-public-key "$TRUSTED_PUBLIC_KEY"
mantle --json cargo-build-bundle replay \
  --from handoff --client-dir /absolute/fresh-client \
  --compatibility client-compatibility.json \
  --expected-bundle-blake3 "$EXPECTED_BUNDLE_BLAKE3" \
  --policy-hash "$PUBLIC_TRUST_POLICY_BLAKE3" \
  --valid-at-unix-s "$VERIFICATION_TIME" --trusted-public-key "$TRUSTED_PUBLIC_KEY"
```

Add `--revocation-ref` and repeat `--revoked-public-key-digest` when the selected
public policy requires them. Verify/replay require the independent bundle hash;
export computes a new one. A policy label in a manifest is not trust authority.

Replay rejects an existing client directory. Only after the entire validation
prefix succeeds does it create state, import the signed archive, import/pin
sources, import verified receipts and invoke ordinary `mantle build
--no-substitute --offline-source-preflight`. Children have empty environments,
isolated HOME/CARGO_HOME/target locations and explicit store/state. They do not
consult a publisher resultDB. The clean-client proof additionally hides the
actual original checkout/Cargo home/target/resultDB using an outer mount
namespace and denies network with an outer network namespace.

Verification reports separate source payload verification, native structural
validation, trusted cached-result verification and execution. `native_plan_admitted`
is false until actual worker replay; replay checks canonical digest and exact
accepted unit IDs in the real worker report and rejects a noncached producer
observation. `execution_observed` remains false for inspect/verify/export.
An all-cache replay also leaves `execution_observed` false: worker admission and
trusted reuse are not observations of a newly executed process.
`receipt_evidence_complete` is reported independently. `client/build-report.json`
is the actual worker output, not a synthesized receipt.

## Two-package fixture and proof inputs

`fixtures/cargo-build-bundle` contains an app, reusable engine, engine build
script, native configuration payload, immutable lock and `observe.rs` consumer.
Declare the engine's native role as a signed store root containing `version.txt`:

```json
{
  "inputs": { "configuration": "/nix/store/<pinned-hash>-bundle-native" },
  "env": { "BUNDLE_NATIVE_VERSION_FILE": "{{role:configuration}}/version.txt" },
  "path": []
}
```

Key that declaration by the exact Cargo package ID, not `bundle-engine` alone.
The build script consumes the real declared file and affects the app's stdout.
For the offline Cargo parity lane, provide the same declared input/environment;
it must not discover the file through an ambient host path. Compile `observe.rs`
with the explicitly pinned static toolchain and transport the resulting signed
executable. It executes the real app twice and records exact stdout/stderr/exit
for normal termination and exit 7. No raw-binary equality is claimed.

Run `scripts/prove-cargo-build-bundle.py --campaign ABSOLUTE.json --output
FRESH_ABSOLUTE_DIR` after parent integration/build. It performs real export,
path-independent re-export, verify, early negative replay checks, network-denied
clean-client replay, actual app parity, exact unit reuse and unchanged rerun.
It never builds the fixture or manufactures publisher evidence. Its mandatory
campaign schema is `mantle-cargo-build-bundle-campaign-v1` with these inputs:

- `mantle`: final immutable compatible executable, outside hidden checkout;
- `compatibility`, `policy_hash`, `valid_at_unix_s`, `trusted_public_keys`, optional
  `revocation_ref`/`revoked_public_key_digests`, and `unrelated_public_key` (a valid
  public key not authorized for the archive);
- `bwrap`: explicitly admitted Linux namespace executable;
- `blake3_program`: explicitly admitted `b3sum`-compatible executable used only
  to reseal outer byte digests for negative inner-format checks;
- `native_payload_marker`: an exact nonempty UTF-8 byte sequence present in the
  transported native file, for example `baseline\n` for the checked fixture;
- `forbidden_client_paths`: exact existing paths keyed `checkout`, `cargo-home`,
  `target`, `resultdb`; output and executable must be outside these roots;
- `offline_cargo_observation`: actual default offline-Cargo app observation
  directory containing normal/nonzero `.stdout`, `.stderr`, `.exit` files;
- `observation_consumer_label`: the frozen app observer goal label;
- `variants`: objects keyed `baseline`, `app-edit`, `outside-edit`, `tool-edit`,
  `native-edit`. Each has absolute `manifest`, `inputs`, `producer_build_report`
  paths and explicit `expected_units_executed`, `expected_units_reused` counts.
  All are actual publisher builds from the corresponding immutable inputs.

The app-only edit must rebuild exactly its unit while retaining dependencies;
an outside-package edit must execute zero units and retain plan identity. Tool
and native changes must change bundle identity and execute affected units.
The driver requires a real native declaration even in the baseline, checks
publisher observations instead of assuming reuse, excludes a cached app observer,
and compares all six app observation files. Its retained raw command
stdout/stderr/exit and `campaign-report.json` are the proof entrypoints. Missing
prerequisites are failures, not skips or fabricated pass records.

Negative checks cover unknown schema/version/plan/source/native/planning
versions, incompatible target/prefix/network/limits, missing/tampered
source/archive/plan/config/lock payloads and an unauthorized signing key, asserting
no client directory was created. It also reseals only the outer declared byte
digest around changed inner source/receipt versions, a syntactically valid but
invalid Ed25519 signature, and corrupted native NAR bytes. These must fail inner
schema, existing signature or streamed NAR integrity checks before publishing an
export destination. Tool and native bytes share the explicitly signed store
archive; its complete missing/tamper and trust checks apply to both. Core tests
additionally cover tool/native identity invalidation, stale resolved roles,
undeclared roles, missing package sources/dependencies, cycles, policy mismatch
and unknown mandatory fields. Preserve the existing receipt suite's revoked and
expired signature cases; composition does not replace those checks with a new
verifier.

No successful campaign, end-to-end acceptance, compiler correctness,
reproducibility, Cargo-free execution, arbitrary Cargo compatibility, parity
outside the declared fixture, bootstrap fixed point or release eligibility is
asserted merely by adding these APIs or running their structural tests.
