## Context

`foreign-import produce-nix` accepts concrete derivation-JSON closure facts. The missing proof is a live export path that starts with a real `nixpkgs#hello` drv path and ends with Mantle planning from emitted artifacts without a Nix executable available during consumption.

## Decisions

### 1. Evidence is checked in, generated artifacts stay evidence-local

**Choice:** Store the transcript and bounded generated artifacts under the change evidence directory. The generated graph/index prove the pipeline but are not promoted to ordinary test fixtures unless they become stable regression inputs later.

**Rationale:** The live closure can vary with the machine's nixpkgs registry revision. Treating it as evidence avoids pretending the exact closure is stable product data.

### 2. PATH-stripped consumption is the key proof boundary

**Choice:** Run `mantle foreign-import validate` and `mantle foreign-import plan` with a fake PATH containing no `nix` or `nix-store`.

**Rationale:** Host Nix is allowed only before artifact emission. Consumption must use the lowered JSON artifacts only.

### 3. Proof remains admission/planning-only

**Choice:** The evidence must record explicit non-claims for substitution, rebuild, output trust, package correctness, and reproducibility.

**Rationale:** Live export-to-plan does not exercise cache.nixos.org substitution or local Nixpkgs rebuild compatibility.

## Risks / Trade-offs

- Host Nix registry state can drift, so the proof records the resolved drv path and Nix version instead of claiming a fixed upstream revision.
- The generated closure can be larger than the small checked fixtures; evidence storage stays bounded to one `hello` proof.
