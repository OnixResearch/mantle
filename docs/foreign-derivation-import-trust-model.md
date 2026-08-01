# Foreign derivation import trust model

Foreign derivation import is an admission and planning boundary. It lets Mantle
consume a lowered, adapter-neutral store graph without running Guix, Nix, flakes,
overlays, or package-module evaluation during consumption. The receipt is useful
because it binds the data and policies that were reviewed, but the receipt is not
realization evidence and is not output trust.

Use this guide when reviewing `foreign-derivation-graph-v1`,
`foreign-package-index-v1`, and foreign import receipts. It explains what a
receipt binds, which trust decisions remain separate, and which extra evidence is
needed before reporting trusted build outputs.

## CLI usage

Validate lowered artifacts without invoking a foreign frontend:

```bash
mantle --json foreign-import validate \
  --graph tests/fixtures/foreign-import/guix-hello.graph.json \
  --package-index tests/fixtures/foreign-import/guix-hello.index.json \
  --policy tests/fixtures/foreign-import/policy.json
```

Emit a receipt-bound adapter plan:

```bash
mantle --json foreign-import plan \
  --graph tests/fixtures/foreign-import/nix-hello.graph.json \
  --package-index tests/fixtures/foreign-import/nix-hello.index.json \
  --policy tests/fixtures/foreign-import/policy.json \
  --package hello \
  --system x86_64-linux
```

`validate` and `plan` read JSON artifacts in the CLI shell, pass owned in-memory
data into the import core, and print deterministic reports. `validate --receipt
<path>` additionally checks that an existing receipt still matches the current
graph and policy digests.

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
trusted outputs. Receipt existence is not proof of correctness, and receipt
existence is not proof that Mantle can build or substitute the imported closure.
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

A Nix-compatible identity must not be replaced with a Mantle BLAKE3 derivation or
receipt digest. A Mantle receipt identity must not be replaced with a Nix
SHA-256-compatible digest. If either domain is supplied in the wrong place,
validation must fail closed before planning or substitution.

### Source verification

Fixed-output source descriptors and mirror lists are source policy, not source
contents by themselves. A descriptor can say which hash or content reference a
later fetch must verify; it does not prove the bytes have been fetched,
materialized, unpacked, patched, or built.

Trusted source-output claims require later evidence from Mantle fetchers, source
hash verification, store admission, or attestation sidecars.

### Cache and substitution trust

Cache hints remain subject to store/substitution trust policy. A cache hint in a
receipt can describe where an output may be fetched from or which substitution
class was expected, but it does not bypass output admission or signature
verification.

A cached output is trusted only after the normal Mantle store policy accepts its
PathInfo signature, NAR hash, store-prefix contract, and artifact attestation
requirements. Failed or skipped cache verification leaves the import receipt
valid as admission evidence but does not create output trust.

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
attempted. Realization can succeed while release reproducibility or independent
witness policy remains unproven. Report each layer with its own evidence.

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
3. Substitution-first planning may carry `cache.nixos.org` or another binary
   cache as trust-scoped policy data, but every output still needs normal Mantle
   PathInfo signature, NAR hash, store-prefix, and artifact-attestation admission
   before output trust is reported.
4. Local rebuild compatibility is a separate level and is not proven by an
   admitted or planned Nixpkgs import receipt.

Claim-safe summary:

> The Nixpkgs `hello` graph was admitted from concrete derivation artifacts and
> receipt-bound policy. This strongest proven state is admitted or planned unless
> later substitution, rebuild, or verification evidence is cited. It does not
> claim nixpkgs package correctness, local rebuild success, output trust,
> bootstrap parity, reproducibility, or future producer availability.

Current live export-to-plan evidence for this boundary is captured in
`cairn/archive/2026-07-03-live-nixpkgs-foreign-import-proof/evidence/live-nixpkgs-hello/summary.md`.
That bundle records host-Nix `nixpkgs#hello` derivation export, Mantle
`produce-nix`, and no-Nix validate/plan consumption. Treat it as admitted/planned
only until separate substitution or rebuild evidence exists.

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

Mantle fetches and validates every bounded narinfo record before it requests NAR
content. The closure plan binds the normalized cache authority, trusted public
keys, store prefix, root, limits, and member metadata with BLAKE3. Nix store
paths, NAR hashes, and narinfo signatures keep their required Nix identity
rules.

Mantle reuses a local member only when its PathInfo matches the plan, its
signature satisfies the selected trust policy, and its complete castore content
is present. It downloads dependencies before the selected root. A missing or
invalid dependency prevents root admission.

The generic `store pull --closure` command is not yet bound to a foreign-import
receipt. The operator must compare the selected root with the accepted import
plan. A successful closure pull proves cache admission under the configured
trust policy. It does not prove package correctness, local rebuild compatibility,
evaluator parity, reproducibility, private-cache authentication, or release
eligibility.

## Claim-safe reporting checklist

Before reporting a foreign import result, name the strongest current evidence
class and keep missing layers explicit:

- **Admitted:** receipt and policy digests are valid; no realization claim.
- **Planned:** accepted import data produced a Mantle plan; no output trust claim.
- **Substituted:** Mantle accepted a cached output through store policy; no package
  correctness or reproducibility claim.
- **Rebuilt:** Mantle built an output locally; cite build report and sandbox
  compatibility evidence.
- **Verified:** output signatures, hashes, attestations, and any requested
  release or witness policy were checked in the current run.
- **Blocked:** report the deterministic diagnostic and next action; blocked
  evidence is not proof success.

Use the exact non-claim language from this guide when summarizing admission-only
foreign import evidence.
