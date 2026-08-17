# bundle provider fixed-point release evidence

## Problem

Mantle release verification can now validate an external provider-backed Cargo-free fixed-point proof bundle. That is useful for operators who already know which proof directory to supply, but it is still too easy for release evidence to omit the proof or for downstream verifiers to lose the sidecar path.

The next boundary should make the proof bundle part of the release evidence bundle itself. Release packaging should copy and hash the provider fixed-point proof, record it in `manifest.json`, and let release verification require the bundled proof without an extra sidecar path. The claim must stay bounded: a valid provider fixed-point proof supports the Cargo-free/source-built handoff evidence, not release reproducibility or full Cargo compatibility.

## Proposed change

Add a bundle-local provider fixed-point proof artifact to release evidence creation and verification. `mantle release create` should accept a provider fixed-point proof directory, copy it into the release evidence bundle, and record its digest and bounded role in the manifest. `mantle release verify --require-provider-fixed-point-proof` should use the bundle-local proof automatically when no external proof path is supplied.

Keep the existing external `--provider-fixed-point-proof` verifier as an override for operators validating standalone proof directories or older bundles.

## Success criteria

- Release creation can include a provider fixed-point proof directory as a first-class bundle artifact.
- Release manifest records the proof artifact path, digest, and bounded evidence role.
- Release verification can require and validate the bundle-local provider fixed-point proof without an external proof path.
- External proof paths remain supported and are reported distinctly from bundled proof evidence.
- Non-claims remain visible: not release reproducibility and not full Cargo compatibility.
