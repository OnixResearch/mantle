# Reviewed Onix kernel-bundle fixtures

These files are exact snapshots from the sibling OnixOS checkout at accepted
commit `8a99461` (`define the portable kernel-bundle evidence boundary`).

| File | Source | BLAKE3 (hex) |
|---|---|---|
| `kernel-bundles-spec.md` | `cairn/specs/kernel-bundles/spec.md` | `a87e25785b74338bf1bc3c77a24f7cdf9ce17c3a581ae60f1174a2b8584bda01` |
| `full-expected.json` | `tests/fixtures/kernel-bundle/full-expected.json` | `5389f7b9dc30cf6a08664c8a3db340d26ea9044f991f68b9108e2e9140bcb1ae` |
| `minimal-expected.json` | `tests/fixtures/kernel-bundle/minimal-expected.json` | `3df16f474661fa901dc13a6bc987eb44f335528b87758f754f1ae04fbf8dae68` |
| `import-admitted-expected.json` | `tests/fixtures/kernel-bundle/import-admitted-expected.json` | `e5200a4eed5893b6e7ef7ae1b0be8ff726f3c3e8c57ae93e4c87ac2b6abc4d30` |
| `import-compatibility-only-expected.json` | `tests/fixtures/kernel-bundle/import-compatibility-only-expected.json` | `55b6a469e08540748af4cb871d00d057bec9ee62397a1e73c7aa45b95f50c67e` |

The Onix projection is a frontend-neutral handoff, not a directly admitted
Mantle export request. Its generic `blake3:` references identify frontend
objects and it intentionally omits Mantle CAS admissions, platform selection,
canonical archive policy, and exact layer mode. The bounded adapter must first
import each object into Mantle's CAS, preserve the Onix identity fields, attach
path-free reductions of successful `mantle-frontend-artifact-admission-v1`
attestations bound to this spec snapshot and their source-attestation BLAKE3s,
retain the exact source attestations as a separate export-only bundle, choose
explicit exact-blob or canonical-archive modes, and seal the resulting
`mantle-oci-projection-v1` document. Mantle rejects the raw Onix
fixture rather than filling those fields from host state or hidden defaults.

The import-response snapshots define the frontend boundary: a Mantle-profile
round trip can supply evidence for Onix to reconstruct and re-admit every
identity; an external safe OCI/KBI import remains `compatibility-only`.
