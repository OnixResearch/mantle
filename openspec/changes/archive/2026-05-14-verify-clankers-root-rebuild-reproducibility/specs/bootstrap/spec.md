## ADDED Requirements

### Requirement: Clankers root rebuild reproducibility proof

Crunch MUST record whether the pinned Clankers root derivation rebuilds to the same output binary BLAKE3 digest in a fresh Crunch store.
ID: bootstrap.external-fixed-bundle.clankers-rebuild-reproducibility

The proof MUST run `packages/clankers/clankers.ncl` from the committed source/bundle metadata without changing the derivation, compute BLAKE3 for the rebuilt `$out/bin/clankers`, compare it to the recorded original binary BLAKE3, and record a machine-readable receipt with command, store path, original digest, rebuilt digest, and verdict. A mismatch MUST be recorded as a failed reproducibility proof rather than silently updating the final proof hash.

#### Scenario: Fresh rebuild matches original binary digest

- GIVEN the committed Clankers root bundle and derivation
- AND the original proof records binary BLAKE3 `e1e8e1c36e0979a2534bcb8c394d4c360068985b0700b0e73ee32f1bd23917ff`
- WHEN Crunch rebuilds `packages/clankers/clankers.ncl` in a fresh store
- THEN the rebuilt `$out/bin/clankers` BLAKE3 equals the original binary BLAKE3
- AND the receipt records verdict `match`

#### Scenario: Fresh rebuild mismatch fails closed

- GIVEN a fresh rebuild produces a different `$out/bin/clankers` BLAKE3
- WHEN the reproducibility receipt is generated
- THEN the receipt records verdict `mismatch`
- AND the final proof hash is not updated to hide the mismatch
