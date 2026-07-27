## Context

`bootstrap/seed.ncl` selects the admitted full-source native provider, but `bootstrap/rust.ncl` still downloads the official Rust 1.94.1 standalone toolchain. Separately, `mantle bootstrap rust-source-provider` has produced a real source-built Rust provider and a zero-seed Cargo-free fixed point, but that proof used an earlier source-root native closure rather than the currently admitted provider. Reusing either result without rebuilding and rebinding the dependency closure would join claims only in prose.

## Decisions

### Decision: select the compiler-host route with a discriminating build probe

**Choice:** Probe an `x86_64-unknown-linux-musl` compiler-host route against the final shared musl/libgcc provider first. If Rust compiler dylib or proc-macro requirements still block that route, a GNU compiler-host route may proceed only with a separately authenticated source-built GNU native closure. The implementation must record the selected mechanism and exact rejected alternatives.

**Rationale:** The earlier GNU-host split solved real Rust bootstrap constraints, but importing it would reintroduce an unrelated native trust root. The final full-source provider now has stronger shared-runtime surfaces, so the former musl-host blocker must be retested rather than assumed.

### Decision: bind construction inputs, not only final Rust binaries

**Choice:** The Rust provider receipt must bind the admitted native-provider receipt, native provider BLAKE3, source-closure BLAKE3, Rust source identities, stage plan, stage outputs, and final provider artifacts. Every executable/runtime used by the claimed compiler build must be classified as source-built or as an explicit environmental assumption outside the claim.

**Rationale:** A final `rustc --version` smoke cannot distinguish a source-built compiler from an imported or wrapper-delegated compiler.

### Decision: keep validation pure and materialization explicit

**Choice:** Extend pure in-memory provider/closure validation separately from the shell that acquires sources, executes stages, hashes artifacts, and publishes the provider directory. Publication remains create-new and follows complete validation.

**Rationale:** The closure and claim logic must be testable without running the multi-hour Rust bootstrap.

### Decision: do not silently replace the development route

**Choice:** Keep fetched Rust available only under an explicit compatibility mode until the new provider has current construction and smoke evidence. Full-bootstrap modes must reject that compatibility route.

**Rationale:** Development availability and bootstrap trust reduction are different contracts.

## Risks / Trade-offs

- The musl compiler-host route may still be structurally incompatible with Rust compiler dylibs; that is a valid bounded blocker, not permission to use ambient GNU tools.
- Rebuilding Rust is expensive, so candidate probes must retain precise stage logs and stop at the first causal failure.
- Existing source-built Rust receipts remain historical evidence and cannot substitute for a current build against the admitted provider.