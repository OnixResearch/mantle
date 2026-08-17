# Proposal: Repair CA archive final NAR metadata

## Summary

Repair content-addressed output persistence and Mantle-native store archives so `PathInfo.nar_size` and `PathInfo.nar_sha256` describe the final stored node after marker-to-store-path rewriting, while the pre-rewrite `CAHash` remains the bounded path-identity input. Archive export will reject stale final-NAR metadata before writing bytes, and archive import will validate CA path identity separately from final payload identity.

## Motivation

A retained full-source build state reproduces an exact archive failure for `57dqxg2kjddkvjmwkvkr54nifqfs38l0-bison-2.3-gcc-v6`: signed `PathInfo` records NAR SHA-256 `fff5e607c24805da4403a7619a5a967b61d0ea3fca268103773a47bde813f3eb`, but rendering its stored final node yields `c4bc724b6e5f92047cc57e6d9da09a71eb6c46110fac04254eba87062f1c6738`. Import fails inside `ingest_nar_and_hash` because it treats the marker-normalized CA hash as the final NAR hash.

The mismatch originates in the two-pass CA output shell: pass one hashes marker-normalized content to derive the stable store path; pass two rewrites those markers to final paths but persists pass-one NAR facts alongside the final node. This makes newly exported archives internally inconsistent even though archive framing and streaming are correct.

## Scope

- Recompute final NAR size and SHA-256 after every final marker rewrite and persist those facts with the final node.
- Preserve the marker-normalized CA hash used to derive the content-addressed store path.
- Add a pure validator that binds CA metadata to the signed store-path identity under the supported standard or Mantle marker-normalized path forms.
- Ingest archive NAR bytes without conflating CA path identity with final NAR identity, then verify payload BLAKE3, final NAR SHA-256/size, and exact reconstructed node before persistence.
- Preflight archive exports against freshly rendered final NAR facts before writing archive bytes.
- Add positive and negative tests, including a CA hash intentionally different from the final NAR hash and stale metadata that must fail before archive output.

## Non-goals

- No change to CA store-path derivation or marker rewriting.
- No migration that silently rewrites already-signed stale `PathInfo`; old state must fail closed until rebuilt or explicitly repaired by a future migration.
- No claim that the marker-normalized CA hash is a hash of final self-rewritten bytes.
- No change to the Mantle archive wire version or Nix nario compatibility claims.
- No weakening of signature, store-prefix, payload, node, or trust-policy checks.

## Target Spec Domain

- `store-transports`, refining deterministic export and idempotent import requirements around marker-normalized CA identity versus final NAR identity.
