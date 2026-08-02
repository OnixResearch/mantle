# Design: live GuixPkgs export and Guix-free consumption

## Decision

Use GuixPkgs as the external translation boundary. Producer-side Nix evaluates
the pinned checked-in package expression and exports the recursive
`hello.unwrapped` derivation graph. The producer realizes the translated root,
signs its runtime closure, and writes a dedicated Nix-compatible cache. Mantle
then uses its existing `cache-only-preserve-v1` route against that cache.

This route preserves the translated `/nix/store` paths. It does not import the
original `/gnu/store` closure. A dedicated Ed25519 exporter key authenticates
the published cache facts.

## Producer boundary

The producer records these facts before consumption:

- GuixPkgs flake revision and source fingerprint;
- GuixPkgs `guix-metadata.json` channel, revision, and timestamp;
- pinned `guix-transfer` revision from the GuixPkgs lock;
- selected package attribute and raw translated derivation path;
- recursive concrete derivation graph and graph digest;
- signed export-cache URL, trusted public key, and closure root.

Nix and GuixPkgs are producer tools. Guix is not required because GuixPkgs checks
the translated graph into its repository.

## Functional core

Existing pure functions own these decisions:

- recursive graph and policy admission;
- exact preserved-path output maps;
- cache-only plan identity;
- bounded closure metadata planning;
- exact path, reference, NAR, and signature checks;
- realization and provenance receipt classification.

The live proof adds evidence, not a second graph compiler or cache protocol.

## Imperative shell

The producer shell may run Nix to evaluate and export GuixPkgs. It may realize
the translated root, sign its complete runtime closure, and write the export
cache. It stops before Mantle consumption starts.

The consumer shell uses a bounded `PATH` that contains Mantle and required host
utilities, but no `nix`, `nix-store`, `guix`, or `guix-daemon` command. Mantle
hydrates the selected root through its normal HTTP cache, PathInfo, castore,
scheduler, worker, and store services.

## Trust and identity

The proof distinguishes three identities:

1. GuixPkgs records the upstream Guix revision and translation source.
2. The exported recursive graph records the translated derivation identity.
3. The exporter signature and NAR hash authenticate the exported runtime bytes.

Mantle binds those facts in evidence. Mantle does not claim that the exporter
signature proves Guix's original `/gnu/store` output identity or translation
correctness.

## Rejected alternatives

### Add native Guix narinfo signature verification

Guix uses libgcrypt canonical S-expressions and a different substitute trust
model. This would add a new cache protocol when a bounded producer can publish
the translated closure through Mantle's supported Ed25519 cache format.

### Preserve the original `/gnu/store` closure

That route would require Guix cache parsing, Guix signature verification, and
new `/gnu/store` policy contracts. It would prove direct Guix substitution, not
the requested GuixPkgs export boundary.

### Rebuild the full translated graph locally

The graph reaches Guix's deep bootstrap and takes substantial time. It also
requires sandbox parity work. This proof concerns bounded signed substitution,
not source rebuild parity.

## Failure rules

- Missing or inconsistent producer metadata invalidates the proof bundle.
- Missing, inconsistent, or wrongly signed export-cache facts fail closed.
- The package root must belong to the exported recursive graph.
- Consumption must use an empty source bundle and the cache-only route.
- Missing, unsigned, wrongly signed, path-mismatched, NAR-mismatched, incomplete,
  or over-limit closure facts fail before complete realization.
- Mantle must not run a local or remote builder as fallback.
- Existing evidence files must not be overwritten.
