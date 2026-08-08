# Design: fix Nix producer

## Context

The `foreign-derivation-import` spec defines a producer/consumer split. Producer shells evaluate foreign package sets and emit `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts. Consumers validate, translate, plan, and realize those artifacts without any foreign frontend. The spec already accepts explicit concrete `.drv` closures through `foreign_derivation_import.direct_drv_producer` and `foreign_derivation_import.drv_dir_closure_producer`, parsed with Mantle's own Rust ATerm code.

Today the nixpkgs producer either runs a host Nix (`nix-instantiate` / `nix eval`) or consumes `.drv` files that a host Nix wrote earlier. The host Nix binary is an ambient fact: its version, patches, and evaluation behavior are not Mantle build products and cannot be rebuilt from Mantle receipts.

`fix` evaluates Nix source to bytecode and instantiates derivations without requiring a Nix or Lix executable. Store-writing commands need a reachable daemon, but evaluation and instantiation do not. A direct build needs Zig 0.16, `pkg-config`, libcurl, and libgit2. This makes `fix` a candidate producer that Mantle can pin, build, and run under its own evidence rails.

## Decisions

### Decision: Pin the fix source as an ordinary source record

**Choice:** The `fix` source tree enters Mantle as a fixed-output source record that binds the upstream repository, the exact revision, and the content hash. The producer adapter and build derivations reference only this record. No ambient `git clone`, network fetch, or floating branch is permitted at build or adapter run time.

**Rationale:** Producer identity must be receipt-bound. A floating checkout would make producer behavior unobservable to the receipt chain.

### Decision: Build fix through the ordinary derivation pipeline

**Choice:** The Zig toolchain enters as a pinned fixed-output binary distribution derivation. The `fix` binary builds inside a Mantle sandbox derivation with Zig, libcurl, libgit2, and `pkg-config` as explicit inputs, running `zig build --release=fast`. The output gets signed PathInfo and an artifact attestation like any other Mantle build product.

**Rationale:** The producer binary must carry the same provenance and admission rules as every other output. A host-built `fix` binary would reintroduce the ambient-frontend problem one level down.

### Decision: Reuse the direct-.drv producer path

**Choice:** The `fix` adapter runs `fix instantiate` (or `fix eval` plus instantiation) into a bounded output directory, then hands that directory to the existing `.drv` directory closure producer. The adapter emits the same `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts as the host-Nix path. No new import ABI, translation rule, or receipt schema is added.

**Rationale:** `foreign-derivation-import` already owns ATerm parsing, closure selection, translation, and receipts. A parallel `fix`-specific artifact path would fork the ABI and double the audit surface.

### Decision: Run the producer under a bounded process policy

**Choice:** The adapter launches `fix` through an explicit bounded execution policy with named wall-time, memory, and output-size limits, an owned teardown sequence, and a confined working directory. Evaluation errors, timeouts, malformed `.drv` output, and oversized output fail closed with stable error classes. The adapter never executes `fix` store commands that require a daemon.

**Rationale:** The producer runs a foreign evaluator over arbitrary Nix expressions. Unbounded execution would let a hostile or pathological expression consume the operator process.

### Decision: Record compatibility evidence as bounded receipt data

**Choice:** Evidence records name the `fix` revision, the reference-Nix version, the language-suite pins, the nixpkgs universe pin, and the agreement counts (for example 80,586 of 80,586 matching `.drv` paths, including agreement on evaluation failures). Receipts state exactly which pins the evidence covers and MUST NOT claim general Nix equivalence, package correctness, or realization success.

**Rationale:** The differential results are strong but bounded evidence. Pin-bound honesty keeps the claim surface reviewable and prevents evidence drift when pins change.

### Decision: Keep hash domains separate

**Choice:** `.drv` files, store paths, NAR hashes, and NARInfo identity keep the Nix-required hash algorithms. Mantle-owned receipts, source records, policy digests, and artifact identities use BLAKE3. The adapter fails closed when a digest appears in the wrong domain.

**Rationale:** This mirrors `foreign_derivation_import.nix_hash_domain_boundary`. Cross-domain digest substitution is a silent identity corruption.

## Risks / Trade-offs

- `fix` is alpha-quality and x86_64-Linux-centric. The adapter must fail closed on unsupported platforms instead of degrading to a host Nix silently.
- A fixed-output binary Zig toolchain is a trust input, not a source-built compiler. The receipt must name it as such; source-built Zig stays a non-goal for this change.
- `fix` evaluation of a hostile expression can still burn the bounded budget. The bounded process policy caps the blast radius but does not make evaluation cheap.
- Differential evidence ages as upstream pins move. Evidence records must be regenerated when the `fix` or nixpkgs pin changes, or the receipt must mark the evidence stale.

## Validation

Positive fixtures cover: pinned-source admission, sandboxed `fix` build with attestation, `fix instantiate` of a small expression, artifact parity between the `fix` adapter and the host-Nix producer on the same expression, evidence record binding, and bounded cancellation.

Negative fixtures cover: source hash mismatch, floating revision rejection, unsupported platform, `fix` evaluation error propagation, malformed `.drv` output, oversized producer output, budget timeout, wrong-domain digest, and stale-evidence marking.
