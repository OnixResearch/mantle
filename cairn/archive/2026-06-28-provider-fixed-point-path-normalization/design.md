## Context

The release source archive parity fix makes the source closure bytes match native path-source hashing, but rustc still receives absolute source paths and can inherit provider-specific compile-time environment such as `SNIX_BUILD_SANDBOX_SHELL`. Prior publisher and witness provider receipts showed different `source_closure_digest_blake3` values from source packaging and different `rustc_args_digest_blake3` values from absolute path surfaces. After source parity, the path surfaces remain a separate replay blocker.

## Decisions

### 1. Deterministic path mode is explicit and receipt-visible

**Choice:** Add a hidden rust-plan flag used by cargo-free provider fixed-point proof stages. The flag derives stable remap pairs from the already supplied `--root` and `--execution-output-root`, records the mode in `rust_plan.invocation`, and appends rustc `--remap-path-prefix` arguments to every unit.

**Rationale:** The mode is specific to provider/release proof execution. Keeping it explicit avoids silently changing general native rust-plan CLI behavior while making proof receipts auditable.

### 2. Provider compile-time paths become placeholders

**Choice:** In deterministic path mode, rustc child environments override `SNIX_BUILD_SANDBOX_SHELL` with a stable placeholder path. The runtime fallback path already treats missing compile-time sandbox-shell defaults as non-authoritative, so the placeholder prevents provider scratch paths from becoming part of release binary identity.

**Rationale:** `env!("SNIX_BUILD_SANDBOX_SHELL")` can otherwise bake witness or publisher local provider paths into the binary. The placeholder is a deterministic non-claim, not a runtime trust root.

### 3. Build-script execution keeps real working directories

**Choice:** Rustc compile environments may carry remapped metadata, but build-script execution must continue to use real package roots and OUT_DIRs for filesystem access. Source roots used for `Command::current_dir` are derived from the actual build-script source path when needed.

**Rationale:** Path normalization must not make build scripts unable to read checked-in inputs or write generated artifacts.

## Risks / Trade-offs

- Some crates can intentionally embed absolute paths through generated code or custom rustc env values. The deterministic mode narrows known Mantle/provider path surfaces but does not claim universal Cargo reproducibility.
- Adding remap flags changes rustc argument digests for provider proof receipts; this is desired and must be reflected in evidence.
