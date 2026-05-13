## Context

`../../clankers/` is a large Rust workspace with over thirty crates, Cargo git dependencies, private SSH-pinned `subwayrat` crates, sibling path-source assumptions, vendored `openspec`, and Nix/unit2nix reference packaging. Crunch already has `bootstrap/rust.ncl`, which fetches and installs a prebuilt musl Rust/Cargo toolchain. That is sufficient for the first external-workspace proof, but not sufficient for a full source-built toolchain claim.

## Goals / Non-Goals

**Goals:**
- Prove Crunch can build an external Rust workspace package from fixed inputs.
- Build Clankers incrementally, starting with a low-dependency crate and ending with the root `clankers` binary.
- Keep Cargo offline during derivation execution.
- Record enough evidence to debug missing source closure or native build-script dependencies.

**Non-Goals:**
- Prove source-built Rust/Cargo correctness.
- Run all Clankers tests, NixOS VM checks, or plugin bundle builds.
- Replace unit2nix or Nix packaging in Clankers.
- Fetch private/git sources from the network inside the Crunch sandbox.

## Decisions

### 1. Start with `clanker-message`, not the root binary

**Choice:** The first implementation task targets `cargo build --locked --offline -p clanker-message`.

**Rationale:** `clanker-message` is a small workspace crate and should expose the basic source/vendor/Cargo sandbox contract without immediately pulling in TUI, provider, daemon, `aws-lc-rs`, `ort-sys`, or plugin runtime dependencies.

**Alternative:** Start with `cargo build -p clankers`. Rejected because a failure would conflate source closure, toolchain, native build-script, feature, and root binary issues.

### 2. Use existing `bootstrap/rust.ncl` initially

**Choice:** The build ladder uses Crunch's current prebuilt musl Rust toolchain derivation for initial Clankers packages.

**Rationale:** The question is whether Crunch can drive an external Rust workspace build. Source-built Rust is a separate bootstrap parity problem and should not block the first package proof.

**Alternative:** Wait for source-built Rust. Rejected because it would delay learning about Cargo/source closure mechanics.

### 3. Freeze source closure before derivation execution

**Choice:** Implementation must construct or reference a fixed source/vendor closure before the Crunch derivation runs, then force Cargo offline inside the derivation.

**Rationale:** Clankers has git and sibling path dependencies; allowing live Cargo fetches or host sibling checkouts would make the Crunch result non-reproducible and unreviewable.

**Alternative:** Let Cargo fetch online in the sandbox. Rejected because it hides undeclared dependencies and cannot support bootstrap evidence.

### 4. Add native build tools only on demand

**Choice:** Later rungs add `cmake`, `go`, `pkg-config`, C toolchain, onnxruntime, or wasm target support only when the selected package actually requires them.

**Rationale:** This keeps the ladder small and makes each blocker actionable.

**Alternative:** Preload the full Clankers Nix dev/build closure. Rejected because it would obscure which Crunch package/tool is needed and reintroduce Nix-shaped assumptions.

## Risks / Trade-offs

**Cargo source replacement drift** → Mitigate by recording closure digests and failing before a success receipt if a git/path source is missing.

**Private SSH source availability** → Mitigate by resolving private sources outside the derivation into fixed inputs; the Crunch sandbox should consume only fixed source trees.

**Prebuilt Rust trust boundary** → Mitigate by labeling this as an external-workspace build proof, not a full-source/bootstrap parity proof.

**Feature creep into VM/plugin checks** → Mitigate by defining root binary smoke as the first complete success and leaving VM/plugin/source-built Rust claims for follow-up specs.

## Validation Plan

- `openspec validate build-clankers-with-crunch --strict`
- `./target/debug/crunch eval packages/clankers/clanker-message.ncl` and extract builder script syntax.
- `crunch build packages/clankers/clanker-message.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned`.
- For the root binary rung, verify `$out/bin/clankers` and a network-free `--help` or `--version` smoke.
