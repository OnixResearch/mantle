## Context

The public seed contract is `{ name, target, dynamic_linker, toolchain, provider }`. `seed-full.ncl` normalizes outputs from `gcc-10.ncl`, `musl-full.ncl`, and `binutils-full.ncl`, but its provenance currently crosses a non-native GCC 4.0 pass1 bridge. The bridge synthesizes generator outputs, substitutes empty/backend objects, and installs wrappers that delegate compilation to TinyCC. Static contract checks therefore cannot authorize selection.

The first concrete frontier is the real GCC 4.0 `c-parse.o` build under the TinyCC/musl handoff. Existing v22 evidence records rc=2 after a generated `ssize_t` probe. Upstream live-bootstrap instead regenerates GCC 4.0's autotools, Bison, Flex, and libcpp artifacts before building the real subdirectories. Mantle already has the required source-built Perl/autoconf/automake/Bison/Flex stages, so the first candidate is an upstream-faithful derivation rather than another wrapper-local semantic stub.

## Decisions

### Decision: promotion requires real runtime artifacts

**Choice:** Treat a provider as selectable only when its evaluated closure excludes `seed-legacy.ncl`/musl.cc, every compiler ladder stage builds real source objects, and runtime smoke proves the normalized compiler, C++ driver, assembler/linker/archive tools, CRT, libc, and libgcc surfaces.

**Rationale:** Metadata shape, executable bit checks, wrapper version output, and contract-only receipts can all pass while compilation still delegates to TinyCC or uses fabricated generator objects.

### Decision: follow the upstream GCC 4.0 regeneration path

**Choice:** Regenerate the bounded GCC 4.0 files named by upstream live-bootstrap with receipt-bound source-built autotools, Bison, Flex, and Perl, then run the real `libiberty`, `libcpp`, and `gcc` makes. Do not preserve the current source-file dispatch wrapper or generated-object stubs in the admitted path.

**Rationale:** Twenty-two wrapper-local frontier reductions did not produce native `c-parse.o`, while the upstream mechanism is a materially different construction with a known source-chain role.

### Decision: keep selection separate from construction

**Choice:** Build and validate the candidate through `seed-full.ncl` first. Change `seed.ncl` only after durable runtime evidence exists from committed implementation source.

**Rationale:** This prevents an experimental or partially built provider from becoming the default bootstrap authority.

### Decision: reject host-assisted source-root relabeling

**Choice:** Do not reuse `mantle bootstrap --source-root` as completion evidence. It remains a host-assisted materialization operation whose metadata says `full_source_bootstrap_eligible: false`.

**Rationale:** Deterministic input/output identities do not erase host compiler, linker, make, extraction, kernel, or runtime influence.

### Decision: preserve the orchestration trust boundary

**Choice:** Report the provider as source-built within Mantle's declared sandbox/orchestration boundary. Do not claim compiler correctness, elimination of the 229-byte/bootstrap execution seed, or freedom from the sandbox shell/runtime unless separately proven.

**Rationale:** A real source-built artifact chain is stronger than the legacy musl.cc provider but is not a proof that every tool or execution substrate is correct.

## Risks / Trade-offs

- The upstream-faithful GCC 4.0 path may expose missing or non-functional autotools stages. Each new blocker must be captured by the smallest deterministic root build rather than bypassed.
- Full evaluation and build are expensive. Narrow predecessor roots and shared authenticated source state should be used before whole-provider and fixed-point reruns.
- The current Nickel graph can be expensive to evaluate. Evaluation latency is a blocker if it prevents bounded operator execution and must not be confused with a compiler-chain failure.
- A provider may compile simple C while still lacking C++, libgcc, dynamic runtime, or binutils completeness. Admission requires all declared surfaces.
