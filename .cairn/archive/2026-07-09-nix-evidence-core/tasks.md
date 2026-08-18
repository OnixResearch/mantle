## Tasks

- [x] [serial] Define store path ref, derivation/output identity, realization role, caveat, artifact digest, and non-claim contracts. r[mantle.release_provenance.nix_evidence_core.contract]
- [x] [serial] Add positive fixtures for Mantle build reports, release bundle rows, sidecar rows, and external Nix evidence rows. r[mantle.release_provenance.nix_evidence_core.fixtures.positive]
- [x] [serial] Add negative fixtures for malformed store paths, wrong outputs, digest mismatch, unsupported derivations, missing caveats, ambiguous roles, missing non-claims, and overclaims. r[mantle.release_provenance.nix_evidence_core.fixtures.negative]
- [x] [serial] Implement pure Nix evidence validation and DTO normalization. r[mantle.release_provenance.nix_evidence_core.validation]
- [x] [serial] Add compatibility adapters for build reports, release provenance, Cairn Nix gates, Molten promotion evidence, and Valence provenance inputs. r[mantle.release_provenance.nix_evidence_core.adapters]
- [x] [serial] Document build/evaluation shell boundaries and downstream migration order. r[mantle.release_provenance.nix_evidence_core.docs]
- [x] [serial] Run focused Nix evidence fixtures, Mantle release evidence checks, and Cairn validation/gates. r[mantle.release_provenance.nix_evidence_core.final_validation]
