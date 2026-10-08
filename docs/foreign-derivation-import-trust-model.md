# Foreign derivation import trust model

Foreign derivation import starts with an admission and planning boundary. Mantle
can then realize an accepted plan through a separate receipt-bound adapter. It
does not run Guix, Nix, flakes, overlays, or package-module evaluation. An import
receipt is not realization evidence and is not output trust.

Use this guide when reviewing `foreign-derivation-graph-v1`,
`foreign-package-index-v1`, and foreign import receipts. It explains what a
receipt binds, which trust decisions remain separate, and which extra evidence is
needed before reporting trusted build outputs.

Concrete `/nix/store/*.drv` inputs use the reviewed boundary in
[`nix-derivation-compatibility-boundary.md`](nix-derivation-compatibility-boundary.md).
That boundary preserves Nix SHA-256 identity rules and Mantle-owned limits.
It does not change the foreign graph or receipt schemas.

## CLI usage

Validate lowered artifacts without invoking a foreign frontend:

```bash
mantle --json foreign-import validate \
  --graph tests/fixtures/foreign-import/guix-hello.graph.json \
  --package-index tests/fixtures/foreign-import/guix-hello.index.json \
  --policy tests/fixtures/foreign-import/policy.json
```

Emit a receipt-bound executable plan:

```bash
mantle --json foreign-import plan \
  --graph tests/fixtures/foreign-import/nix-hello.graph.json \
  --package-index tests/fixtures/foreign-import/nix-hello.index.json \
  --policy tests/fixtures/foreign-import/policy.json \
  --package hello \
  --system x86_64-linux
```

`validate` and `plan` read JSON artifacts in the CLI shell. The shell passes
owned data to the pure import core. `validate --receipt <path>` checks that a
receipt still matches the current graph and policy digests.

Prepare source records and realize an executable plan:

```text
mantle --json foreign-import prepare-sources \
  --plan plan.json \
  --source foreign-builder=./builder \
  --out source-bundle.json

mantle --json foreign-import realize \
  --plan plan.json \
  --import-receipt import-receipt.json \
  --source-bundle source-bundle.json \
  --source-bundle-blake3 <manifest-blake3> \
  --execution-profile profile.json \
  --receipt-out realization-receipt.json \
  --offline --no-substitute

mantle --json --state-dir ./state --store ./output foreign-import audit \
  --plan plan.json \
  --realization-receipt realization-receipt.json \
  --policy config/foreign-provenance-audit/generated/default.json \
  --root nix:hello \
  --out provenance-audit.json
```

See the [foreign realization operator guide](foreign-realization-operator-guide.md)
for the complete procedure and failure rules.

## Executable foreign plan

A successful `plan` report contains `mantle-foreign-executable-plan-v1`. The
plan binds the accepted import receipt, selected roots, and exact path maps. It
also binds native units, source requirements, and execution profile references.

Each native unit records canonical ATerm data and its exact target identities.
Foreign SHA-256 facts stay in the foreign digest domain. Mantle target and plan
identities use labeled BLAKE3 roles.

The plan proves compilation only. It does not prove source availability. It
does not prove scheduler execution. It does not prove store admission. It does
not prove realization or output trust. These facts require realization receipts.

## What an import receipt binds

A foreign import receipt binds the reviewable import inputs:

- producer identity and generator identity when known;
- frontend or source revision facts supplied by the graph producer;
- root derivation identities and package-index roots;
- raw graph BLAKE3 digest;
- translation policy BLAKE3 digest;
- translated graph BLAKE3 digest;
- explicit Nix-compatible hash-domain records when a Nixpkgs producer emits `.drv`, store-path, NAR, NARInfo, or cache identities;
- Mantle BLAKE3 receipt-domain records for graph, policy, and translated-artifact identities when hash domains are present;
- package-index digest when a package index is present;
- fetch/cache policy digest;
- sandbox compatibility policy; and
- explicit receipt non-claims.

Those facts make import admission auditable. A reviewer can inspect the graph,
policy digests, package roots, unsupported-feature diagnostics, and non-claims
without running the foreign frontend.

## What an import receipt does not claim

A valid import receipt alone does not claim build success. It does not claim
package correctness. It does not claim bootstrap parity. It does not claim output
trust. It does not claim reproducibility. It does not claim foreign-frontend
availability.

Additional realization and verification evidence is required before claiming
trusted outputs. Receipt existence is not proof of correctness. A receipt does
not prove source availability, scheduler execution, or store admission. It also
does not prove that Mantle can build or substitute the imported closure.
Treat a receipt as the input-admission record for later planning, build,
substitution, attestation, or release-proof workflows.

## Trust boundaries

The receipt keeps trust boundaries visible instead of merging them into one
success claim.

### Graph provenance

Graph provenance answers where the lowered store graph came from. Guix-like
producers may name a channel, commit, package expression, generator binary, and
system. Nix-like producers may name `.drv` paths, derivation-JSON exports,
flake-output discovery, a lockfile revision, or a package-index producer.

Mantle consumption trusts only the lowered graph facts that are present in the
artifact. A receipt does not prove that a live Guix or Nix frontend would still
produce the same graph later. If that claim matters, capture separate producer
transcripts and source-control evidence.

### Policy digests

Translation policy, fetch/cache policy, and sandbox compatibility policy are
separate receipt-bound inputs. A policy digest change means the imported graph
was admitted under different rules, even when the raw graph digest is unchanged.

Review policy digests before comparing two receipts. Do not treat unrecorded
operator preferences, frontend defaults, or environment variables as part of the
foreign import identity.

### Hash domains

Nixpkgs producer artifacts keep Nix-compatible identities separate from Mantle
receipt identities. `.drv` paths, Nix store paths, NAR hashes, NARInfo metadata,
and binary-cache lookup facts stay in the Nix-compatible hash domain required by
those formats. Raw graph digests, policy digests, translated artifact digests,
and import receipts stay in Mantle's BLAKE3 receipt domain.

A Nix-compatible identity must not replace a Mantle BLAKE3 identity. A Mantle
BLAKE3 identity must not replace a Nix SHA-256 identity. The executable plan
records algorithms, domains, and roles for these values. Validation fails before
planning or substitution when a value uses the wrong role.

### Source verification

Fixed-output source descriptors and ordered mirror lists are source policy.
They are not source contents. An executable plan maps each descriptor to a
source requirement. It does not prove that the bytes are available, fetched,
materialized, unpacked, patched, or built.

`prepare-sources` binds each required payload to a bounded source record. The
record preserves file content, executable modes, safe links, and directory-tree
completeness. Realization reconstructs this payload before exact-path store ingest.

Mantle verifies fixed-output downloads before store admission. Ordered candidates
retain plan order, and failed candidates appear in the realization receipt.
Trusted source-output claims still require the recorded store or attestation evidence.

For supported Nix fixed-output fetches, the producer normalizes `url`,
whitespace-separated `urls`, or structured `__json.urls` into one ordered graph
field. The compiler binds that field into derivation and plan identity. It then
emits the private runtime candidate binding. The fetch service does not parse
Nix fields or ambient mirror settings.

The producer rejects duplicate addresses, unsupported schemes, URL userinfo,
unresolved `mirror://` aliases, conflicting primary addresses, and more than 16 candidates.
A foreign graph cannot set `__mantle_foreign_candidates`. Candidate failure can
advance in order. A fixed-output mismatch stops the attempt before PathInfo
admission. This behavior does not prove mirror trust or arbitrary Nix fetcher
compatibility.

### Cache and substitution trust

Cache hints remain subject to store/substitution trust policy. A cache hint in a
receipt can describe where an output may be fetched from or which substitution
class was expected, but it does not bypass output admission or signature
verification.

A cached output is trusted only after normal store policy accepts its PathInfo signature, NAR hash, and store-prefix contract.
An artifact attestation remains subject to its separate policy when a consumer requires one.
Failed cache verification leaves the import receipt valid as admission evidence.
It does not create output trust.

For remote output consumers, a transfer checkpoint or acknowledgement cannot
introduce a session or replace signed PathInfo/content verification. The
[transient-handle admission rule](remote-transfer.md#transient-handle-admission)
names the establishing declaration and the checked fixtures for each boundary.

`preserve-cache-paths-v1` permits exact path identity only with one unchanged store prefix.
It selects the distinct `cache-only-preserve-v1` route.
This route requires online substitution, an empty source bundle, and one selected output root.

The plan binds trusted cache keys in its cache URLs.
Mantle validates bounded NARInfo metadata before it imports NAR content.
The metadata plan limits members, references, depth, NARInfo bytes, and total NAR bytes.
It also rejects paths outside the plan's exact identity map.

The route then imports trusted NARs through normal PathInfo and castore admission.
It reopens the store without remote services before scheduler observation.
The observer cannot execute foreign builder inputs or fall back to a local build.

### Sandbox capabilities

Sandbox capability records describe compatibility needs such as network access,
setuid/setgid behavior, writable prefixes, identity assumptions, or syscall
exceptions. They are policy-checked inputs for later realization planning.

A declared sandbox capability does not mean the build already ran with that
capability, and an undeclared capability must fail closed before realization.
Keep capability approval separate from build success and package correctness.

### Realization and output verification

Realization is the act of building or substituting an accepted plan into Mantle's
store. Output verification is the later check that admitted store paths, hashes,
signatures, attestations, and release evidence match the claimed result.

Import admission can succeed while realization is blocked, unsupported, or never
attempted. `mantle-foreign-realization-receipt-v1` binds the accepted plan, sources,
profiles, build report, PathInfo facts, fetch attempts, failures, and non-claims.

A `complete` receipt records successful observations for all selected roots. A
`partial-failure` receipt records completed work and the failure. Preflight rejection
creates no receipt and makes no store change.

The `audit` command reads signed PathInfo and castore content. It does not scan
exported host paths. It checks bounded files, ELF headers, scripts, links, tar
archives, newc initrds, and gzip or zstd streams. It normalizes safe lexical
store suffixes before it resolves references against plan-bound closure identities.

A passing `mantle-foreign-provenance-audit-v1` can report
`provenance-audited`. This state proves only the recorded bounded classification
and path-resolution facts. An audit failure keeps the prior `realized` state.
It does not rewrite the realization receipt or its build report.

Configured limits cover PathInfo, nodes, blobs, bytes, depth, findings,
duplicates, container entries, expansion, recursion, paths, and shebangs. Limit
exhaustion fails the audit and records the exhausted limit.

Realization can succeed while release reproducibility or independent witness policy
remains unproven. Report each layer with its own evidence. OnixOS still owns system
assembly, activation, deployment, boot, and machine-level evidence.

## Guix-like hello import

A Guix-like `hello` import can be reviewable without Guix at consumption time:

1. A producer lowers the selected package into `foreign-derivation-graph-v1` and
   optionally `foreign-package-index-v1`.
2. The receipt binds the producer facts, graph digest, policy digests, root
   derivation identity, source descriptors, sandbox compatibility policy, and
   non-claims.
3. Mantle validates admission from those files only.

Claim-safe summary:

> The Guix-like `hello` graph was admitted from explicit foreign import artifacts
> with receipt-bound policies. This does not claim build success, output trust,
> package correctness, bootstrap parity, reproducibility, or future Guix
> availability. Trusted output claims require later Mantle realization and
> verification evidence.

## GuixPkgs producer export

GuixPkgs checks translated Guix derivations into a Nix flake. A bounded producer
can export one package without running Guix:

1. Pin the GuixPkgs flake revision.
2. Record its `guix-metadata.json` Guix revision and locked `guix-transfer` revision.
3. Use producer-side Nix to export the recursive raw derivation graph.
4. Realize the selected translated root and sign its runtime closure under a
   dedicated exporter key.
5. Publish that closure as a Nix-compatible cache.
6. Stop the producer boundary before Mantle planning and consumption.

Mantle then uses `preserve-cache-paths-v1` and `cache-only-preserve-v1`. It
verifies the exporter signature, NAR facts, references, limits, and exact
`/nix/store` paths. It does not run Nix, Guix, GuixPkgs, or `guix-transfer`.

The exporter signature authenticates the exported translated bytes. It does not
prove Guix translation correctness or original `/gnu/store` identity. A failed
castore audit keeps `realized` as the strongest state and retains every finding.

The live `hello.unwrapped` proof reached `realized`, exact reuse, and fresh-store
hydration. Its audit retained one `unclassified-executable` finding. Guix glibc's
`bin/mtrace` has executable mode, no shebang, and an ambient `perl` invocation.
Therefore, this proof does not claim `provenance-audited`.

## Nix-like hello import

A Nix-like `hello` import follows the same boundary:

1. A producer exports concrete `.drv` or derivation-JSON facts plus a package
   index when package-name lookup is needed.
2. Mantle consumes the lowered graph and policy files. It does not evaluate Nix
   expressions, flakes, overlays, or module-layer package selection during
   consumption.
3. The receipt records import admission and explicit non-claims; later build or
   substitution evidence decides whether an output was trusted.

Claim-safe summary:

> The Nix-like `hello` graph was admitted from concrete derivation artifacts and
> receipt-bound policy. This does not claim build success, output trust, package
> correctness, bootstrap parity, reproducibility, or future Nix availability.
> Trusted output claims require later Mantle realization and verification
> evidence.

## Nixpkgs producer adapter levels

A Nixpkgs `hello` graph is claim-safe only when its level is named explicitly:

1. A producer adapter may use host Nix, flake output lookup, overlays, or a future
   `snix-eval` backend before artifact emission. That producer shell writes only
   concrete `foreign-derivation-graph-v1`, `foreign-package-index-v1`, and
   receipt inputs for later consumption.
2. Mantle consumption validates and plans from those lowered artifacts only. It
   does not evaluate nixpkgs, flakes, overlays, package-set replacement logic,
   `nix`, `nix-store`, or `snix-eval` while consuming the artifact.
3. Substitution-first planning may carry `cache.nixos.org` as receipt-bound policy data.
   A cache-only plan can preserve exact `/nix/store` paths under explicit policy.
   Every output still needs normal signature, NAR, prefix, and PathInfo admission.
4. A complete receipt can report `realized` after bounded runtime-closure hydration.
   A passing castore audit can then report `provenance-audited`.
5. Cache-only proof does not change that local rebuild compatibility is a separate level.
   Cache-only evidence does not prove a local build.

Claim-safe summary:

> The Nixpkgs `hello` graph was admitted from concrete derivation artifacts.
> The recorded graph then reached `realized` through trusted cache-only hydration.
> Its bounded castore audit reached `provenance-audited`.
> These states do not prove package correctness, local rebuild success, evaluator parity, reproducibility, bootstrap parity, runtime safety, or release eligibility.

The earlier export-to-plan evidence remains in
`cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/summary.md`.
The current archived change adds realization, reuse, audit, fresh-store hydration, and negative evidence.
Future Nixpkgs revisions and cache availability remain outside this proof.
The GuixPkgs proof is separate and retains its exact audit finding rather than
promoting the Nixpkgs audit result.

## Complete HTTP cache closure pull

Use explicit closure mode to import one root and its complete signed runtime
closure from a public Nix-compatible HTTP cache:

```text
mantle --nix-compat store pull \
  --from https://cache.nixos.org \
  --closure \
  --trusted-public-keys 'cache.nixos.org-1:...' \
  /nix/store/<hash>-<name>
```

Mantle fetches and validates every bounded NARInfo record before it requests NAR content.
The closure plan binds cache authority, trusted keys, store prefix, root, limits, and member metadata with BLAKE3.
The limits include member count, reference count, depth, NARInfo bytes, and total NAR bytes.
Nix store paths, NAR hashes, and NARInfo signatures keep their required Nix identity rules.

Mantle reuses a local member only when its PathInfo matches the plan, its
signature satisfies the selected trust policy, and its complete castore content
is present. It downloads dependencies before the selected root. A missing or
invalid dependency prevents root admission.

A complete foreign realization receipt can provide the closure root:

```text
mantle store pull \
  --from https://cache.example.invalid/mantle \
  --closure \
  --foreign-realization-receipt realization-receipt.json \
  --trusted-public-keys 'mantle-cache-1:...'
```

The cache must contain the receipt's target paths under the same logical store
prefix. Mantle validates the receipt schema, status, identity, and root before
store mutation. A successful pull proves cache admission under the configured
trust policy. It does not prove package correctness, local rebuild
compatibility, evaluator parity, reproducibility, or release eligibility.

## CLI observation boundary

The `foreign-import` `validate`, `plan`, `prepare-sources`, `realize`, and
`audit` operations declare their bounded file and store effects before their
host ports execute. Accepted plan and source outputs, realization receipts, and
audit receipts are read back from their committed paths and classified before
the command reports success. When Nario archives are used, source preparation
also checks the independently recorded imported archive count. Rejected
inputs and partial realization or audit outcomes retain their failure exit
instead of turning an observation failure into a successful report.

An embedded `mantlepkgs build` caller may inspect the persisted foreign
realization receipt through a pre-report callback; the direct `foreign-import`
CLI still renders the receipt once. This ordering permits the caller to
classify its own observed result before reporting it, without claiming that
the foreign frontend, package, or resulting output is correct.

## Claim-safe reporting checklist

Before reporting a foreign import result, name the strongest current evidence
class and keep missing layers explicit:

- **Admitted:** receipt and policy digests are valid; no realization claim.
- **Planned:** accepted import data produced a Mantle plan; no output trust claim.
- **Substituted:** Mantle accepted a cached output through store policy; no package
  correctness or reproducibility claim.
- **Rebuilt:** Mantle built an output locally; cite build report and sandbox
  compatibility evidence.
- **Provenance-audited:** Mantle scanned signed castore facts under recorded
  limits; no package-correctness or runtime-behavior claim.
- **Verified:** output signatures, hashes, attestations, and any requested
  release or witness policy were checked in the current run.
- **Blocked:** report the deterministic diagnostic and next action; blocked
  evidence is not proof success.

Use the exact non-claim language from this guide when summarizing admission-only
foreign import evidence.
