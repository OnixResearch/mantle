## Context

`nix-compat-derive` is a proc macro crate vendored from snix. Its doctests
demonstrate derive macro usage:

```rust
# use nix_compat_derive::{NixDeserialize, NixSerialize};
#[derive(NixDeserialize, NixSerialize)]
struct Foo { x: u64 }
```

The expanded code references `::nix_compat::nix_de::NixDeserialize` etc.
In the original repo, a `[dev-dependencies]` on `nix-compat` made this work.
In crunch's vendored layout, the proc macro crate's `Cargo.toml` doesn't
(and shouldn't) depend on `nix-compat` for its library code — it only needs
it for doctests.

## Goals / Non-Goals

**Goals:** Make `cargo test --workspace` pass with zero failures.

**Non-Goals:** Don't rewrite the derive macros. Don't change the macro
expansion output.

## Decisions

### Option A: Add `nix-compat` as dev-dependency of nix-compat-derive

**Choice:** Add `[dev-dependencies] nix-compat = { path = "../nix-compat" }`
to `vendor/nix-compat-derive/Cargo.toml`.

**Rationale:** This is how the original repo worked. Doctests get
`nix_compat` in scope. No code changes needed.

**Risk:** Circular dependency concern — but dev-dependencies don't create
cycles in Cargo's dependency graph (they're only used for tests/benches).

### Option B: Mark doctests as `no_run` or `ignore`

**Choice:** Change ```` ```rust ```` to ```` ```rust,ignore ```` on all 17
doctest blocks.

**Rationale:** Quick fix. The doctests still serve as documentation but
don't try to compile.

**Downside:** Doctests rot silently. Code examples could become wrong.

### Option C: Add `#[doc(hidden)]` compile_fail

Not applicable — these aren't supposed to fail.

### Recommendation

**Option A** if the dev-dependency doesn't cause build issues. **Option B**
as fallback. Try A first.

## Risks / Trade-offs

**[Dev-dep cycle]** → `nix-compat` depends on `nix-compat-derive` (for the
proc macros). Adding `nix-compat` as a dev-dep of `nix-compat-derive`
creates `nix-compat-derive --dev--> nix-compat --> nix-compat-derive`.
Cargo allows this (dev-deps don't participate in normal resolution), but
verify it builds.
