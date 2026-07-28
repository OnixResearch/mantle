# Offline source-bundle handoff

This project exports the source requirement constructed by `source-root.ncl`, verifies the bundle without mutating source state, imports and pins it into a fresh consumer state directory, and performs an offline readiness preflight.

File URLs in source evidence are explicit absolute locators. Materialize a checkout-specific build root in scratch, then run the workflow:

```bash
work=$(mktemp -d)
producer_state="$work/producer-state"
consumer_state="$work/consumer-state"
bundle="$work/source-bundle.json"
root="$work/source-root.ncl"

cat > "$root" <<EOF
let make_source = import "$PWD/source-root.ncl" in
make_source "file://$PWD/sources/payload.txt"
EOF

mantle --json --state-dir "$producer_state" source bundle plan \
  --build-root "$root"
mantle --json --state-dir "$producer_state" source bundle export \
  --build-root "$root" --to "$bundle"
mantle --json --state-dir "$consumer_state" source bundle list --from "$bundle"
mantle --json --state-dir "$consumer_state" source bundle verify \
  --from "$bundle" --imported
mantle --json --state-dir "$consumer_state" source bundle import \
  --from "$bundle" --pin
mantle --json --state-dir "$consumer_state" source bundle verify \
  --from "$bundle" --imported
mantle --json --state-dir "$consumer_state" source bundle preflight \
  --build-root "$root"
```

Before import, `verify --imported` reports one missing record without mutating the consumer. After the pinned import, verification and preflight report `ready`.

Inspect `record_count`, `ready_class`, `source_state_blake3`, and each file's BLAKE3 identity.

The source fetcher keeps its declared SHA-256 hash for interoperability. Mantle-owned bundle and state identities use BLAKE3.

For the negative path, copy the bundle, alter one `records[].files[].content_hex` value without updating its digest, and import it into a new state directory. Mantle rejects the digest mismatch and writes no source-state record.

Source-bundle evidence proves declared source availability and identity only. It does not prove build success, source trust, compiler correctness, or output correctness.

Source-bundle route execution is future work.
