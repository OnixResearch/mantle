# Typed source for the protected seed-to-GNU-patch-2.5.9 transition.
# This file does not yet describe the later native toolchain or normalized StageX provider.

let Digest = {
  algorithm | String,
  hex_value | String,
} in
let Seed = {
  seed_class | String,
  instruction_set | String,
  entry_point | String,
  io_contract | String,
  host_interface_surface | String,
  human_readable_source | String,
  reproduction_transcript | String,
  audit_note | String,
  audit_seed_max_bytes | Number,
  seed_bytes_len | Number,
  seed_digest | Digest,
} in
let SourceArtifact = {
  id | String,
  name | String,
  digest | Digest,
  provenance | String,
} in
let GeneratedArtifact = {
  id | String,
  name | String,
  producing_tool_id | String,
  digest | Digest,
} in
let TransitionTool = {
  id | String,
  name | String,
  source_artifact_ids | Array String,
  input_artifact_ids | Array String,
  output_artifact_ids | Array String,
  digest | Digest,
} in
let Patch = {
  id | String,
  name | String,
  target_artifact_id | String,
  digest | Digest,
  provenance | String,
} in
let ProviderOutput = {
  role | String,
  producing_artifact_id | String,
} in
let EnvironmentAssumption = {
  id | String,
  description | String,
  category | String,
} in
let StageTransition = {
  from_node_id | String,
  to_node_id | String,
} in
let Manifest = {
  seed | Seed,
  source_artifacts | Array SourceArtifact,
  generated_artifacts | Array GeneratedArtifact,
  transition_tools | Array TransitionTool,
  patches | Array Patch,
  provider_outputs | Array ProviderOutput,
  environment_assumptions | Array EnvironmentAssumption,
  stage_graph | Array StageTransition,
} in
let blake3 = fun value => {
  algorithm = "blake3",
  hex_value = value,
} in
let source = fun source_id source_name source_digest source_provenance => {
  id = source_id,
  name = source_name,
  digest = blake3 source_digest,
  provenance = source_provenance,
} in
let generated = fun artifact_id artifact_name artifact_digest => {
  id = artifact_id,
  name = artifact_name,
  producing_tool_id = "stage0-mini-kaem-exec",
  digest = blake3 artifact_digest,
} in
let full_generated = fun artifact_id artifact_name artifact_digest => {
  id = artifact_id,
  name = artifact_name,
  producing_tool_id = "stage0-full-kaem-exec",
  digest = blake3 artifact_digest,
} in
let mes_generated = fun artifact_id artifact_name artifact_digest => {
  id = artifact_id,
  name = artifact_name,
  producing_tool_id = "mes-m2-kaem-exec",
  digest = blake3 artifact_digest,
} in
let mes_runtime_generated = fun artifact_id artifact_name artifact_digest => {
  id = artifact_id,
  name = artifact_name,
  producing_tool_id = "mes-runtime-exec",
  digest = blake3 artifact_digest,
} in
let tinycc_generated = fun producer_id artifact_id artifact_name artifact_digest => {
  id = artifact_id,
  name = artifact_name,
  producing_tool_id = producer_id,
  digest = blake3 artifact_digest,
} in
{
  seed = {
    seed_class = "hex0_seed",
    instruction_set = "x86_64",
    entry_point = "0x600078",
    io_contract = "argv[1] input path and argv[2] output path; output mode 0700",
    host_interface_surface = "Linux x86_64 open, read, write, and exit system calls",
    human_readable_source = "bootstrap/seeds/AMD64/hex0_AMD64.hex0",
    reproduction_transcript = "copy checked seed, set mode 0700, execute seed with hex0 source and create-new output",
    audit_note = "The 229-byte checked seed matches the annotated hex source byte for byte.",
    audit_seed_max_bytes = 4096,
    seed_bytes_len = 229,
    seed_digest = blake3 "cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80",
  },
  source_artifacts = [
    source "hex0-source" "annotated AMD64 hex0 source"
      "0fb23576a10b41df29c165e39514f18c411da96ae71873a6a2e2f0b1d94de614"
      "bootstrap/seeds/AMD64/hex0_AMD64.hex0",
    source "kaem-minimal-source" "minimal AMD64 kaem source"
      "5a56b4164dca4d1e03ba35bc8ce4b418a0de2baf1cb9e6a0912607e26532d30d"
      "bootstrap/seeds/AMD64/kaem-minimal.hex0",
    source "kaem-smoke-script" "empty kaem smoke script"
      "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
      "generated as zero bytes by the bounded transition plan",
    source "stage0-source-bundle" "authenticated offline full-source closure"
      "7ec4786f81326dc6633479ff78ed53cfd18eb5086ae7374113fdef4921f78751"
      "mantle-source-bundle-v1 manifest identity loaded and validated before Stage0 execution",
    source "stage0-posix-amd64-source" "stage0-posix AMD64 snapshot"
      "f983a7b79acdd53fd4d31c8959c62ca24eabbea491327aaa113c4bc11549e5f9"
      "Git revision 82efa0d6be1c9bb993a7a62af1cccd8d2cda91f6",
    source "bootstrap-seeds-source" "bootstrap-seeds snapshot"
      "30d135a3e1c83495b521ae969d2c0e97da44baff7e143a16bcab0919fb0a36d2"
      "Git revision cedec6b8066d1db229b6c77d42d120a23c6980ed",
    source "m2-planet-source" "M2-Planet snapshot"
      "a4b081c01854e735697908471cd6170e270f5084170a64e51c5a20302c842d89"
      "Git revision bd2fe4b0659fd0ad3f476a5ad0ef801bd134665d",
    source "m2libc-source" "M2libc snapshot"
      "13450e1e9a31edfbeccd0adc30c52b2d085d4c3c6b58f42a1b6f902e903f3b4a"
      "Git revision 68a23cfd05d5a355ba7a30c770d684cbe86fcc4e",
    source "mescc-tools-source" "mescc-tools snapshot"
      "2dcf2577f60e195efa112a8d8978b41542a13f25ef0ee3d9ab4068256096ce94"
      "Git revision 5adfbf3364261a77109878a56b100aeeb6ef9ac4",
    source "mescc-tools-extra-source" "mescc-tools-extra snapshot"
      "0cc000667e857118296d9abd7959a323024eed0fc09231d307985e9ffe7a6ae6"
      "Git revision a151c245e512076971a3c85bb1502cf92cfa83b6",
    source "m2-mesoplanet-source" "M2-Mesoplanet snapshot"
      "f2c5526d58cc9424f32e78f6d96e9c271237998a7c8ac007d80fdd118d50fa47"
      "Git revision 4b011a85da73a7c97212468d41f17e806ba99547",
    source "stage0-amd64-answers" "Stage0 AMD64 SHA-256 interoperability answers"
      "0ddbb5dad8868a17ddcb7e36bcd981e822f8f374c057ad7d40b5ea5fdf160765"
      "bootstrap/stage0-amd64.answers",
    source "stage0-after-script" "empty Stage0 after hook"
      "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
      "generated as zero bytes by the bounded transition plan",
    source "mes-0.27.1-source" "GNU Mes 0.27.1 fixed-output source"
      "dd0e49258701c541d8fc097a899e4d00ae987d49951346ee22bd5743d5e81582"
      "source bundle record fixed-url-5fda7b5bed4dbf8d8da85b3dd19ee873cae96c563d0b33d7d796c645aaf2e14d",
    source "nyacc-1.00.2-source" "NYACC 1.00.2 fixed-output source"
      "3cdb842d8699276ce030e5fbbab455193d12ab41cfb36aabfc3f3bbdfa4c47db"
      "source bundle record fixed-url-cc3d97f8a1c6486f4393ae3002300713a08781fc4b8ec843a9bd2fe73ba082bc",
    source "tinycc-0.9.26-source" "TinyCC 0.9.26 fixed-output source"
      "db8c11387fd0db5783b4d2349b578ba1edfc739bf13d0865ab8b6b5f83a0711b"
      "source bundle record fixed-url-71495e1ddb39612a9f8197f340bf7254942f843ef7ff361eca64dae0b0a7bd2f",
    source "tinycc-mes-0.27.1-source" "fresh GNU Mes 0.27.1 source for TinyCC handoff"
      "438536d1095c3cd1f19b2645572dbe4ec265353513076db6ba5c395e23e6841a"
      "source bundle record fixed-url-6b4961e6c9fff76f8cf2bab720817eff96172d622164c4b23580203514d4b7b2",
    source "tinycc-0.9.27-source" "TinyCC 0.9.27 fixed-output source"
      "a3417d7e6218de60bfb3b30cab2db9fbe3e65891d9b3e765f09a4ac87539d03d"
      "source bundle record fixed-url-767003cf551d3f2e8409b6666cc3e000e1377f298a55fbf20daea3867bd3aed4",
    source "make-3.82-source" "GNU Make 3.82 fixed-output source"
      "b768ff74b8f16f6c41fb869833582c7cf614e98a8728a1c74616a6b6157ece77"
      "source bundle record fixed-url-ae11d5ec6f5d6b01fdeeb081181a8a51ad8208ab2d8b651cd2d8383f8fcda3f0",
    source "make-recipe-runner-source" "bounded GNU Make recipe runner source"
      "2d6a9757ecb8ba18de695edadd2473f14cc8c51427c8a922310c13f97221f6e8"
      "bootstrap/seeds/make-recipe-runner.c",
    source "gnu-patch-2.5.9-source" "GNU patch 2.5.9 fixed-output source"
      "d7dd692b74dea83bc6b6aed6efbd9f874aa991a1ba5722e8f4b06bf0c20a21e0"
      "source bundle record fixed-url-08fc9bbfa8cbf77e5bbfca6cffcfe7116d4927854bfc2b9352b912a00e017545",
  ],
  generated_artifacts = [
    {
      id = "hex0-reproduced",
      name = "reproduced hex0 assembler",
      producing_tool_id = "hex0-seed-exec",
      digest = blake3 "cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80",
    },
    {
      id = "kaem-0",
      name = "minimal kaem transition runner",
      producing_tool_id = "hex0-reproduced-exec",
      digest = blake3 "edc664d028b349824cc8c66030a8883b81bdc03d07bc871e44e60ecb4311ab2a",
    },
    generated "stage0-m0" "Stage0 M0" "080eca455a46f2cab998f0ff78b401077294ad612d829397309df7be23fa5a8e",
    generated "stage0-m1-bootstrap" "Stage0 bootstrap M1" "78be6fd6f49cee594969f3f82c8677a7b771e2907c023380c637628eff92c760",
    generated "stage0-m2" "Stage0 M2" "2d583c15ad3fa677883b769e7d8e91c7847ea30cff93013e1f0307b5451d70ed",
    generated "stage0-blood-elf-bootstrap" "Stage0 bootstrap blood-elf" "2f4f5d787f886552ef4600ab8b7e31c612e78b59a341e79ed2911558c9180eda",
    generated "stage0-catm" "Stage0 catm" "0d2c2c14eaf77c130f5ffaaea777e7b6f7eab1408acf57c8b516b5f56c451899",
    generated "stage0-cc-amd64" "Stage0 cc_amd64" "e91376dee1ac7aa4ef6dc5b86c3e73f08b71fc78b5e069f5e80035955eb971e4",
    generated "stage0-hex1" "Stage0 hex1" "580b8b771a6dc2b91feb34775ea7152623cc096b37013382f166f2fa9c6d1e40",
    generated "stage0-hex2-bootstrap" "Stage0 bootstrap hex2" "84c05689d80bacb397a41eb22c02b3e8e827c648b3b1b5f941d4545decbae0b3",
    generated "stage0-hex2-intermediate" "Stage0 intermediate hex2" "2954bef3c2eeae38f0de55556d920c51491f9dd4e74ed482fecbc43c83ef3c1a",
    generated "stage0-m1" "Stage0 M1" "b58f328ad2106ed666e68c485b1e1504ef893e09aed44fbe9805f2a534543642",
    generated "stage0-hex2" "Stage0 final hex2" "2954bef3c2eeae38f0de55556d920c51491f9dd4e74ed482fecbc43c83ef3c1a",
    generated "stage0-kaem" "Stage0 full kaem" "c6c2f9136a41ec16b049e2af0a017d87804d69f4cad882b20eefbbc6725ea854",
    full_generated "stage0-full-m2-mesoplanet" "full Stage0 M2-Mesoplanet" "e473c5670f6a2cfc0af270d75c72405930c24d741febbe4577d28f4cbf90cd36",
    full_generated "stage0-full-m2-planet" "full Stage0 M2-Planet" "9133f106b326b9badb5945fa50a7a5f36cffa2bdea76ad0568328cb06685f166",
    full_generated "stage0-full-blood-elf" "full Stage0 blood-elf" "23154d5fdfc6b669714a5551587feb34d4964759729dd28bc51581960fd19c25",
    full_generated "stage0-full-catm" "full Stage0 catm" "2ecde44089def1bb3496368f90d17fbba30d8938cf4109383375e3845ce52be3",
    full_generated "stage0-full-chmod" "full Stage0 chmod" "1ce14e51cd2020400ecaa356a8d67cb8fa6d53357a63e236c45901ee9759ac63",
    full_generated "stage0-full-cp" "full Stage0 cp" "636b9ee7cc548bfb59fd3425d9fd7cdfe9340766f2ea23e85c98f76f5276c23b",
    full_generated "stage0-full-get-machine" "full Stage0 get_machine" "94eb62461a1e94ca7fef4c5c90fa017505696dc22e3235df6b9cc341cf5234e3",
    full_generated "stage0-full-match" "full Stage0 match" "b6862c346146a63d7e334c3e057e54eb77ce841e788583f0d0a824879cff0c97",
    full_generated "stage0-full-mkdir" "full Stage0 mkdir" "a023e38258367002b7f9dde5fcb49cec445c34cce08a8b63c8933fa3fab5cf62",
    full_generated "stage0-full-replace" "full Stage0 replace" "4e67d5271e052629f8d92a11979ec8a93e7d8c416070544b71266d189f37dbbd",
    full_generated "stage0-full-rm" "full Stage0 rm" "e7a77376f5a41340b42929b679da8af84ff508ceda13b0f077e6f77e06423e20",
    full_generated "stage0-full-sha256sum" "full Stage0 sha256sum" "c507561a47ca67648247eaa06d054193d9aac21160708a7671987dc2f1c48a61",
    full_generated "stage0-full-unbz2" "full Stage0 unbz2" "51319fe1892cd3b43fee2751ef3d00ec4ae1ff2a70a6fa72c0764cf9d9e75010",
    full_generated "stage0-full-ungz" "full Stage0 ungz" "d7e6dae760f6635196ebd80f203aada32ad83eb8fb0d013b7b09f879dc45290d",
    full_generated "stage0-full-untar" "full Stage0 untar" "794461d61868aa601955182f5abc1fbc6f840a1402ece32855c41b59efb2bdf0",
    full_generated "stage0-full-unxz" "full Stage0 unxz" "8a1543cab9d6290e9c9306e16efc83ac0d230307c9c394b048e2d0b41b719771",
    full_generated "stage0-full-wrap" "full Stage0 wrap" "1d08486cfb920032a821c7603f56410af6c0372c0a4b78546d6c8d8bb1c0f9e0",
    mes_generated "mes-m2" "source-built mes-m2 Scheme runtime" "de4f20cdc2ad232ca79c5aee20c1ef491a71ce95fcfdb28020fc0c9a4c51ea3d",
    mes_generated "mes" "source-built Mes runtime compatibility executable" "de4f20cdc2ad232ca79c5aee20c1ef491a71ce95fcfdb28020fc0c9a4c51ea3d",
    mes_runtime_generated "mescc-entrypoint" "relocatable mescc Scheme entrypoint" "34ecfa676a692c79fe14fe85882dbd3b325c6f6f732e475ab94b9240f18cf445",
    mes_runtime_generated "mes-crt1" "Mes x86_64 crt1 object" "e49c87f324b58369183ab753fb1f346aeb28fc2974956d2872e046f8ef27416d",
    mes_runtime_generated "mes-x86_64-m1" "Mes x86_64 M1 definitions" "a1107fe39ca1c6486dc57bccf82ad40569fa13e34695e8f2c119fbb892b02892",
    mes_runtime_generated "mes-libc-mini" "Mes minimal libc archive" "12f9ee98f249da9a85158df6f242f5382c1e2d9a590dc84d521c70249d01dda1",
    mes_runtime_generated "mes-libmes" "Mes runtime archive" "2f7c7eb4d002097cf68ad3b84ffecbb107195563fd21f27672b9918c3cfad2bd",
    mes_runtime_generated "mes-libmescc" "Mes compiler support archive" "6a8d1e0a5f9dea02ae863552744e36f721833e57d054c993466d74c83eed7cdc",
    mes_runtime_generated "mes-libc" "Mes libc archive" "d70f11fe75e9eaa59b0aa4d0d92d8c663afe33a0e8f60e3864d364b45d0105c4",
    mes_runtime_generated "mes-libc-tcc" "Mes TinyCC support libc archive" "348f304db99daa3dc1eec807133cb232d28768b0d20a563bf8f6d68b493440d7",
    mes_runtime_generated "mes-libtcc1" "Mes TinyCC runtime archive" "c11be20d8d70e8ed3a88f1cea0a54ade5c64f8035c50974859dccb77f1bea055",
    mes_runtime_generated "mes-libgetopt" "Mes getopt archive" "04806a625d31f6a1ae4c92343e23ac4d355bcaa880f8aaa55b372f4a3785f135",
    mes_runtime_generated "mes-elf-header" "Mes x86_64 ELF header recipe" "892b042fe8e7d17b9774e89c83979c35f79f067fa2bdfd2bcc7279ac2858d8d4",
    mes_runtime_generated "mes-elf-footer" "Mes x86_64 ELF footer recipe" "b23189272376d99464531e041d1367b5a93086665513280645c1f41ce7ecda79",
    tinycc_generated "tinycc-mes-exec" "tcc-mes" "Mes-linked TinyCC compiler" "ef10b32c5de0b9322dfdcc5653bab9f0b9e8d5ee57b248ec8cb570c2ae10e9b3",
    tinycc_generated "tinycc-boot0-exec" "tcc-boot0" "first TinyCC self-compiled boundary" "8b3f20ef69f2356e64d513b82d26afb5c67bce9e51c045bbfbbf14784d13c66f",
    tinycc_generated "tinycc-final-exec" "tinycc-0.9.26" "final source-built TinyCC 0.9.26 compiler" "ef10b32c5de0b9322dfdcc5653bab9f0b9e8d5ee57b248ec8cb570c2ae10e9b3",
    tinycc_generated "tinycc-runtime-exec" "tinycc-crt1" "TinyCC-refreshed crt1" "d3fec15bd12fd72fdbdf4248d6df6b21a44cfb6a3e2c9d787a0d081ff7143df1",
    tinycc_generated "tinycc-runtime-exec" "tinycc-libc" "TinyCC-refreshed Mes libc" "cbc10117c34515b9b3ad14f3e4dc73a37708c6206f36cd51dc6f1e8a98093f8c",
    tinycc_generated "tinycc-runtime-exec" "tinycc-libtcc1" "TinyCC runtime support archive" "0e8b75458ad70ab03142b27a3014af68f57c03004f4d22a65702e60d531140e9",
    tinycc_generated "tinycc-runtime-exec" "tinycc-libgetopt" "TinyCC-refreshed getopt archive" "1fb5e11a952e173ec451601f2f11c09b2703d48e20b6533d327dc83b6b520449",
    tinycc_generated "tinycc27-runtime-exec" "tinycc27-va-list" "TinyCC 0.9.27 x86_64 va_list object" "0374e6d6c06eb731b2189d02283736cb93a0caa66ef4022737f2617ae037c7e8",
    tinycc_generated "tinycc27-runtime-exec" "tinycc27-libc" "TinyCC 0.9.27 Mes libc archive" "bd46a4a350e17ec4a7870be6c6dea82f7327c513ee4e2720bebb7b6e26f20d85",
    tinycc_generated "tinycc27-build-exec" "tinycc-0.9.27" "source-built TinyCC 0.9.27 compiler" "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39",
    tinycc_generated "tinycc27-build-exec" "tinycc27-alias" "TinyCC 0.9.27 compiler alias" "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39",
    tinycc_generated "tinycc27-smoke-exec" "tinycc27-positive-object" "TinyCC 0.9.27 positive smoke object" "19efdabfedafc9bf94f3ddc49967773a2a5e5b5f28128c6f95ac4f36184fdb92",
    tinycc_generated "make-build-exec" "make-3.82" "source-built GNU Make 3.82" "8a5aa115eab6069a8f69cf1b2c63c69c478d64b271bc13fda7ea4e3531b841d1",
    tinycc_generated "make-recipe-runner-build-exec" "make-recipe-runner" "source-built bounded Make recipe runner" "028ececbb67a3394595027c9273a44ed2ba5b465c209767589b5a989032b93d1",
    tinycc_generated "make-smoke-exec" "make-recipe-smoke" "GNU Make protected recipe smoke output" "8dd632d8df367d6751cc2e2497e8d9c38e0c5a1e58b100d59e1c0fdac235e3a2",
    tinycc_generated "gnu-patch-build-exec" "gnu-patch-2.5.9" "source-built GNU patch 2.5.9" "2193ea653bfbc16acc2609a5c7ef6c0b03a7a65dc50e6525d23868598e25fbcb",
    tinycc_generated "gnu-patch-smoke-exec" "gnu-patch-smoke" "GNU patch protected positive smoke output" "79d1d8da0b625035cdbfc9d51841030861b9f4cf7c5abbe442a8d13efc352170",
  ],
  transition_tools = [
    {
      id = "hex0-seed-exec",
      name = "audited hex0 seed",
      source_artifact_ids = ["hex0-source"],
      input_artifact_ids = [],
      output_artifact_ids = ["hex0-reproduced"],
      digest = blake3 "cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80",
    },
    {
      id = "hex0-reproduced-exec",
      name = "reproduced hex0 assembler",
      source_artifact_ids = ["kaem-minimal-source"],
      input_artifact_ids = ["hex0-reproduced"],
      output_artifact_ids = ["kaem-0"],
      digest = blake3 "cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80",
    },
    {
      id = "stage0-mini-kaem-exec",
      name = "source-built kaem-0 executing the absolute-path mini Stage0 recipe",
      source_artifact_ids = [
        "stage0-source-bundle",
        "stage0-posix-amd64-source",
        "bootstrap-seeds-source",
        "m2-planet-source",
        "m2libc-source",
        "mescc-tools-source",
        "mescc-tools-extra-source",
        "m2-mesoplanet-source",
      ],
      input_artifact_ids = ["hex0-reproduced", "kaem-0"],
      output_artifact_ids = [
        "stage0-m0",
        "stage0-m1-bootstrap",
        "stage0-m2",
        "stage0-blood-elf-bootstrap",
        "stage0-catm",
        "stage0-cc-amd64",
        "stage0-hex1",
        "stage0-hex2-bootstrap",
        "stage0-hex2-intermediate",
        "stage0-m1",
        "stage0-hex2",
        "stage0-kaem",
      ],
      digest = blake3 "edc664d028b349824cc8c66030a8883b81bdc03d07bc871e44e60ecb4311ab2a",
    },
    {
      id = "stage0-full-kaem-exec",
      name = "source-built full kaem executing the absolute-path full Stage0 recipes",
      source_artifact_ids = ["stage0-amd64-answers", "stage0-after-script"],
      input_artifact_ids = [
        "stage0-m2",
        "stage0-blood-elf-bootstrap",
        "stage0-m1",
        "stage0-hex2",
        "stage0-kaem",
      ],
      output_artifact_ids = [
        "stage0-full-m2-mesoplanet",
        "stage0-full-m2-planet",
        "stage0-full-blood-elf",
        "stage0-full-catm",
        "stage0-full-chmod",
        "stage0-full-cp",
        "stage0-full-get-machine",
        "stage0-full-match",
        "stage0-full-mkdir",
        "stage0-full-replace",
        "stage0-full-rm",
        "stage0-full-sha256sum",
        "stage0-full-unbz2",
        "stage0-full-ungz",
        "stage0-full-untar",
        "stage0-full-unxz",
        "stage0-full-wrap",
      ],
      digest = blake3 "c6c2f9136a41ec16b049e2af0a017d87804d69f4cad882b20eefbbc6725ea854",
    },
    {
      id = "mes-m2-kaem-exec",
      name = "source-built full Stage0 tools executing the bounded mes-m2 recipe",
      source_artifact_ids = [
        "stage0-source-bundle",
        "mes-0.27.1-source",
        "nyacc-1.00.2-source",
      ],
      input_artifact_ids = [
        "stage0-kaem",
        "stage0-m1",
        "stage0-hex2",
        "stage0-full-m2-planet",
        "stage0-full-blood-elf",
        "stage0-full-mkdir",
        "stage0-full-cp",
      ],
      output_artifact_ids = ["mes-m2", "mes"],
      digest = blake3 "de4f20cdc2ad232ca79c5aee20c1ef491a71ce95fcfdb28020fc0c9a4c51ea3d",
    },
    {
      id = "mes-runtime-exec",
      name = "source-built mes-m2 and full Stage0 M1 materializing Mes modules and runtime archives",
      source_artifact_ids = ["mes-0.27.1-source", "nyacc-1.00.2-source"],
      input_artifact_ids = ["mes-m2", "stage0-m1", "stage0-full-blood-elf"],
      output_artifact_ids = [
        "mescc-entrypoint",
        "mes-crt1",
        "mes-x86_64-m1",
        "mes-libc-mini",
        "mes-libmes",
        "mes-libmescc",
        "mes-libc",
        "mes-libc-tcc",
        "mes-libtcc1",
        "mes-libgetopt",
        "mes-elf-header",
        "mes-elf-footer",
      ],
      digest = blake3 "27e64a1954a37d4ac5ec39c219fa9906237a0f6861ea10fbc3d528b3a514f549",
    },
    {
      id = "tinycc-mes-exec",
      name = "Mes and full Stage0 tools materializing the first TinyCC compiler",
      source_artifact_ids = ["tinycc-0.9.26-source", "tinycc-mes-0.27.1-source"],
      input_artifact_ids = ["mes-m2", "mescc-entrypoint", "mes-libc-tcc", "stage0-m1", "stage0-hex2"],
      output_artifact_ids = ["tcc-mes"],
      digest = blake3 "de4f20cdc2ad232ca79c5aee20c1ef491a71ce95fcfdb28020fc0c9a4c51ea3d",
    },
    {
      id = "tinycc-runtime-exec",
      name = "Mes-linked TinyCC refreshing its crt and runtime archives",
      source_artifact_ids = ["tinycc-0.9.26-source", "tinycc-mes-0.27.1-source"],
      input_artifact_ids = ["tcc-mes", "mes-crt1", "mes-libc-tcc"],
      output_artifact_ids = ["tinycc-crt1", "tinycc-libc", "tinycc-libtcc1", "tinycc-libgetopt"],
      digest = blake3 "ef10b32c5de0b9322dfdcc5653bab9f0b9e8d5ee57b248ec8cb570c2ae10e9b3",
    },
    {
      id = "tinycc-boot0-exec",
      name = "Mes-linked TinyCC compiling the first TinyCC self-boundary",
      source_artifact_ids = ["tinycc-0.9.26-source"],
      input_artifact_ids = ["tcc-mes", "tinycc-crt1", "tinycc-libc", "tinycc-libtcc1"],
      output_artifact_ids = ["tcc-boot0"],
      digest = blake3 "ef10b32c5de0b9322dfdcc5653bab9f0b9e8d5ee57b248ec8cb570c2ae10e9b3",
    },
    {
      id = "tinycc-final-exec",
      name = "source-built Mes tools materializing final TinyCC 0.9.26",
      source_artifact_ids = ["tinycc-0.9.26-source", "tinycc-mes-0.27.1-source"],
      input_artifact_ids = ["tcc-boot0", "tcc-mes", "mes-m2", "stage0-m1", "stage0-hex2"],
      output_artifact_ids = ["tinycc-0.9.26"],
      digest = blake3 "de4f20cdc2ad232ca79c5aee20c1ef491a71ce95fcfdb28020fc0c9a4c51ea3d",
    },
    {
      id = "tinycc27-runtime-exec",
      name = "TinyCC 0.9.26 refreshing the Mes runtime for the TinyCC 0.9.27 ABI",
      source_artifact_ids = ["tinycc-0.9.27-source", "tinycc-mes-0.27.1-source"],
      input_artifact_ids = ["tinycc-0.9.26", "tinycc-crt1", "tinycc-libtcc1"],
      output_artifact_ids = ["tinycc27-va-list", "tinycc27-libc"],
      digest = blake3 "ef10b32c5de0b9322dfdcc5653bab9f0b9e8d5ee57b248ec8cb570c2ae10e9b3",
    },
    {
      id = "tinycc27-build-exec",
      name = "TinyCC 0.9.26 materializing TinyCC 0.9.27",
      source_artifact_ids = ["tinycc-0.9.27-source"],
      input_artifact_ids = ["tinycc-0.9.26", "tinycc27-va-list", "tinycc27-libc", "tinycc-libtcc1"],
      output_artifact_ids = ["tinycc-0.9.27", "tinycc27-alias"],
      digest = blake3 "ef10b32c5de0b9322dfdcc5653bab9f0b9e8d5ee57b248ec8cb570c2ae10e9b3",
    },
    {
      id = "tinycc27-smoke-exec",
      name = "TinyCC 0.9.27 executing bounded positive and negative smoke checks",
      source_artifact_ids = [],
      input_artifact_ids = ["tinycc-0.9.27"],
      output_artifact_ids = ["tinycc27-positive-object"],
      digest = blake3 "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39",
    },
    {
      id = "make-recipe-runner-build-exec",
      name = "TinyCC 0.9.27 materializing the bounded Make recipe runner",
      source_artifact_ids = ["make-recipe-runner-source"],
      input_artifact_ids = ["tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1"],
      output_artifact_ids = ["make-recipe-runner"],
      digest = blake3 "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39",
    },
    {
      id = "make-build-exec",
      name = "TinyCC 0.9.27 materializing GNU Make 3.82",
      source_artifact_ids = ["make-3.82-source"],
      input_artifact_ids = ["tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1"],
      output_artifact_ids = ["make-3.82"],
      digest = blake3 "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39",
    },
    {
      id = "make-smoke-exec",
      name = "GNU Make 3.82 dispatching a protected bounded recipe and rejecting malformed input",
      source_artifact_ids = [],
      input_artifact_ids = ["make-3.82", "make-recipe-runner"],
      output_artifact_ids = ["make-recipe-smoke"],
      digest = blake3 "8a5aa115eab6069a8f69cf1b2c63c69c478d64b271bc13fda7ea4e3531b841d1",
    },
    {
      id = "gnu-patch-build-exec",
      name = "TinyCC 0.9.27 materializing GNU patch 2.5.9 after the Make handoff",
      source_artifact_ids = ["gnu-patch-2.5.9-source"],
      input_artifact_ids = ["tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1", "make-recipe-smoke"],
      output_artifact_ids = ["gnu-patch-2.5.9"],
      digest = blake3 "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39",
    },
    {
      id = "gnu-patch-smoke-exec",
      name = "GNU patch 2.5.9 applying a positive patch and rejecting malformed input",
      source_artifact_ids = [],
      input_artifact_ids = ["gnu-patch-2.5.9"],
      output_artifact_ids = ["gnu-patch-smoke"],
      digest = blake3 "2193ea653bfbc16acc2609a5c7ef6c0b03a7a65dc50e6525d23868598e25fbcb",
    },
  ],
  patches = [
    {
      id = "tinycc-0.9.27-stagex-patch",
      name = "bounded TinyCC 0.9.27 StageX compatibility patch",
      target_artifact_id = "tinycc-0.9.27-source",
      digest = blake3 "b05705f548352080041ae56e2cacdf2e0898aa7dd5c857888004cf808884fc7f",
      provenance = "bootstrap/patches/tcc-0.9.27-stagex.patch derived from the checked TinyCC 0.9.27 recipe",
    },
    {
      id = "make-3.82-stagex-patch",
      name = "bounded GNU Make 3.82 TinyCC compatibility patch",
      target_artifact_id = "make-3.82-source",
      digest = blake3 "51c890dd1f9a2ddf816c68ebfbd4506241e45e3993ca4f14151c0d8665f87e59",
      provenance = "bootstrap/patches/make-3.82-stagex.patch derived from the checked Make 3.82 recipe",
    },
  ],
  provider_outputs = [
    {
      role = "retained_tool_metadata",
      producing_artifact_id = "stage0-kaem",
    },
  ],
  environment_assumptions = [
    {
      id = "linux-kernel",
      description = "Linux x86_64 kernel with seccomp user notification and the seed syscall ABI",
      category = "kernel",
    },
    {
      id = "mantle-orchestrator",
      description = "The running Mantle process stages checked bytes and supervises exact executable decisions",
      category = "orchestrator",
    },
  ],
  stage_graph = [
    { from_node_id = "seed", to_node_id = "hex0-source" },
    { from_node_id = "seed", to_node_id = "kaem-minimal-source" },
    { from_node_id = "seed", to_node_id = "kaem-smoke-script" },
    { from_node_id = "seed", to_node_id = "stage0-source-bundle" },
    { from_node_id = "seed", to_node_id = "stage0-posix-amd64-source" },
    { from_node_id = "seed", to_node_id = "bootstrap-seeds-source" },
    { from_node_id = "seed", to_node_id = "m2-planet-source" },
    { from_node_id = "seed", to_node_id = "m2libc-source" },
    { from_node_id = "seed", to_node_id = "mescc-tools-source" },
    { from_node_id = "seed", to_node_id = "mescc-tools-extra-source" },
    { from_node_id = "seed", to_node_id = "m2-mesoplanet-source" },
    { from_node_id = "seed", to_node_id = "stage0-amd64-answers" },
    { from_node_id = "seed", to_node_id = "stage0-after-script" },
    { from_node_id = "seed", to_node_id = "mes-0.27.1-source" },
    { from_node_id = "seed", to_node_id = "nyacc-1.00.2-source" },
    { from_node_id = "seed", to_node_id = "tinycc-0.9.26-source" },
    { from_node_id = "seed", to_node_id = "tinycc-mes-0.27.1-source" },
    { from_node_id = "seed", to_node_id = "tinycc-0.9.27-source" },
    { from_node_id = "tinycc-0.9.27-source", to_node_id = "tinycc-0.9.27-stagex-patch" },
    { from_node_id = "seed", to_node_id = "make-3.82-source" },
    { from_node_id = "seed", to_node_id = "make-recipe-runner-source" },
    { from_node_id = "make-3.82-source", to_node_id = "make-3.82-stagex-patch" },
    { from_node_id = "seed", to_node_id = "gnu-patch-2.5.9-source" },
  ],
} | Manifest
