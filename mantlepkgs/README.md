# Mantlepkgs

Mantlepkgs generates a bounded Mantle package catalog from concrete Nixpkgs derivation graphs.

Nix runs only in the explicit producer command. Catalog verification, package selection, planning, and building do not run Nix.

## Files

- `contracts.ncl` defines the typed Nickel manifest contract.
- `fixtures/` contains positive and negative manifest fixtures.
- `live-cohort/manifest.ncl` selects the pinned validation cohort.
- `live-cohort/policy/` contains the recompute policy and execution profile.

A successful generation contains these files:

- `catalog.ncl`: the generated, typed Nickel catalog.
- `catalog.json`: the machine catalog with its BLAKE3 identity.
- `artifacts/shared.graph.json`: the deduplicated foreign graph.
- `artifacts/packages.index.json`: package names, aliases, systems, and roots.
- `artifacts/source-requirements.json`: explicit source requirements.
- `policies/translation.json`: the bound recompute policy.
- `policies/execution-profile.json`: the bound execution profile.
- `producer-receipt.json`: the Nix producer and artifact receipt.

## Validate a manifest

```console
mantle mantlepkgs validate \
  --manifest mantlepkgs/live-cohort/manifest.ncl
```

This command evaluates Nickel and runs the pure manifest validator. It does not run Nix or write generated artifacts.

## Generate a catalog

Use one exact Nix executable. The producer records its canonical path, version, and BLAKE3 digest.

```console
mantle mantlepkgs generate \
  --manifest mantlepkgs/live-cohort/manifest.ncl \
  --nix-program /run/current-system/sw/bin/nix \
  --output-root target/mantlepkgs-live
```

The command stages all files in the generation directory. It validates the staged catalog before one atomic, no-replace rename.

During production, the command realizes each selected fixed-output dependency. It binds the exact Nix output path and canonical content BLAKE3.

The producer creates Nix GC roots under `<output-root>/.mantlepkgs-producer-seed-roots/`. Each root identity binds the source lock, producer, and package selector.

Keep this root directory until you prepare all required source bundles. External Nix garbage collection cannot remove the bound seed paths while these roots exist.

If one package fails, the command writes a failure report under `failures/`. It does not publish a success catalog.

## Verify and plan without Nix

```console
mantle mantlepkgs verify \
  --generation target/mantlepkgs-live/mantlepkgs/live-generations/<catalog-blake3>

mantle mantlepkgs plan \
  --generation target/mantlepkgs-live/mantlepkgs/live-generations/<catalog-blake3> \
  --package hello \
  --system x86_64-linux \
  --plan-out target/hello.plan.json \
  --import-receipt-out target/hello.import-receipt.json
```

Verification rejects path traversal, symlinks, missing files, digest changes, receipt changes, policy drift, and catalog identity changes.

Package selection accepts an exact package name or one unambiguous alias. A blocked package is visible, but it is not buildable.

## Prepare sources and build

Source preparation stays explicit. On the producer host, bind all recorded foreign source paths into one source bundle.

Do not remove `<output-root>/.mantlepkgs-producer-seed-roots/` before this step completes. The consumer does not use this producer-only root directory.

The command rejects content that differs from a producer-bound fixed-output seed:

```console
mantle mantlepkgs prepare-sources \
  --generation target/mantlepkgs-live/mantlepkgs/live-generations/<catalog-blake3> \
  --package hello \
  --system x86_64-linux \
  --out target/hello.sources.json
```

You can also use `mantle foreign-import prepare-sources` to bind different admitted paths for each source requirement.

```console
mantle mantlepkgs build \
  --generation target/mantlepkgs-live/mantlepkgs/live-generations/<catalog-blake3> \
  --package hello \
  --system x86_64-linux \
  --source-bundle target/hello.sources.json \
  --source-bundle-blake3 <manifest-blake3> \
  --plan-out target/hello.plan.json \
  --import-receipt-out target/hello.import-receipt.json \
  --receipt-out target/hello.realization-receipt.json \
  --signing-key /path/to/existing/signing-key \
  --offline
```

The build command always disables substitution. It imports verified seeds at their recomputed Mantle output paths before it starts the scheduler.

For modern Nix derivations, the producer retains `structuredAttrs` as canonical protocol JSON. The consumer writes `.attrs.sh` and `.attrs.json` under `/build`, then sets `NIX_ATTRS_SH_FILE` and `NIX_ATTRS_JSON_FILE`. It replaces known output placeholders before it writes these files.

The consumer implements Nix `passAsFile` with protocol SHA-256 paths under `/build/.attr-*`. It adds these payloads to the bounded build request. It also supplies the Nix protocol environment and the complete PathInfo input-reference closure.

Foreign Nix sandboxes expose normal `/proc` metadata and real random devices. Other sandbox profiles keep the existing metadata and random-device masks.

The command uses the existing foreign graph compiler, scheduler, worker, store, and realization receipt.

## Package dispositions

Each selected package has one disposition:

- `buildable`: graph, index, root, policy, and source facts passed generation checks.
- `blocked`: one or more ordered blocker codes prevent planning and building.

Examples of blockers include stale digests, missing roots, graph cycles, unsupported builtins, conflicting shared nodes, and opaque source-store assumptions.

## Claim boundary

A catalog and its receipts apply only to the exact source lock, selectors, systems, graphs, policies, profiles, sources, and outputs that they identify.

Mantlepkgs does not claim:

- Nix source translation;
- Nix evaluator parity;
- full Nixpkgs coverage;
- package correctness;
- reproducibility;
- bootstrap parity;
- release eligibility.
