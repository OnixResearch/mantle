# Local kernel-bundle OCI round trip

This supported workflow imports two deterministic frontend artifacts, projects their checked admission records into an atomic local OCI image layout, and imports that layout into a fresh Mantle object store. It demonstrates the local export/import boundary without a registry or a frontend-specific evaluator.

Run from the repository root. The fixture projection is sealed against the exact checked-in spec, source admissions, object identities, media types, archive policy, and non-claims.

```sh
example=examples/projects/kernel-bundle-oci-local
work=$(mktemp -d /tmp/mantle-kernel-oci.XXXXXX)

mantle --json --state-dir "$work/source-state" \
  artifact import "$example/fixtures/vmlinuz" \
  --report-out "$work/kernel-import.json"
mantle --json --state-dir "$work/source-state" \
  artifact import "$example/fixtures/modules" \
  --report-out "$work/modules-import.json"

mantle --json --state-dir "$work/source-state" \
  artifact oci-export \
  --projection "$example/fixtures/projection.json" \
  --spec-material "$example/fixtures/kernel-bundles-spec.md" \
  --source-admissions "$example/fixtures/source-admissions.json" \
  --out "$work/layout" \
  > "$work/export.json"

mantle --json --state-dir "$work/import-state" \
  artifact oci-import \
  --layout "$work/layout" \
  --report-out "$work/import-report.json" \
  > "$work/import.json"

jq '{exported, projection_blake3, layout_blake3, layers}' "$work/export.json"
jq '{imported, state, layout_blake3, round_trip, objects, non_claims}' "$work/import.json"
```

The import must report `imported: true` and `state: "admitted"`. Every imported object carries a `mantle://blake3/...` reference, while OCI descriptors retain their separate `sha256:...` identities.

## Negative path

Copy and tamper with one referenced layer. Descriptor verification must fail before writing the requested import report:

```sh
cp -R "$work/layout" "$work/tampered-layout"
layer=$(jq -r '.layers[0].blob_sha256' "$work/tampered-layout/mantle-oci-export-report.json")
printf tampered >> "$work/tampered-layout/blobs/sha256/${layer#sha256:}"

if mantle --state-dir "$work/tampered-state" \
  artifact oci-import \
  --layout "$work/tampered-layout" \
  --report-out "$work/tampered-import.json"
then
  echo "unexpected tampered-layout acceptance" >&2
  exit 1
fi

test ! -e "$work/tampered-import.json"
```

## Deterministic validation rail

```sh
nix develop -c cargo test -p mantle --test kernel_bundle_oci_cli
nix develop -c cargo test -p mantle --bin mantle oci_projection_shell::tests
```

The first command exercises the checked gallery bytes through the public CLI, including a fresh-state positive round trip and tampered-layer rejection. The second covers deterministic archive construction, stale admissions, atomic publication, bounded readers, unexpected layout entries, and failed-report recovery.

## Non-claims

This workflow proves only a bounded local projection and descriptor-verified import over the supplied bytes and admissions. It does **not** publish to a registry, validate kernel compatibility, prove bootability or deployability, establish signature policy, or make a release-eligibility claim. Frontend semantics remain owned by the supplied spec and admission boundary.
