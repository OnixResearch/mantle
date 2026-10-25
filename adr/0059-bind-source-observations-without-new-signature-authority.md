# ADR 0059: Bind source observations without new signature authority

## Status

Proposed

## Context

Mantle source bundles identify canonical payloads with BLAKE3 and already support fixed URLs, local paths, VCS snapshots, mirrors, bootstrap archives, and proof inputs. Origin, revision, projection, and adapter facts can still remain in string metadata.

The [Atom Reforged architecture](https://nrd.sh/blog/atom-reforged.html) links source, revision, path, snapshot, and publication. Mantle needs the source relation, but it is a build tool rather than a package ownership or naming system.

Release attestations already sign the release-evidence manifest. Source review, build witnesses, and release signing have separate authority roles.

## Decision Drivers

- Bind immutable revision and snapshot facts to measured content.
- Keep URLs, mirrors, branches, tags, and local display paths non-authoritative.
- Preserve source-bundle v1 compatibility without inventing missing provenance.
- Make conflicting ingest fail without state mutation.
- Avoid duplicate signing and revocation policy.

## Decision

Mantle will define a versioned source observation with checked source kind, locator class, immutable revision when applicable, normalized projection, snapshot profile, content BLAKE3, and domain-separated observation BLAKE3.

A locator records where bytes were observed. It is not canonical content identity or ownership authority. Git identity uses the declared object format and immutable revision. A mutable ref can remain a non-authoritative hint. Mantle will not infer repository authority from one genesis commit.

A pure ingest planner returns add, identical reuse, identity conflict, or invalid rejection. Add uses atomic create-new publication. Reuse does not write. Every rejection preserves existing records, payloads, pins, roots, and readiness state.

Source-bundle v1 remains readable. A compatibility adapter creates a stronger source observation only when all required facts are present and unambiguous. Other valid v1 records keep a provenance-unavailable disposition.

Release evidence binds the accepted source-observation digest. The existing release signature covers that manifest linkage. Mantle will not add a source-observation signature role.

## Alternatives Considered

### Adopt Atom's package claim and publication protocol

Rejected because package naming, ownership, and version authority belong above Mantle's build-shaped boundary.

### Use URL or Git genesis as canonical source identity

Rejected because locations move, repositories can have multiple roots, and history can be rewritten.

### Backfill all v1 records from metadata

Rejected because incomplete metadata cannot support an honest immutable source relation.

### Sign each source observation separately

Rejected because the existing release, review, and witness roles already authenticate their own decisions.

## Consequences

- Source observation logic moves into a pure source core.
- Source adapters must name immutable revision and snapshot rules.
- New evidence gains one versioned identity while legacy bytes remain stable.
- Release verification gains an exact source-observation link without a new trust root.
- The observation proves a measured relation only. It does not prove ownership, review quality, or build correctness.

## Current adapter compatibility

The v1 shell requires explicit `projection`, `snapshot_profile`, and
`snapshot_profile_version` metadata before projecting an observation. Git
records additionally require explicit `git_object_format` (`sha1` or `sha256`)
and the corresponding immutable `rev`; length alone never chooses the object
format. Missing facts remain unavailable rather than receiving a default
profile, projection, or format. Explicit malformed facts reject during bundle
export, verification, and import; incomplete legacy records retain their
existing canonical bytes. The pure ingest planner independently re-admits a
stored observation and checks its digest before authorizing identical reuse.
These observation facts do not implement SHA-256 Git repository acquisition:
the pinned acquisition backend's supported object formats remain a separate
gate, as do reviewer-policy authority and genuine detached review signatures.

## Release binding and preparation

The seven-field release binding is a compact projection, not permission to
choose any non-content digest. `SourceObservationBinding::for_acquisition`
reconstructs its identity using `crunch-source-core`: exact archive payload,
kind/locator class, immutable Git revision/object format, logical projection
`source`, and `mantle-release-source` numeric profile version `1` (archive
format `v1`). Validation recomputes that identity. A stale revision, projection,
profile, or arbitrary observation digest is rejected. Mirrors and mutable ref
hints do not change identity. Absence still preserves generic release bytes.

For an existing genuine generic release bundle, the following preparation
command measures its actual source archive and emits the compact observation
input. It does not produce approval, sign anything, or accept a build proof.
`GENERIC_RELEASE_BUNDLE` must already name a real bundle produced by Mantle,
with acquisition metadata; output is outside the tracked source tree. Mantle
independently regenerates and validates this projection on consumption.

```sh
export GENERIC_RELEASE_BUNDLE=/absolute/path/to/existing-release-bundle
export SOURCE_OBSERVATION_OUTPUT=/absolute/path/outside-checkout/source-observation.json
python3 - <<'PY'
import json, os, pathlib, subprocess
root = pathlib.Path(os.environ["GENERIC_RELEASE_BUNDLE"])
manifest = json.loads((root / "manifest.json").read_text())
acquisition = manifest["source_acquisition"]
archive = root / manifest["source_archive"]["relative_path"]
payload = subprocess.check_output(["b3sum", str(archive)], text=True).split()[0]
assert payload == manifest["source_archive"]["digest_blake3"] == acquisition["digest_blake3"]
kind = {"external-archive": "fixed-url", "git": "git"}[acquisition["kind"]]
revision = acquisition.get("commit")
git_format = ({40: "sha1", 64: "sha256"}[len(revision)] if kind == "git" else None)
preimage = {
    "schema": "mantle-source-observation-v1",
    "kind": kind,
    "locator_class": "git-remote" if kind == "git" else "fixed-url",
    "git_object_format": git_format,
    "git_revision": revision,
    "projection": {"components": ["source"]},
    "snapshot_profile": {"name": "mantle-release-source", "version": 1},
    "payload_blake3": payload,
    "encoding_version": 1,
}
canonical = json.dumps(preimage, ensure_ascii=False, separators=(",", ":")).encode()
observation = subprocess.check_output(["b3sum"], input=b"mantle-source-observation-v1\0" + canonical).split()[0].decode()
binding = {
    "schema": preimage["schema"], "encoding_version": 1, "source_kind": kind,
    "payload_blake3": payload, "observation_blake3": observation,
    "snapshot_profile_name": "mantle-release-source", "snapshot_profile_version": 1,
}
with open(os.environ["SOURCE_OBSERVATION_OUTPUT"], "x") as output:
    json.dump(binding, output, indent=2)
    output.write("\n")
PY
```

With the same source checkout, genuine proof bundle, release binary, and
acquisition URL as the generic bundle, create a new bundle (never overwrite the
existing one):

```sh
"$MANTLE" release create --release-id "$RELEASE_ID" \
  --bundle-dir "$NEW_RELEASE_BUNDLE" --binary "$RELEASE_BINARY" \
  --proof-bundle "$GENUINE_PROOF_BUNDLE" \
  --source-acquisition-url "$SOURCE_URL" \
  --source-observation "$SOURCE_OBSERVATION_OUTPUT"
```

For Git acquisition, use the original `--git-source-url` and immutable
`--git-source-commit` instead of `--source-acquisition-url`. Changing tracked
source between preparation and creation makes the payload check fail.
These commands are preparation guidance, not a claim they ran or that a
reviewed-source policy was satisfied. The current external review prerequisite
is recorded in the source-review change's
`evidence/current-producer-contracts-2026-10-18.md`.

## Durable ingest boundary

Source imports preflight the entire incoming batch, including storage-key
collisions and existing pins, before creating state. Identical records and pins
are reused without rewriting. Publication uses staged create-new persistence,
so an intervening conflicting writer cannot be overwritten. A conflict found
by preflight leaves records, pins, and readiness unchanged. This is not a
multi-file transaction: interruption or an I/O failure during publication can
leave successful new additions. No stronger crash-atomicity claim is made, and
the source-built I4 compatibility/proof dependency remains independently open.
