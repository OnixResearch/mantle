# ADR 0059: Source observations bind immutable facts; ingest is monotonic

## Status

Accepted (in the `bind-source-observations-and-monotonic-ingest` change).

## Context

Source-bundle v1 records keep origin and adapter facts in string maps, and
release `SourceAcquisition` uses structural strings for kind, URL, revision,
reference, and profile. A locator answers where bytes were observed; it does
not prove ownership, trust, or content identity, and mirrors and mutable refs
can move.

## Decision

Add a pure `no_std + alloc` core (`crates/crunch-source-core`) that admits a
structural request into a checked `SourceObservation` binding: source kind,
locator class, immutable Git revision and object format when applicable,
normalized relative projection, snapshot profile name and version, payload
BLAKE3, and a domain-separated observation BLAKE3.

- Locators are observations. Userinfo, fragments, and secret-bearing query
  fields are rejected; other query fields require explicit policy approval.
  Mutable refs may be recorded only as hints and never enter the identity.
- Git SHA-1 object IDs are admitted as protocol interoperability, not as
  stack identity; SHA-256 revisions are admitted at equal rank.
- Ingest planning is pure and returns exactly one of `Add`,
  `ReuseIdentical`, `RejectIdentityConflict`, or `RejectInvalid`. Only `Add`
  authorizes a durable create-new commit; reuse and rejection preserve
  durable state.
- Legacy v1 records remain readable. A projection is derived only from
  unambiguous metadata; otherwise the record keeps a valid legacy content
  identity with an explicit `provenance-unavailable` disposition. Legacy
  canonical bytes are never rewritten.

## Consequences

- Adapters (fixed URL, Git, local logical, package mirror, opaque) map onto
  one admitted observation shape; filesystem, network, Git, and release I/O
  stay in std shells.
- Release evidence binds the observation digest alongside the exact source
  content digest; no second signature role is introduced.
- Backfilled provenance is impossible by construction: incomplete legacy
  metadata yields an explicit unavailable disposition instead.

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
