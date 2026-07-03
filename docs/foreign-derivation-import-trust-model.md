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

## Claim-safe reporting checklist

Before reporting a foreign import result, name the strongest current evidence
class and keep missing layers explicit:

- **Admitted:** receipt and policy digests are valid; no realization claim.
- **Planned:** accepted import data produced a Mantle plan; no output trust claim.
- **Realized:** Mantle built or substituted an output; cite build report and
  store/attestation evidence.
- **Verified:** output signatures, hashes, attestations, and any requested
  release or witness policy were checked in the current run.
- **Blocked:** report the deterministic diagnostic and next action; blocked
  evidence is not proof success.

Use the exact non-claim language from this guide when summarizing admission-only
foreign import evidence.
