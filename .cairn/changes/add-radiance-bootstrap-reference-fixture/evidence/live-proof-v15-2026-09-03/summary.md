# Radiance live proof V15

## Result

The optional offline proof completed with disposition `match`.

- Proof success: `true`
- Proof-time network requests: `0`
- Allowed execution events: `50`
- Denied execution events: `0`
- Fixed-point bytes: `1285492`
- Fixed-point BLAKE3: `a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07`
- Receipt BLAKE3: `61b2bbd1b01b35692d83b2230dfbbce4294bbc927cdeee2cf5e7e858cb5d949f`
- Protected-audit BLAKE3: `cf46accf75d21eb66e685c25121ce79ac12469754e0a2ba5645bcb79a684b625`
- Plan BLAKE3: `c633f5936848ae2ecf29a462a9b6d1e58d0e0658ac872065cbee13d456505cc2`
- Source-state BLAKE3: `f7f7c80eff2072784b7e55f236129862ab1ea9cb713ab41f807a533190894cbd`

Both route-local stage-two and stage-three outputs match. The two route fixed
points also match. The rebuilt verifier replayed the receipt, publication, raw
audit identity, and executable-tool reconciliation with the same disposition.

## Source identities

- Source-bundle BLAKE3: `21f962ce6bf2ba89cbd94758d95abe8418cbafc243cdc81a146693d00f2539f4`
- Source-cohort BLAKE3: `80c82a016b6232aa4f8002624fa2fe6296ad3aac9fcf738fdbbad6c424993d4b`

The proof consumed the prepared source bundle. It performed no source fetch,
substitution, fallback, or ambient discovery.

## Tool and runtime identities

The receipt binds eight roles:

- Compiler launcher: `bf001a7008d11536ffda1370cd0368d13b80ef7e6aa4c0f579ca8e74694acdb2`
- Compiler driver: `acfd6b4ca32ff8dd24bbc0a5254949f7e4568e7841ab1a8208b39f1c5326734e`
- Linker: `746fd75279b882f5cfb425627e593b3187ce348b0b8bdbda763361e7b3428c03`
- CRT input tree: `ebf9bad0070579d25d7b88a23089ebd5f82fd6e12fa47a64f54b006b939fa122`
- Libgcc input tree: `1df3e2cb7404f995ee4da9ac4f40c1bd4615969fa9435541f4ca7f5ffccf989d`
- Built bootstrap compiler: `ac040ea04c2a83b521c87c53a2bce2cc77b7efbab52f781ad5e1e7af24e53c75`
- Built emulator: `ce738d442238fa06553a857118b588b4c998ce963d6279389ef57394b3280672`
- RV64 seed: `a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07`

Mantle observed the CRT and libgcc trees before the native builds. It observed
them again after both builds and rejected any change.

## Execution shape

Each C translation unit used one protected compiler root. Only the declared
launcher and resolved compiler driver could execute. Each native link used one
protected `ld.lld --threads=1` root. Every root installed the proof-time network
filter.

## Relationship to V14 and V98

V14 remains preserved as the first successful confined execution proof. V15 is
the final successor receipt because it also binds the compiler driver, CRT
tree, and libgcc tree as typed tool observations.

V98 is unchanged. This optional Radiance run does not relabel, rerun, or extend
the V98 Mantle fixed-point proof.

## Non-claims

Equality proves bounded convergence for these exact inputs. It does not prove
compiler correctness, seed trust, semantic equivalence, or universal
reproducibility.
