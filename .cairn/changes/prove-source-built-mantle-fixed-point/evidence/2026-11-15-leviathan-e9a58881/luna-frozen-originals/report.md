# Luna frozen-source and original AWS-LC audit

## Bounded verdict

**PASS for frozen-source assembly and original-source identity only.** The actual frozen complete tree, source/original assembly metadata, original Git commit/tree, extracted capture, and frozen AWS-LC copy all match their supplied identities. The two AWS-LC trees are byte-for-byte equal to all 10,427 Git blobs, have exact tracked file modes and directory membership, and contain no symlinks, gitlinks, special files, missing entries, or extras. This is not a complete-cohort, prepared-profile, recipe, launch, fixed-point, compiler-correctness, reproducibility, or release verdict.

## Exact subject and measurements

Host: `leviathan.cymric-daggertooth.ts.net`.

Frozen complete source: `/home/brittonr/sbfp-20261113-berdKH/runner-source-e9a58881-typed-index`, revision `e9a58881b9ff491f47864632367c9b8ea4af76e3`.

- Normalized frozen complete tree BLAKE3: `e62509cd40ab6e4b202dce425a40483d74101210c6322c64f70b196ff7e177f7` (measured from the frozen root and the retained complete archive; both match).
- Code-only normalized tree BLAKE3: `1ea2223d9f992ed98f511fe4489cc415002e2fcb33718b0f7181331a0b6db1df` (measured from the frozen root excluding `./bootstrap/source-inputs`, which was created when the original source was added after the code-tree measurement).
- Compressed code archive BLAKE3: `494a677f912de7f25d9efadde55ba6bcc3052bf9193a80022332a1da810ee4a5`; decompressed archive BLAKE3: `fca622321dec330f848520f1fc6e9f89561869b4c4453ad7bb14f2f6a092f13f`.
- Complete source archive BLAKE3: `e62509cd40ab6e4b202dce425a40483d74101210c6322c64f70b196ff7e177f7`; compressed archive BLAKE3: `02cb82a404b09b3b85aec971aacf3437271f203157e9e7acd296aaeee1624819`.

Original AWS-LC capture: `/home/brittonr/sbfp-20261113-berdKH/aws-original-source-02561621.TGISLN`; captured source root `/home/brittonr/sbfp-20261113-berdKH/aws-original-source-02561621.TGISLN/original`.

- Git commit: `02561621ffa4cf17c0c4f70bc11a82df36b42ae9`; commit object type `commit`.
- Git tree: `969d75db5c0494de4a26bbb2dcbf6837058054c1` (both `git show` and `rev-parse <commit>^{tree}` agree).
- Captured source archive BLAKE3: `282f13d4bb302e497915301156539943fd3e1e45f6a60f36678ae13e40636a88`.
- Normalized captured original tree BLAKE3: `8f731ad6e1c7b40d9e046a788dd7a7a088686b098e02d83c89750e7965a17f89`; the frozen AWS-LC copy has the same normalized tree identity.

Both the captured original and frozen copy contain exactly 10,427 regular files in 583 directories, totalling 315,868,378 bytes. For each tree, all 10,427 blobs were streamed from Git `cat-file --batch`, compared byte-for-byte with the corresponding filesystem file, and independently checked against the Git blob object ID. Modes match exactly: 10,207 `100644` / filesystem `0644`, and 220 `100755` / filesystem `0755`. Both inventories have zero missing or extra files and directories, zero Git symlinks/gitlinks, zero filesystem symlinks, and zero special files. The retained JSON result records the per-root counts and empty mismatch/error lists.

## Frozen helper / translator / Rust-authority source identities

The frozen generator mapping was read but not run: `source_built_mantle_programs_graph.rs` assembles its metadata helper from the frozen metadata-helper template, `native::CC_PROCESS_SOURCE`, and `compression::NATIVE_REQUEST_HELPER_SOURCE`; `source_built_mantle_programs_metadata.rs` delegates to the template's `source` function. Hashes below bind those selected frozen source inputs and related Rust authority; they do **not** claim byte identity with a generated helper member. The full 22-file list, byte counts, modes and BLAKE3 values is retained in `luna-frozen-originals-result.json`.

| Frozen source file | Bytes | BLAKE3 |
|---|---:|---|
| `src/source_built_mantle_programs.rs` | 45,251 | `70973ba46431cbbda391a55c9a0724215e3005f25d2f1a1a9d4227d4b5f07921` |
| `src/source_built_mantle_programs_graph.rs` | 34,424 | `a07836d408247111138dc22066505a55728859e06a20322af1ccd5621bbcd730` |
| `src/source_built_mantle_programs_metadata.rs` | 33,717 | `4cfd54ee23c42c9a4cd872589a435b0e97219564b2cfcd7052bc577f8a3a74c7` |
| `src/source_built_mantle_programs_metadata_helper.rs` | 51,749 | `a3e2e1ae870459019462b11832f37827b489fe371856ee61d02b0a33728ce831` |
| `src/source_built_mantle_programs_native.rs` | 69,410 | `5cb0ebc1fa11afd7c28f8381a46791b4e20b57a10ab3b1ad4adfc9f734888403` |
| `src/source_built_mantle_programs_native_helper.rs` | 7,176 | `44f4bf3ddebf711cf51390879245bccaf63ed71e13f2a89f8fb3ca8f0a4cb550` |
| `src/source_built_mantle_programs_compression.rs` | 17,574 | `650cc9058345d84808cb18b936e75da781ea0c284133d8f88d66be7b9b781f4f` |
| `src/source_built_mantle_programs_native_request.rs` | 26,599 | `ea8e6aae0bd87fcbaf5428d07324fdc80bf0af9a9ce97eeb6962ff3681fb886c` |
| `src/source_built_mantle_programs_aws.rs` | 24,302 | `819c04cafb218205a6f56aabd5c6672e5e6637a6372841b7a3a722c0d860c904` |
| `src/source_built_mantle_programs_aws_helper.rs` | 12,042 | `79b30f32724f2d7528b787ace20da86be7760db319987e3c8e4e5792c1c9e1ff` |
| `src/source_built_fixed_point.rs` | 46,688 | `9401d0a197dd5cafd411ed9f8ac026397ef460c00a080877156e443c96cde122` |
| `src/source_built_fixed_point_planning.rs` | 5,841 | `91be8df65a37d1344c194062f3d09043567479a4efc295f86d09144ba43181bd` |
| `src/source_built_fixed_point_stagex_actions.rs` | 51,795 | `32937f01d8199b5a83c0a2af692c1755324de87e65d176a801fe1286346d29fd` |
| `src/source_built_fixed_point_rust_actions.rs` | 15,755 | `03347c19dea92ab80d08b0f65df54ec9cb020443b5dda5f62bb6a9dcbc9bf55f` |
| `src/source_built_fixed_point_mantle_actions.rs` | 48,772 | `d55b45a11b922b6dceb428879b8723fa8c27503f3311b29418841d0bfcfcf1b2` |
| `src/source_built_fixed_point_native_actions.rs` | 24,424 | `027be7f263cb328aaea5340cdc3be639bf8d9499ba3382897180b9212cb8caef` |
| `src/source_built_fixed_point_shell.rs` | 189,978 | `e7343037d6f1dae709f0a15cb1601af8eccf5a9213a0d26ff76288de7ebf4fcb` |
| `src/source_built_fixed_point_authoring.rs` | 28,213 | `2755d9ab07792600ce5491c4d134b33339c3119f67b71148558262157d6817c8` |
| `src/source_built_fixed_point_action_trust.rs` | 93,151 | `af9a141259548a73baf13bd60487412c01ee0c871ae572f1daac2ed97f681754` |
| `src/source_built_fixed_point_action_controller.rs` | 36,587 | `aafd4668ae21b831b33f726121fb9894b5da517e4f29353fee79e02eb8a12768` |
| `src/source_built_program_authoring.rs` | 6,244 | `23c52720c463872d698140fd2043fd938125692991e2b57a3fccb089d3bf7c9b` |
| `src/source_built_recipe.rs` | 165,340 | `28ca15f8081b88c5747a851a1474d92277fba142f0b7fdfa1f74d8a48374d009` |

## Reproduction and retained evidence

Assigned evidence directory:
`.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/luna-frozen-originals/`

The retained inspector is `audit-frozen-originals.py`; it reads only the frozen source, captured original, original Git object database, and assembly/capture metadata. It writes only to the distinct inert remote scratch directory below. Exact setup/run/collection commands:

```sh
ssh leviathan.cymric-daggertooth.ts.net 'test ! -e /tmp/luna-frozen-originals-e9a58881-gpt6luna-20261115 && mkdir -m 700 /tmp/luna-frozen-originals-e9a58881-gpt6luna-20261115'
scp .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/luna-frozen-originals/audit-frozen-originals.py leviathan.cymric-daggertooth.ts.net:/tmp/luna-frozen-originals-e9a58881-gpt6luna-20261115/audit-frozen-originals.py
ssh leviathan.cymric-daggertooth.ts.net "'/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3' '/tmp/luna-frozen-originals-e9a58881-gpt6luna-20261115/audit-frozen-originals.py' > '/tmp/luna-frozen-originals-e9a58881-gpt6luna-20261115/audit.stdout.log'"
scp leviathan.cymric-daggertooth.ts.net:/tmp/luna-frozen-originals-e9a58881-gpt6luna-20261115/luna-frozen-originals-result.json .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/luna-frozen-originals/luna-frozen-originals-result.json
scp leviathan.cymric-daggertooth.ts.net:/tmp/luna-frozen-originals-e9a58881-gpt6luna-20261115/audit.stdout.log .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/luna-frozen-originals/audit.stdout.log
```

The independent full-tree hash command was:

```sh
ssh leviathan.cymric-daggertooth.ts.net 'R=/home/brittonr/sbfp-20261113-berdKH/runner-source-e9a58881-typed-index; /run/current-system/sw/bin/tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$R" . | /nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum --no-names'
```

The code-only tree command adds `--exclude=./bootstrap/source-inputs` before `--create`; the original tree command uses the same normalized-tar flags with `--directory /home/brittonr/sbfp-20261113-berdKH/aws-original-source-02561621.TGISLN/original`. `audit-frozen-originals.py` records the exact Git blob streaming/comparison, mode/member checks, archive hashes, metadata checks, tree hashes, and selected frozen-source file hashes.

Remote and locally collected artifact identities agree: inspector BLAKE3 `52efefc540bcedbc4b8a3c4f74ed8ce7a00a5f6e9b51b9a46bd390318f3db635`; result JSON BLAKE3 `84d7ce8d11c7509e5e3e37b0d29bf599d61ca982a2bfbcfee955d911065802d2`; stdout log BLAKE3 `84d7ce8d11c7509e5e3e37b0d29bf599d61ca982a2bfbcfee955d911065802d2`. The JSON contains `errors: []` and all identity checks pass.

Resolved inspector-only issues: remote default `PATH` did not provide `python3`, so the run used the pinned Nix-store interpreter above. An initial SSH quoting attempt passed an empty tar directory and produced the empty-stream BLAKE3; it was replaced with the exact quoted command shown above. An initial code-subtree check excluded only the AWS-LC leaf and yielded `0871577eaeba9fddfeb3dea43bf1523bd90ba52e2dbf3bb024c47700140a87b3`; reading the assembly procedure showed the `bootstrap/source-inputs` parent was created after the code-only hash, and excluding that parent reproduced the metadata identity `1ea2223d9f992ed98f511fe4489cc415002e2fcb33718b0f7181331a0b6db1df`. These were inspection-command/scope corrections; the final source/copy checks report no mismatch.

## Explicit remaining complete-cohort work

This frozen-input slice did not inspect cohort data. After publication, the parent still needs to independently:

1. Bind the current cohort report, plan, input/output profiles and their cross-references; verify every one of the seven materialized role trees and distinguish preauthor role identities from augmented prepared directories.
2. Verify full original source-record body preservation and every current generated program record identity, canonical content, archive member inventory/bytes, recursive NAR and physical payload/chunk bytes. Hex-encoded lengths alone do not establish payload equality.
3. Compare all selected cohort helper/translator/Rust-authority copies against the frozen source identities above; review the current source selectors, all selected original AWS/Ring boundaries, all recipes/lineage/routes and their consumers.
4. Establish current cold-selector membership and absence of initial output/provider/store authority. Do not promote this source/capture identity pass into prepared-profile or launch acceptance.
