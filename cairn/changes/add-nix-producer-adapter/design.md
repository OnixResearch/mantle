# Design: Nix producer adapter

## Context

The `foreign-derivation-import` spec defines a producer/consumer split. Producer shells evaluate foreign package sets and emit `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts. Consumers validate, translate, plan, and realize those artifacts without any foreign frontend. The spec already accepts explicit concrete `.drv` closures through `foreign_derivation_import.direct_drv_producer` and `foreign_derivation_import.drv_dir_closure_producer`, parsed with Mantle's own Rust ATerm code.

Today the producer side has two implicit frontends: a host Nix (`nix-instantiate` / `nix derivation show`) and pre-built `.drv` bundles. Neither names a contract. Producer identity is a free-form string, and swapping evaluators means editing call sites.

`fix` evaluates Nix source to bytecode and instantiates derivations without requiring a Nix or Lix executable. A direct build needs Zig 0.16, `pkg-config`, libcurl, and libgit2. It is the first candidate worth pinning and building as a Mantle product, but the same need will recur for Lix or `snix-eval`. The contract comes first.

## Decisions

### Decision: Define a versioned, backend-neutral producer contract

**Choice:** Add a pure core contract, `nix-producer-v1`, that every backend satisfies. Input: bounded expression references (file, expression text, attribute path, installable), evaluation arguments, system label, and backend policy. Output: a `.drv` closure directory, the selected root `.drv` identity, and typed producer identity facts (backend kind, backend version, source or binary identity, evaluation arguments digest).

The contract core is pure: it validates requests, classifies outcomes, and builds receipt inputs over in-memory data. Backend shells own process launch, file handoff, and output collection.

**Rationale:** A stable contract makes the evaluator a swappable component. Backends differ in launch and trust posture, not in what they owe the pipeline.

### Decision: Make backend selection explicit and fail closed

**Choice:** Producer policy names exactly one backend per run: `fix`, `host-nix`, or a future registered kind. Unknown kinds, missing backend binaries, and unsupported platforms reject before any evaluation. There is no silent fallback from a pinned backend to an ambient one.

**Rationale:** Silent fallback makes producer identity unobservable. The receipt must name the backend that actually ran.

### Decision: Pin and build the fix backend with Mantle

**Choice:** The `fix` source tree enters as a fixed-output fetch pin that binds the upstream repository, the exact revision, and the content hash. The `fix` binary builds inside a Mantle sandbox derivation with Zig, libcurl, and libgit2 as explicit inputs. The output gets signed PathInfo and an artifact attestation.

**Rationale:** The `fix` backend binary must be a Mantle build product with provenance. A host-built binary would reintroduce the ambient-frontend problem one level down.

### Decision: Admit two receipt-bound toolchain sources

**Choice:** Toolchain inputs (Zig, libcurl, libgit2) come from exactly two admitted source classes. The first is the pinned upstream binary tarball as a fixed-output source record. The second is signed nixpkgs binary-cache closures admitted through `mantle store pull` with explicit trusted keys, mounted as declared string store-path inputs. Ambient host paths are never toolchain inputs. Receipts name the source class and mark both classes as binary trust inputs, not source-built compilers.

**Rationale:** `fix` force-links libcurl and libgit2, and building that C library chain from source is a multi-rung effort that belongs to the bootstrap roadmap, not to this producer change. Signed cache substitution gives receipt-bound inputs with signature verification today. The pinned tarball remains the fallback when cache trust is unwanted. Both classes keep the build honest: every input is declared, hashed, and mounted, and nothing leaks from the host environment.

### Decision: Keep host Nix as an explicit ambient backend

**Choice:** The existing host-Nix producer path becomes the `host-nix` backend. Its binary path, version fact, and ambient trust posture are recorded in producer identity. Receipts must distinguish `host-nix` (ambient, not reproducible by Mantle) from `fix` (pinned, Mantle-built).

**Rationale:** Operators keep the current workflow, but the trust difference between backends becomes explicit receipt data instead of an assumption.

### Decision: Route all backends through the direct-.drv producer

**Choice:** Every backend's output is a `.drv` closure directory handed to the existing `.drv` directory closure producer. The adapter emits the same `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts regardless of backend. No backend-specific field enters the stable artifacts.

**Rationale:** `foreign-derivation-import` already owns ATerm parsing, closure selection, translation, and receipts. Backend-specific artifact paths would fork the ABI.

### Decision: Run backends under a bounded process policy

**Choice:** Each backend shell launches its evaluator through an explicit bounded execution policy with named wall-time, memory, and output-size limits, an owned teardown sequence, and a confined working directory. Evaluation errors, timeouts, malformed `.drv` output, and oversized output fail closed with stable error classes shared across backends. Daemon-requiring commands are rejected at the adapter boundary.

**Rationale:** A backend runs a foreign evaluator over arbitrary Nix expressions. Bounded execution caps the blast radius, and shared error classes keep contract conformance testable per backend.

### Decision: Record compatibility evidence per backend as bounded receipt data

**Choice:** Evidence records name the backend, its exact source or binary identity, the reference-Nix version, the language-suite pins, the nixpkgs universe pin, and the agreement counts. Receipts state exactly which pins the evidence covers and MUST NOT claim general Nix equivalence, package correctness, or realization success. When a backend pin drifts from its evidence pins, the receipt marks the evidence stale.

**Rationale:** Differential results are strong but bounded evidence. Pin-bound honesty keeps the claim surface reviewable per backend.

### Decision: Keep hash domains separate

**Choice:** `.drv` files, store paths, NAR hashes, and NARInfo identity keep the Nix-required hash algorithms. Mantle-owned receipts, source records, policy digests, and artifact identities use BLAKE3. The adapter fails closed when a digest appears in the wrong domain.

**Rationale:** This mirrors `foreign_derivation_import.nix_hash_domain_boundary`. Cross-domain digest substitution is a silent identity corruption.

## Risks / Trade-offs

- `fix` is alpha-quality and x86_64-Linux-centric. The backend must fail closed on unsupported platforms instead of falling back to host Nix.
- The Zig toolchain and the C libraries are binary trust inputs (upstream tarball or signed nixpkgs cache artifacts), not source-built compilers. Receipts must name the source class; source-built toolchains stay a non-goal for this change.
- Cache-substituted toolchain inputs trust the cache.nixos.org signer set. Operators that reject that trust must use the pinned-tarball source class, which trusts the ziglang.org distribution instead.
- A second backend doubles some test matrices. Shared contract conformance fixtures keep the cost proportional to backends, not to the product of backends and cases.
- Backend parity (same expression, two backends) can drift when Nix releases change semantics. Parity fixtures compare against recorded expectations per backend, not a claim that all backends always agree.

## Validation

Positive fixtures cover: contract conformance per backend, explicit backend selection, pinned-source admission, sandboxed `fix` build with attestation, `.drv` closure emission, artifact parity between backends on one bounded expression, evidence record binding, and clean cancellation.

Negative fixtures cover: unknown backend, unavailable backend binary, source hash mismatch, floating revision rejection, unsupported platform, evaluation error propagation, malformed `.drv` output, oversized producer output, budget timeout, daemon-requiring command rejection, wrong-domain digest, and stale-evidence marking.
