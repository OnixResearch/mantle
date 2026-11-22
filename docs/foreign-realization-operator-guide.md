# Foreign derivation realization

Mantle can realize an admitted foreign derivation graph without running its foreign frontend.
The input must be a concrete, executable Mantle plan.
Mantle does not evaluate Nix, Nixpkgs, Guix, GuixPkgs, flakes, overlays, or package modules in this flow.

## Prepare a plan

Create the plan and import receipt together:

```text
mantle --json foreign-import plan \
  --graph tests/fixtures/foreign-import/realize-two-node.graph.json \
  --package-index tests/fixtures/foreign-import/realize-two-node.index.json \
  --policy tests/fixtures/foreign-import/realize-policy.json \
  --package two-node \
  --system x86_64-linux \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --plan-out plan.json \
  --receipt-out import-receipt.json
```

The execution profile is part of each derivation identity.
A profile change therefore changes target derivation paths.

## Review Nix fetch candidates

For supported concrete Nix fetches, inspect each graph node's
`fetch_candidates` field. The first address is primary. Later addresses are
fallbacks. Order changes derivation and executable-plan identity.

Mantle tries these addresses in order. Transport, Git, and I/O failures can
select the next address. A fixed-output mismatch is terminal. Mantle does not
consult `NIX_MIRRORS_*`, Nixpkgs mirror tables, or unresolved `mirror://`
aliases during realization.

Reject a graph that contains `__mantle_foreign_candidates` in its foreign
environment. Mantle creates this private binding only after graph validation.

## Prepare source records

Bind every non-derivation source requirement to an explicit local path:

```text
mantle --json foreign-import prepare-sources \
  --plan plan.json \
  --source foreign-builder=tests/fixtures/foreign-import/realize-two-node-builder.sh \
  --out source-bundle.json
```

The command accepts regular files and directory trees.
Directory records can contain bounded regular files and safe relative symbolic links.
The bundle records content, executable modes, symbolic-link targets, and BLAKE3 identities.

Copy the reported `manifest_blake3` value into the realization command.
Mantle rejects missing records, changed content, incomplete trees, and wrong modes before source ingest.
It does not read an ambient `/nix/store` or `/gnu/store` path to satisfy a requirement.

A Nario v2 archive can supply an exact admitted Nix source requirement:

```text
mantle --json foreign-import prepare-sources \
  --plan plan.json \
  --nario-v2 sources.nario \
  --nario-trusted-public-key NAME:BASE64 \
  --nario-evidence-out nario-source-evidence.json \
  --out source-bundle.json
```

Every Nario record must match one source requirement by its original path. Mantle rejects
unmatched records and built outputs. Original Nix trust remains provenance for the original
path. The target source identity is a separate Mantle content identity. See
[`nario-v2-import.md`](nario-v2-import.md).

## Realize a preserved Nix cache path

Use `preserve-cache-paths-v1` only with one unchanged `/nix/store` prefix.
The plan records the separate `cache-only-preserve-v1` route.
It maps each foreign output and source path to the same path.

Use this translation policy during planning:

```text
--policy config/foreign-cache-closure/generated/preserve-nix.json
```

Prepare the required empty source bundle without `--source`:

```text
mantle --json foreign-import prepare-sources \
  --plan plan.json \
  --out source-bundle.json
```

Then run cache-only realization:

```text
PATH=/path/without/nix mantle --json \
  --state-dir ./state \
  --store ./output \
  --store-prefix /nix/store \
  --remote-binding-config remote-endpoint-binding.json \
  foreign-import realize \
  --plan plan.json \
  --import-receipt import-receipt.json \
  --source-bundle source-bundle.json \
  --source-bundle-blake3 <manifest-blake3> \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --cache-closure-policy config/foreign-cache-closure/generated/default.json \
  --receipt-out realization-receipt.json \
  --substitute
```

The plan cache URL must bind each trusted public key.
Mantle supports one selected output root for this route.
It validates all bounded NARInfo metadata before it imports NAR content.
A metadata failure creates no output and no realization receipt.

Cache-only hydration accepts `http://` and `https://` cache URLs only.
`foreign-import produce-nix --cache-url` and `foreign-import plan` also accept
a `file://` URL, but realization can never hydrate from one. Serve a file cache
over HTTP, for example from a loopback-only server, and plan with that URL.

`--remote-binding-config` names a `mantle-remote-endpoint-binding-config-v1`
file, the operator's admission authority for every plan-bound substituter URL.
Without it, realization fails with `store-source-admission-required`. Opening
the store admits the cache URL with its `?trusted_public_keys[...]` query, but
hydration requests the URL without the query. Bind both exact forms, each with
the same scope and trust requirement; an unbound form fails with
`source endpoint admission denied`.

The shipped `default.json` cache-closure policy caps `max_nar_bytes` at 8 GiB.
A larger runtime closure is rejected before any import. Pass an operator
policy sized for the closure, for example a copy of the default with only
`max_nar_bytes` raised.

After hydration, Mantle reopens the store without remote services.
The ordinary scheduler observes the selected output as already present.
The cache-only observer has no inputs, arguments, or executable builder fallback.
Units outside the runtime closure report `not-required-cache-only`.

## Build a preserved Nix root from cache-hydrated inputs

`cache-inputs-build-roots-v1` builds the selected root instead of hydrating it.
Plan it explicitly with the same preserve policy:

```text
mantle --json --store-prefix /nix/store foreign-import plan \
  --graph graph.json \
  --package-index index.json \
  --policy config/foreign-cache-closure/generated/preserve-nix.json \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --realization-route cache-inputs-build-roots-v1 \
  --plan-out plan.json \
  --receipt-out import-receipt.json
```

Realization takes the cache-only flags and requires `--substitute`, a
`--cache-closure-policy`, one receipt-bound signed cache URL, and the
`--remote-binding-config` that admits that URL:

```text
mantle --json --state-dir ./state --store ./output --store-prefix /nix/store \
  foreign-import realize \
  --plan plan.json \
  --import-receipt import-receipt.json \
  --source-bundle source-bundle.json \
  --source-bundle-blake3 <manifest-blake3> \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --cache-closure-policy cache-closure-policy.json \
  --substitute \
  --receipt-out realization-receipt.json
```

The hydrate set is exactly the outputs the selected roots' direct
input-derivation edges request, the roots' own input sources, and the runtime
reference closure of that set as declared by the cache. Mantle does not walk
the transitive derivation closure. Every hydrated path must be inside the
plan's exact path map. Units outside the hydrated set are not built or
hydrated and report `not-required-cache-only`.

The route fails closed when a selected root output is offered by the cache or
already present in the store, when a selected root is a fixed-output fetch,
when an input is missing, unsigned, untrusted, hash-mismatched, or missing a
declared reference, and when a declared source exists only in the host logical
store. After hydration Mantle reopens the store without substituters and
executes only the selected roots. The receipt keeps the
`mantle-foreign-realization-receipt-v1` field set; roots report `built`,
hydrated inputs report `substituted` or `already-present`, and the strongest
state remains `realized`. `foreign-import audit` remains cache-only; a built
root is not provenance-audited by this route.

## Reuse a previously built preserved root

`cache-inputs-build-roots-v2` is the build-or-reuse route. It accepts the same
preserve policy, flags, and cache admission as route v1, then reuses a selected
root only when Mantle itself built and signed it earlier:

```text
mantle --json --store-prefix /nix/store foreign-import plan \
  --graph graph.json \
  --package-index index.json \
  --policy config/foreign-cache-closure/generated/preserve-nix.json \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --realization-route cache-inputs-build-roots-v2 \
  --plan-out plan.json \
  --receipt-out import-receipt.json
```

Realization takes the same flags as v1. Pass `--signing-key` to select the
builder key explicitly and repeat `--trusted-builder-public-key
<nix-name:base64>` to keep a retired builder key verify-only:

```text
mantle --json --state-dir ./state --store ./output --store-prefix /nix/store \
  foreign-import realize \
  --plan plan.json \
  --import-receipt import-receipt.json \
  --source-bundle source-bundle.json \
  --source-bundle-blake3 <manifest-blake3> \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --cache-closure-policy cache-closure-policy.json \
  --signing-key ./state/signing-key \
  --trusted-builder-public-key <retired-name:base64> \
  --build-cores 4 \
  --substitute \
  --receipt-out realization-receipt.json
```

`--build-cores <N>` (`1..=64`, default `1`) sets `NIX_BUILD_CORES=N` for every
build this realization runs when `N` is above the default. At the default the
build environment is unchanged: sandbox profiles export `1` and Nix-protocol
profiles leave the variable unset. It is a capacity knob, not part of derivation
or action identity: a derivation-declared `NIX_BUILD_CORES` still wins. Only
route `cache-inputs-build-roots-v2` records an explicit value (in
`roots[].built.build_cores`, omitted at the default), so a non-default value is
refused on every other route before any store mutation. Reuse ignores the value:
a root built with any core count is reusable, and receipts written before the
field existed still parse.

A present root is reused only when all of these hold:

- the write-once index at
  `<state-dir>/foreign-realization/built-roots/<root derivation basename>.json`
  holds a valid `mantle-foreign-realization-receipt-v2` whose recomputed digest
  matches its `receipt_blake3`, on route `cache-inputs-build-roots-v2`, with a
  `built` entry for the same derivation, output name and path;
- the NAR SHA-256 recomputed from the present bytes equals the entry's and the
  PathInfo's;
- a PathInfo signature verifies over the Nix fingerprint with a builder-role key
  whose public-key fingerprint equals the entry's signer fingerprint, and that
  key is trusted now.

The index is written only for a `complete` receipt and only for roots this run
built; a failed or reuse-only realization writes no index entry, so it cannot
block a later successful build of the same derivation.

Mantle takes one exclusive lock per selected root derivation under
`<state-dir>/foreign-realization/locks/` and holds it until the receipt is
published, so two concurrent realizations of one job produce exactly one
`built` and one `reused` receipt. A reuse run performs no hydration, download,
build, or signing.

A present root that fails a condition is refused before any store mutation with
one typed reason: `no-built-receipt`, `receipt-invalid`, `route-mismatch`,
`not-built`, `derivation-mismatch`, `nar-hash-mismatch`,
`no-valid-builder-signature`, `signer-mismatch`, or
`builder-key-not-trusted`. Mantle does not rebuild, delete, or re-sign a refused
root. Rotating the local signing key makes earlier roots refuse with
`signer-mismatch` until the retired public key is passed as
`--trusted-builder-public-key`. An absent root follows the v1 path; a root
offered by the cache is still refused. A different `--state-dir` has no
built-root history and therefore builds.

The v2 receipt keeps every v1 field and adds a required `roots` array, one entry
per selected root output: `node_id`, `target_derivation`, `output_name`,
`path`, `nar_sha256`, `nar_size`, sorted `references`, `disposition`
(`built` or `reused`), `signer {key_name, public_key_blake3, role}`,
`trust_policy_blake3`, `built`, and `reused`. Exactly one of `built` and
`reused` is non-null. A `reused` entry links the original `built` receipt's
`receipt_blake3` and records `source: "local"` with zero `builds` and
`downloads`; its root unit reports `already-present`.

A `reused` disposition proves only that an earlier Mantle build signed these
bytes. It is not fresh execution, reproducibility, or independent verification,
and a consumer such as Cairn compares the recorded fields and link rather than
verifying signature bytes itself. Routes other than
`cache-inputs-build-roots-v2` keep emitting receipt v1 unchanged, and route v1
still refuses every present root.

## Export a translated GuixPkgs package

GuixPkgs is a producer boundary. Pin its flake before export. Record the flake
revision, `guix-metadata.json`, and locked `guix-transfer` revision.

Use producer-side Nix to export and realize the raw translated package:

```text
nix derivation show --recursive <guixpkgs-hello-unwrapped.drv> > derivation-json.json
nix build --no-link <pinned-guixpkgs-hello-unwrapped>
```

Generate a dedicated Nix cache signing key. Sign the complete runtime closure,
then copy it to a Nix-compatible cache:

```text
nix key generate-secret --key-name <exporter-name> > export-secret-key
nix key convert-secret-to-public < export-secret-key > export-public-key
nix store sign --key-file export-secret-key --recursive <hello-output>
nix copy --to 'file:///path/to/cache?compression=xz' <hello-output>
```

Do not retain the secret key in evidence or consumer state. Put the public key
in the graph's receipt-bound cache URL. Percent-encode `+` as `%2B` in query
values. Stop the producer boundary before Mantle planning starts.

Serve the exported cache over bounded HTTP. Then use the preserved-path flow
above with a `PATH` that contains no Nix or Guix command.

## Realize locally

Run the ordinary Mantle registry, scheduler, worker, and store path:

```text
mantle --json \
  --state-dir ./state \
  --store ./output \
  foreign-import realize \
  --plan plan.json \
  --import-receipt import-receipt.json \
  --source-bundle source-bundle.json \
  --source-bundle-blake3 <manifest-blake3> \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --receipt-out realization-receipt.json \
  --offline \
  --no-substitute
```

Use `--root <foreign-derivation>` to select a subset of declared plan roots.
Use `--substitute` only when configured caches and trust keys can admit the required outputs.
Candidate source URLs keep plan order, and each attempt appears in the receipt.

Mantle rejects `--remote` for this adapter.
It also rejects stale receipts, unknown roots, profile mismatch, undeclared capabilities, and conflicting source content.
These checks occur before store mutation.

## Read the receipt

A successful run writes `mantle-foreign-realization-receipt-v1`, except on
`cache-inputs-build-roots-v2`, which writes
`mantle-foreign-realization-receipt-v2` with the `roots` array described above.
The receipt binds these facts:

- executable-plan, import-receipt, source-bundle, and build-report BLAKE3 values;
- selected roots and exact target paths;
- execution-profile identities and digests;
- source PathInfo and output PathInfo observations;
- build, substitution, fetch, reuse, and failure dispositions;
- cache policy, ordered fetch attempts, strongest state, and non-claims.

A failed build writes `partial-failure` evidence when execution started.
Preflight rejection writes no realization receipt and does not create store state.

A complete receipt proves only the recorded local realization observations.
It does not prove package correctness, evaluator parity, reproducibility, provenance, release eligibility, or future frontend availability.

## Audit realized provenance

Audit selected roots after a complete realization:

```text
mantle --json \
  --state-dir ./state \
  --store ./output \
  foreign-import audit \
  --plan plan.json \
  --realization-receipt realization-receipt.json \
  --policy config/foreign-provenance-audit/generated/default.json \
  --root nix:hello \
  --out provenance-audit.json
```

The command loads the existing signing key. It verifies every selected PathInfo
before content scanning. It then reads directories and blobs from castore only.
It does not use exported host files as replacement content.

The scanner classifies data, ELF files, scripts, links, tar archives, newc initrds, gzip streams, zstd streams, and libtool archives.
It scans bounded gzip and zstd payloads after decompression.
It normalizes lexical `.` and `..` suffixes only when they stay inside one store root.
Unknown executable bytes fail closed.
Invalid store digests, missing targets, link escapes, malformed containers, and exhausted limits also fail closed.

A passing receipt reports `provenance-audited`. A failed audit reports
`realized` as its strongest state. Both results preserve the original
realization receipt and build-report identity.

Review `policy`, `observation_blake3`, `findings`, and `non_claims`. The audit
cannot prove package correctness, runtime behavior, reproducibility, bootability,
deployment safety, or release eligibility.

## Hydrate a receipt-selected cache closure

A complete receipt can supply one selected root to an HTTP closure pull:

```text
mantle store pull \
  --from https://cache.example.invalid/mantle \
  --closure \
  --foreign-realization-receipt realization-receipt.json \
  --trusted-public-keys 'mantle-cache-1:...'
```

Mantle verifies the receipt schema, status, BLAKE3 identity, and selected root before store mutation.
The cache must contain the receipt's target paths under the same logical store prefix.
The cache path still uses normal signature, NAR hash, store-prefix, and PathInfo checks.
Mantle validates all closure metadata before it downloads content.
Receipt tampering, untrusted signatures, missing members, and exhausted limits fail closed.

## System boundary

This adapter owns package-graph source ingest and local store realization.
OnixOS still owns system intent, target assembly, activation, accounts, services, deployment, boot, and machine evidence.
A package realization receipt is not OnixOS deployment evidence.
