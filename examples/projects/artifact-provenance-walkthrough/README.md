# Artifact provenance walkthrough

This project follows one built artifact from its declared dependency through persisted artifact and runtime-closure attestations. Run from this directory with writable scratch store/state paths:

```sh
work=$(mktemp -d /tmp/mantle-provenance-example.XXXXXX)
mkdir -p "$work/store" "$work/state"

mantle --json --store "$work/store" --state-dir "$work/state" \
  build .#artifact --no-substitute > "$work/build.json"
mantle --store "$work/store" --state-dir "$work/state" \
  build .#source --no-substitute >/dev/null
artifact=$(find "$work/store" -maxdepth 1 -type d -name '*-provenance-walkthrough-artifact' -print -quit)
source=$(find "$work/store" -maxdepth 1 -type d -name '*-provenance-walkthrough-source' -print -quit)

mantle --store "$work/store" --state-dir "$work/state" attest show "$artifact" \
  > "$work/artifact-attestation.json"
mantle --store "$work/store" --state-dir "$work/state" attest closure "$artifact" \
  > "$work/closure-attestation.json"
mantle --store "$work/store" --state-dir "$work/state" attest verify artifact "$artifact"
mantle --store "$work/store" --state-dir "$work/state" attest verify closure "$artifact"
mantle --store "$work/store" --state-dir "$work/state" attest diff "$source" "$artifact"
```

Inspect these items:

- the build report's `artifact_attestation`
- the artifact envelope's `facts`
- the closure members and edges
- the diff between the source and assembled artifacts

The walkthrough test changes persisted sidecars and removes the selected root from cached closure membership. Canonical verification then fails. Mantle also rejects a missing selector.

These attestations bind canonical recorded claims, observed build facts, store identities, and dependency linkage. They do **not** prove that the builder, source text, dependency, compiler, or resulting behavior is correct, and they are not release or witness proofs.
