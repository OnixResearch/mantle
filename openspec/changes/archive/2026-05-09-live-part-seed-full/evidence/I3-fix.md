# I3 normalization hardening evidence

Task-ID: I3
Covers: bootstrap.part.seed.full

- Hardened retained-tool verification to check every expected target-prefixed provider binary.
- Added startup object checks (`crt1.o`, `crti.o`, `crtn.o`) and provider metadata checks.
- Reworded provider metadata so it describes the normalization contract and preserved source-chain intent without claiming end-to-end full-source promotion.
- Updated the change spec to allow explicit prerequisite-gated evidence.
