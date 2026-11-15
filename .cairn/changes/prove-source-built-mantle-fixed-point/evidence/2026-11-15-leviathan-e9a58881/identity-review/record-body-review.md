# Reusable original-source record-body review (stale e9 cohort)

## Bounded result

PASS only for preservation of the 59 original native-source record bodies in the currently published `prepared-cohort-e9a58881-typed-index`. This cohort is the pre-fix subject; the parent reported a deferred-invocation type mismatch and is regenerating/qualifying a corrected subject. This result does not transfer to a regenerated cohort and is not a complete-cohort, prepared-profile, cold-selector, launch, fixed-point, compiler-correctness, or release verdict.

The separate frozen-source/original AWS-LC identity pass remains documented at `../luna-frozen-originals/report.md`; it measured frozen tree `e62509cd40ab6e4b202dce425a40483d74101210c6322c64f70b196ff7e177f7`, original AWS-LC Git commit/tree `02561621ffa4cf17c0c4f70bc11a82df36b42ae9` / `969d75db5c0494de4a26bbb2dcbf6837058054c1`, and byte/mode/member equality for both captured and frozen copies against all 10,427 Git blobs. That earlier bounded pass is not a result for the regenerated source.

## Measurements on the current published e9 subject

- The report's input profile identity is `5974bc464d7bb0f357a89a60dd18f147413500a8533ccd7d0007644ee987c147`; its declared input-plan identity is `87a66a10c559307b92d3aeac145f22251019cd4ad32d901bb9ca031da1432525`, matching the plan's declared digest. The report BLAKE3 is `3c848d614b58613cafb88f2eaf1affad95fa18c642f49eb7b95df4932adc5b96`; the raw input-plan BLAKE3 is `7b511e18bb57dcb2c65a4502281018f47863204624004dbd20db5dd2e29f7c1e`.
- The report identifies output profile `3096052305ed9bc2cf0b23b3291dc98153f5b090d7f9abe726368638f883a946`. The profile file is 15,790,943,073 bytes. Its raw BLAKE3, `32b865ac836fa247a6a20b63fd5dd38530c6a455a39dfe84344694be329806cf`, is the parent's prior full-stream transfer-parity measurement, not a hash recomputed by this record-body pass.
- The original closure at `native-source-closure.json` is 968,316,247 bytes with BLAKE3 `da4860b26b700f044a5280f638b195b37142d06491e88a018041e1bfe5952e3f`; the script checked this against the native-source-bundle role's declared digest and size in the plan.
- The augmented closure at `inputs/native-source-authority/native-source-closure.json` is 1,632,934,959 bytes with BLAKE3 `6d107fc17bd71b213df96dae2bddfdf9aa2da79c3d7c2fb0797f3aec259f40f7`.
- Streaming SHA-256 over each complete serialized record object (normalizing only the final JSON-array comma on its closing line) found exact identity/body matches for all 59 original records in the augmented closure. Duplicate record identities were rejected; the augmented closure has 63 records total and exactly four additional records, each once: `fixed-point-native-source-program`, `fixed-point-rust-source-program`, `source-built-mantle-programs`, and `source-built-stagex-programs`.

The retained auditor is `audit-original-native-records.sh`; `audit.log` and both full record-id/SHA-256 row inventories are adjacent. Their BLAKE3 values are respectively: script `4b33bed9692efe78cdcf81a87b608ccae978add88fe55f8772fb2f617a8b9dc8`, log `708b7fc3fa698e1fb92c7cf6a6968009947e866bab48dea38bffc67cb99fcfe5`, original rows `66f76f83a7bb3048107cc747d6b7eea11dbfad0bb58b22d62996f74744992a53`, augmented rows `dffddff88cb3317002477e86b149ac5e46058a7e2baa82d6aaa7ba4b059f8383`.

## Reproduction

Run from the repository root; the script writes only into the distinct inert remote scratch `/tmp/luna-e9a58881-identity-preservation-gpt6luna` and reads the current cohort. The local result and row files were collected into this directory.

1. Create the scratch directory once:
   `ssh leviathan.cymric-daggertooth.ts.net 'test ! -e /tmp/luna-e9a58881-identity-preservation-gpt6luna && mkdir -m 700 /tmp/luna-e9a58881-identity-preservation-gpt6luna'`
2. Copy the retained inspector:
   `scp .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/identity-review/audit-original-native-records.sh leviathan.cymric-daggertooth.ts.net:/tmp/luna-e9a58881-identity-preservation-gpt6luna/audit-original-native-records.sh`
3. Run it without invoking product code:
   `ssh leviathan.cymric-daggertooth.ts.net "bash '/tmp/luna-e9a58881-identity-preservation-gpt6luna/audit-original-native-records.sh' > '/tmp/luna-e9a58881-identity-preservation-gpt6luna/audit.log'"`
4. Collect output as needed:
   `scp leviathan.cymric-daggertooth.ts.net:/tmp/luna-e9a58881-identity-preservation-gpt6luna/audit.log .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/identity-review/audit.log`
   `scp leviathan.cymric-daggertooth.ts.net:/tmp/luna-e9a58881-identity-preservation-gpt6luna/original-record-rows.tsv .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/identity-review/original-record-rows.tsv`
   `scp leviathan.cymric-daggertooth.ts.net:/tmp/luna-e9a58881-identity-preservation-gpt6luna/augmented-record-rows.tsv .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/identity-review/augmented-record-rows.tsv`

An initial inspector draft failed Perl parsing before completing any comparison; the corrected retained script above completed successfully. No product/helper/generator/provider/recipe/controller code, build, test, or source archive was executed or inspected in this pass. Generated-program archives, per-file payload bytes, NARs, all seven role trees, selected helper/translator copies, recipe/lineage routing, and cold initial-authority absence remain outside this slice and must be checked against the parent's corrected subject; no result here closes those checks.
