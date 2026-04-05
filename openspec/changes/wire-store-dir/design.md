## Context

The v0 review identified this as a "mechanical but touches many call
sites" deviation. The plumbing exists at both ends — `--store` is
parsed in main.rs, and `_with_store_dir()` exists in nix-compat — but
nothing connects them. Every intermediate layer uses `to_absolute_path()`
which hardcodes `/nix/store`.

There are ~25 `to_absolute_path()` call sites across crunch-glue and
crunch-build, plus the `STORE_DIR` constant used in `build_request.rs`
for the sandbox `NIX_STORE` env and `inputs_dir`.

## Goals / Non-Goals

**Goals:**
- `--store /path` works end-to-end: path computation, filesystem
  checks, sandbox env, output display
- Tests don't need `/nix/store` write access
- No change to `StorePath` struct layout

**Non-Goals:**
- Store migration (moving outputs between prefixes)
- Multiple simultaneous store dirs
- Changing the store dir at the nix-compat level globally (we pass
  it explicitly)

## Decisions

### 1. Pass store_dir as &str through function parameters

**Choice:** Add `store_dir: &str` to `convert()`, `Builder::new()`,
and `derivation_to_build_request()`. Don't use a global or
thread-local.

**Rationale:** Explicit parameters are testable and
avoid hidden state. The call chain is linear
(CLI → convert → Builder → build_request), so threading one
parameter is straightforward.

**Alternative:** Store it in a global `static`. Rejected —
makes tests non-isolated and prevents concurrent builds with
different store dirs.

**Implementation:**

```
// crunch-glue
pub fn convert(drv: &CrunchDerivation, known_paths: &mut KnownPaths, store_dir: &str)

// crunch-build
pub fn Builder::new(..., store_dir: PathBuf, ...)
pub fn derivation_to_build_request(derivation: &Derivation, inputs: &..., store_dir: &str)
```

### 2. Add _with_store_dir variants to Derivation methods

**Choice:** Add `calculate_derivation_path_with_store_dir` and
`calculate_output_paths_with_store_dir` to vendored nix-compat.
The original methods remain as convenience wrappers that pass
`STORE_DIR`.

**Rationale:** Keeps existing nix-compat tests and callers
working. The vendored code is ours to modify but keeping the
existing API surface means less churn in the vendored crate.

**Alternative:** Change the original methods to require store_dir.
Rejected — would break every existing call site in nix-compat's
own tests for no benefit.

**Implementation:** In `derivation/mod.rs`:

```rust
pub fn calculate_derivation_path_with_store_dir(
    &self, name: &str, store_dir: &str,
) -> Result<StorePath<String>, DerivationError>
```

And similarly for `calculate_output_paths`. The inner calls to
`build_text_path` and `build_output_path` need `_with_store_dir`
variants too, which delegate to the existing
`build_store_path_from_fingerprint_parts_with_store_dir`.

### 3. KnownPaths stores store_dir

**Choice:** `KnownPaths::new(store_dir: &str)` stores the prefix.
Internal lookup keys use `to_absolute_path_with_prefix(store_dir)`
instead of `to_absolute_path()`.

**Rationale:** KnownPaths is the central index. If it doesn't
know the prefix, every caller needs to apply it before lookup,
which is error-prone.

### 4. Replace to_absolute_path() with to_absolute_path_with_prefix()

**Choice:** In crunch crate code (not vendored nix-compat),
replace every `to_absolute_path()` with
`to_absolute_path_with_prefix(store_dir)` where `store_dir`
is available from context.

**Rationale:** Mechanical but necessary. The grep shows ~25
sites. Most are in orchestrate.rs (filesystem checks) and
build_request.rs (placeholder replacement, refscan needles).

### 5. Cache test uses tempdir

**Choice:** The `builder_skips_build_when_output_exists` test
creates a tempdir, passes it as `store_dir`, and writes the
output file there. No more writing to `/nix/store`.

**Rationale:** The current test skips on NixOS because
`/nix/store` is read-only. A tempdir makes it run everywhere.

## Risks / Trade-offs

**[Churn in test assertions]** Tests that assert exact store
path strings (`"/nix/store/hyvs2ylk..."`) still work because
they use the default store dir. Tests that check filesystem
behavior switch to tempdirs. No test needs to assert paths
with a non-default prefix unless we add new tests for that.

**[Vendored nix-compat divergence]** Adding `_with_store_dir`
methods increases diff from upstream snix. Acceptable — we
already modified the hash function. These are additive methods,
not changes to existing ones.
