# Nario v2 read compatibility

Mantle can list and import Nario v2 archives from Determinate Nix 3.12.0. This support is bound to source revision `9512828397f684d0f732ea76b7631f69a0db34f7`.

## Supported operations

List an archive without target-store mutation:

```text
mantle store archive list --format nario-v2 --from closure.nario
```

Import exact `/nix/store` identities:

```text
mantle --store-prefix /nix/store store archive import \
  --format nario-v2 \
  --from closure.nario \
  --trusted-public-keys NAME:BASE64
```

Use `--trust-unsigned` only when policy permits unauthenticated original-path provenance. The default Mantle-native archive format does not change.

Nario export is not supported. Mantle rejects `store archive export --format nario-v2` before it writes output bytes.

## Checked boundary

The reader accepts the pinned Nario v2 magic and WorkerProto v16 metadata with short store paths. It checks named bounds, record order, duplicate paths, SHA-256 NAR identity, NAR size, references, signatures, supported content-address metadata, exact `/nix/store` identity, trailing data, and complete archive termination.

Import stages castore content first. It publishes all new PathInfo records through one backend transaction after the complete archive passes. Existing matching paths drain and verify payloads without reingest.

Direct import does not rewrite paths. A different logical prefix fails before payload admission.

## Foreign source projection

`foreign-import prepare-sources` can use `--nario-v2 ARCHIVE` when an admitted executable plan names the exact original Nix source path. `--nario-evidence-out PATH` is required.

Mantle verifies and materializes the original Nario record in temporary state. The existing source-bundle planner then canonicalizes it under the plan's target source identity. Original Nix signatures remain original-path provenance only. They do not sign or authorize the target source.

Every archive record must match one non-derivation source requirement. Unmatched records, duplicate matches, built outputs, wrong prefixes, unsafe NAR trees, and identity failures stop preparation before the source bundle is written.

## Compatibility evidence

- Authority: `config/nario-v2/authority.ncl`
- Positive producer fixture: `fixtures/nario-v2/positive-single.nario`
- Negative corpus: `fixtures/nario-v2/negative-*.nario`
- Fixture checker: `scripts/check-nario-v2-fixtures.rs`

The positive fixture was emitted by the pinned Determinate Nix binary with `nix nario export --format 2`. The negative fixtures are deterministic mutations of those producer bytes.

## Non-claims

Nario transports store data. It does not provide package recipes, derivation-graph completeness, Nixpkgs selection meaning, evaluator parity, package correctness, reproducibility, or release eligibility.
