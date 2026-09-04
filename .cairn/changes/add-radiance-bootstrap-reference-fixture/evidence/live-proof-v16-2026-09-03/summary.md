# Radiance live proof V16

## Result

V16 is the final proof run for implementation commit
`5e35c8a518e871cbbf844598b274ddb7842c9316`.

- Disposition: `match`
- Proof success: `true`
- Proof-time network requests: `0`
- Allowed execution events: `50`
- Denied execution events: `0`
- Fixed-point bytes: `1285492`
- Fixed-point BLAKE3: `a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07`
- Receipt BLAKE3: `da80e6d4f2fbf4adb82aabfaeef37e4f61e8a628024d999672458f0589e571c9`
- Protected-audit BLAKE3: `2da1ee4274c085598aef78258cbaec4fc84cc764d80db469356c198160dd004e`
- Plan BLAKE3: `c633f5936848ae2ecf29a462a9b6d1e58d0e0658ac872065cbee13d456505cc2`
- Source-state BLAKE3: `f7f7c80eff2072784b7e55f236129862ab1ea9cb713ab41f807a533190894cbd`

Stages two and three match in both routes. The two route fixed points also
match. Replay verified the receipt, publication, audit identity, and each
executable-tool relation.

## Source identities

- Source-bundle BLAKE3: `21f962ce6bf2ba89cbd94758d95abe8418cbafc243cdc81a146693d00f2539f4`
- Source-cohort BLAKE3: `80c82a016b6232aa4f8002624fa2fe6296ad3aac9fcf738fdbbad6c424993d4b`

The proof used only the prepared source bundle. It performed no source fetch,
substitution, fallback, or ambient discovery.

## Tool and runtime identities

The receipt binds eight roles:

- Compiler launcher: `bf001a7008d11536ffda1370cd0368d13b80ef7e6aa4c0f579ca8e74694acdb2`
- Compiler driver: `acfd6b4ca32ff8dd24bbc0a5254949f7e4568e7841ab1a8208b39f1c5326734e`
- Linker: `746fd75279b882f5cfb425627e593b3187ce348b0b8bdbda763361e7b3428c03`
- CRT input tree: `ebf9bad0070579d25d7b88a23089ebd5f82fd6e12fa47a64f54b006b939fa122`
- Libgcc input tree: `1df3e2cb7404f995ee4da9ac4f40c1bd4615969fa9435541f4ca7f5ffccf989d`
- Bootstrap compiler: `ac040ea04c2a83b521c87c53a2bce2cc77b7efbab52f781ad5e1e7af24e53c75`
- Emulator: `ce738d442238fa06553a857118b588b4c998ce963d6279389ef57394b3280672`
- RV64 seed: `a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07`

Mantle observed both runtime trees before and after the native builds. A change
would have rejected the run.

## Execution shape

Each C translation unit used one protected compiler root. Only the declared
launcher and resolved compiler driver could execute. Each native link used one
protected `ld.lld --threads=1` root. Every root installed the proof-time network
filter.

## Relationship to earlier evidence

V14 remains the first successful confined execution proof. V15 remains the
first eight-role receipt. V16 proves the final committed implementation.

V98 is unchanged. The Radiance fixture does not relabel, rerun, or extend the
V98 Mantle fixed-point proof.

## Non-claims

Equality proves bounded convergence for these exact inputs. It does not prove
compiler correctness, seed trust, semantic equivalence, or universal
reproducibility.
