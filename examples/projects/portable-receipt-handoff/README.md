# Portable receipt and semantic graph handoff

This project builds a local payload, exports its store archive and diagnostic receipt bundle, verifies/imports the receipt against archive facts in fresh state, and demonstrates complete and incomplete semantic graph queries.

## Build and export

Run from this directory on Linux with bubblewrap available:

```sh
work=$(mktemp -d /tmp/mantle-receipt-example.XXXXXX)
mkdir -p "$work/producer-store" "$work/producer-state" "$work/consumer-state"

mantle --json --store "$work/producer-store" --state-dir "$work/producer-state" \
  build .#payload --no-substitute > "$work/build.json"
physical_output=$(find "$work/producer-store" -maxdepth 1 -type d -name '*-portable-receipt-payload' -print -quit)
output_basename=$(basename "$physical_output")
logical_output="/mantle/store/$output_basename"

mantle --store "$work/producer-store" --state-dir "$work/producer-state" \
  store archive export --to "$work/payload.msa" "$output_basename"

mantle --json --store "$work/producer-store" --state-dir "$work/producer-state" \
  receipt bundle export --output "$logical_output" \
  --policy-hash gallery-policy-v1 --to "$work/receipt.json" > "$work/export.json"

mantle --json receipt bundle list --from "$work/receipt.json" > "$work/list.json"
```

## Verify and import in fresh state

```sh
mantle --json --state-dir "$work/consumer-state" receipt bundle verify \
  --from "$work/receipt.json" --archive "$work/payload.msa" \
  --output "$logical_output" --policy-hash gallery-policy-v1 > "$work/verify.json"

mantle --json --state-dir "$work/consumer-state" receipt bundle import \
  --from "$work/receipt.json" --archive "$work/payload.msa" \
  --output "$logical_output" --policy-hash gallery-policy-v1 > "$work/import.json"

# Equivalent re-import is idempotent.
mantle --json --state-dir "$work/consumer-state" receipt bundle import \
  --from "$work/receipt.json" --archive "$work/payload.msa" \
  --output "$logical_output" --policy-hash gallery-policy-v1
```

Inspect `claim_strength`, `evidence_complete`, `missing_evidence`, `output_matches`, `imported`, and `idempotent`. This example intentionally requests diagnostic evidence; it does not fabricate the source/action/sandbox chain required for a strong claim.

## Semantic graph queries

```sh
mantle --json graph gallery-output --graph-file fixtures/semantic-graph.json
mantle --json why gallery-output --graph-file fixtures/semantic-graph.json
mantle --json dependents source:portable-receipt-gallery \
  --graph-file fixtures/semantic-graph.json
```

The incomplete fixture must fail with a missing producing-recipe diagnostic rather than inventing an edge:

```sh
mantle --json why gallery-output --graph-file fixtures/incomplete-semantic-graph.json
```

A policy mismatch must fail before import:

```sh
mantle --json --state-dir "$work/consumer-state" receipt bundle verify \
  --from "$work/receipt.json" --archive "$work/payload.msa" \
  --output "$logical_output" --policy-hash wrong-policy
```

From the repository root, the focused conflict rail is:

```sh
nix develop -c cargo test -p mantle --lib \
  portable_receipt::tests::conflicting_graph_import_fails_without_persisting_bundle
```

Receipt verification proves only matched receipt, attestation, graph, archive-output, and trust-basis facts. It does not prove execution correctness, compiler correctness, payload transfer, full reproducibility, deploy success, or release eligibility.
