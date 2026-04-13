## Phase 1: Lock the stage0 contract

- [x] Update `openspec/specs/bootstrap/spec.md` with explicit requirements for a Nix-free first-bootstrap path and a separate non-Nix-host proof.
- [x] Update bootstrap docs/README so the current self-hosting proof is described as fixed-point evidence, not as proof that first bootstrap is already Nix-free.
- [x] Add an explicit stage0 prerequisite inventory covering host tools, fetched artifacts, and crunch-built outputs.

## Phase 2: Remove hidden Nix fallback

- [x] Remove the `nix-build` fallback from `scripts/prove-self-hosting.sh` and replace it with explicit seed provisioning or a clear missing-prerequisite error.
- [x] Audit `src/self_build.rs` and related helper paths for implicit Nix command invocation or Nix-only fallback behavior.
- [x] Keep NixOS-specific path probes clearly labeled as host-convenience discovery only, not as part of the Nix-free proof contract.

## Phase 3: Reduce stage0 host-tool edges

- [x] Replace host `git`/`tar`/`cp` staging glue with a Rust-side source staging path or an explicit prebuilt source bundle path.
- [x] Remove stage0 dependence on host `cargo vendor` by reusing a precomputed vendor tree or a checked source artifact.
- [x] Re-audit the first-bootstrap path after those changes and update the trust inventory with any remaining host prerequisites.

## Phase 4: Add proof for the stronger claim

- [ ] Add a repeatable proof or smoke path that runs with `nix-build`, `nix-store`, `nix-shell`, and `nix develop` absent from `PATH`.
- [ ] Record the stage0 prerequisite inventory inside the proof bundle so a reviewer can see exactly which external seeds were still used.
- [ ] Re-run `openspec validate non-nix-stage0-bootstrap` and the new non-Nix-host proof command before closing the change.
