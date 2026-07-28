# Typed source for the protected seed-to-coreutils-5.0 transition.
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
      "11a116c4d1a406e46c3e2f15f11f8e643abcd387fc6668e1680abe1d230bfe42"
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
    source "gzip-1.2.4-source" "gzip 1.2.4 fixed-output source"
      "de8516b8bfc78a91f5d37837ff70d6926f4b03cf68924ae7ef6ed602f7f316fd"
      "source bundle record fixed-url-28390b124a4e4a0426e370860163351efc23ff1d3d3b6be84a5609868b00f45d",
    source "gzip-makecrc-source" "bounded gzip CRC generator source"
      "48240f8bb743e9fd202f7bc2d98367607f853d2ed7483d484f50892bc69d9ef1"
      "bootstrap/seeds/gzip-makecrc.c",
    source "tar-1.12-source" "GNU tar 1.12 fixed-output source"
      "0cbc269e6eccf79f32b9f64204a0277a0e6df877dc1a6d0fd0d3e240365abdff"
      "source bundle record fixed-url-56372665c26229fd9923e24885bba78d12de4459e7fb19f3ee6d5cb21dd1f1ac",
    source "tar-getdate-stub-source" "bounded GNU tar date-parser stub source"
      "a479349008d81a7766d3e56dd5bd1d83e8645c0690228d4bb2d89d6f04d4f0a9"
      "bootstrap/seeds/tar-getdate-stub.c",
    source "sed-4.0.9-source" "GNU sed 4.0.9 fixed-output source"
      "65aa1a57a2966a247b4992de6525e319adb99f8ad2fe4cc2eb543a28740994ad"
      "source bundle record fixed-url-2861bfe2bbe8003326313432362cdd09a73a549a874abb706bce6ba4e5378011",
    source "bzip2-1.0.8-source" "bzip2 1.0.8 fixed-output source"
      "969e26e40644aaed6137a7403c29397ada624f6dc20d38dfae5fec9d2079292d"
      "source bundle record fixed-url-29051c18498ee931a89eb96a6c890e06b4098c8ec2e0f4242f0d8337192a8070",
    source "bzip2-utime-header-source" "bounded bzip2 compatibility header"
      "3bf37f6c622b15c4f6acada147c5ca5921da1cfefe9b0cb6dc53c5d10a10de13"
      "bootstrap/seeds/bzip2-utime.h",
    source "coreutils-5.0-source" "coreutils 5.0 fixed-output source"
      "82d3dd8a0cbb5b5032512d747b5bb79b5dfdfd94b2cd11d9ae50a1dd0b05d814"
      "source bundle record fixed-url-758798ffe110eb26e73f9f7832f4addd66c8b8722387fba4513bb786c8528d76",
    source "coreutils-5.0-config-header-source" "bounded coreutils configuration header"
      "03f96124348df7f962ee7c2e23ecfb97a7eeadcbd32a50e503850d5d0c6513b0"
      "bootstrap/seeds/coreutils-5.0-config.h",
    source "coreutils-modechange-patch-source" "live-bootstrap modechange compatibility patch"
      "b3be0a3a118ad92371d0a1b5d664c216d33964a22e18060184e9d10d65d0ddf8"
      "bootstrap/seeds/coreutils-5.0-modechange.patch",
    source "coreutils-mbstate-patch-source" "live-bootstrap mbstate compatibility patch"
      "d6a42d969bc7665e7b9193f15b0bb583c5d6fd1277ab2aa5a180c808aa70a877"
      "bootstrap/seeds/coreutils-5.0-mbstate.patch",
    source "coreutils-ls-strcmp-patch-source" "live-bootstrap ls compatibility patch"
      "80c602c5e65288de9bf4153acc139b71477d89572ef6dc100bc39d40aa93cbd8"
      "bootstrap/seeds/coreutils-5.0-ls-strcmp.patch",
    source "coreutils-touch-getdate-patch-source" "live-bootstrap touch compatibility patch"
      "ea379c4de61b38f39aebb7cac135e0c462814065f7849b9a5bb819154d0ee7f5"
      "bootstrap/seeds/coreutils-5.0-touch-getdate.patch",
    source "coreutils-tac-uint64-patch-source" "live-bootstrap TinyCC integer compatibility patch"
      "e9441c3a23908b695aa7084b50a05065ed7daf852d573645ba82499bbdd6c995"
      "bootstrap/seeds/coreutils-5.0-tac-uint64.patch",
    source "coreutils-expr-strcmp-patch-source" "live-bootstrap expr compatibility patch"
      "0b808546ae32923f754a12dc0c7578aa68624400159a5c97a93cd49a6252f1a6"
      "bootstrap/seeds/coreutils-5.0-expr-strcmp.patch",
    source "coreutils-sort-locale-patch-source" "live-bootstrap sort compatibility patch"
      "69792c09234934fd70f1795affca68cdae6c622a7532898d55794572597ae7c7"
      "bootstrap/seeds/coreutils-5.0-sort-locale.patch",
    source "coreutils-hash-tcc-patch-source" "bounded TinyCC hash-table compatibility patch"
      "407c0d0457158c2ceb0a165babfeb5e2f574c3f3bea44b774eb336203a2d6daf"
      "bootstrap/seeds/coreutils-5.0-hash-tcc.patch",
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
    tinycc_generated "gzip-generator-build-exec" "gzip-makecrc" "source-built gzip CRC generator" "dd10648de513f76a1ecdbfa23cc88953f8c6e46848311a6372de7498aed43c6e",
    tinycc_generated "gzip-generator-run-exec" "gzip-crc-table" "generated gzip CRC table" "19a3f07c4b071fe898927d1da2c3b64d37e02b0e8cf309d2a9fc4d70ac9d8fa1",
    tinycc_generated "gzip-build-exec" "gzip-1.2.4" "source-built gzip 1.2.4" "d2939461715a694222c1a612c0908754e599eb2d268fe012c525845367eff04a",
    tinycc_generated "gzip-build-exec" "gunzip-1.2.4" "source-built gunzip 1.2.4 alias" "d2939461715a694222c1a612c0908754e599eb2d268fe012c525845367eff04a",
    tinycc_generated "gzip-smoke-exec" "gzip-smoke" "gzip protected roundtrip smoke output" "78abaccbae9efa7039ff82d80215d2fbd3c4cd0221dadd85dd9df731f15d8751",
    tinycc_generated "tar-build-exec" "tar-1.12" "source-built GNU tar 1.12" "57c861b961bfc2c1047ab7f175c0f16024ea91787ba5a5e43600c721b5eff6ca",
    tinycc_generated "tar-smoke-exec" "tar-smoke" "GNU tar protected roundtrip smoke output" "59392bf4bef63f5688ff8604868a22adcecfe94f75c1234fe7c9df1107038be2",
    tinycc_generated "sed-build-exec" "sed-4.0.9" "source-built GNU sed 4.0.9" "cc4bc898fbad65d9c0a07eab85c61c99abdb9d7d7d747948067bea7d9954618a",
    tinycc_generated "sed-smoke-exec" "sed-smoke" "GNU sed protected substitution smoke output" "91f02abc6d98e0f00da4f3b1249ae0acd29544f7b19304f190d29f523885c077",
    tinycc_generated "bzip2-rewrite-exec" "bzip2-configured-source" "protected bzip2 source rewrite identity" "a3e8d977c0652889d2714f7eb9d1cfbe48581c05c7b7e3b79405334504fedd73",
    tinycc_generated "bzip2-build-exec" "bzip2-1.0.8" "source-built bzip2 1.0.8" "b6911b1a367e8b159264078e7976d363e3cb9d268a1dd074ddca51a76916b8a7",
    tinycc_generated "bzip2-build-exec" "bzip2recover-1.0.8" "source-built bzip2recover 1.0.8" "f4bf3e19bd09bcd64503def65eb7bd7eaa5ea43af94536b8d6e638a48501e0a0",
    tinycc_generated "bzip2-smoke-exec" "bzip2-smoke" "bzip2 protected compression roundtrip output" "b37fa30b2c16ae67d4e023b19fbecf3b10dab989e5fdedec510c29780cc5793b",
    tinycc_generated "coreutils-patch-exec" "coreutils-configured-source" "protected coreutils configured source identity" "232ccf0c23c2e8a028c5ca42cbb46f3afa1aad664a5d5187e11fecdcb04334d4",
    tinycc_generated "coreutils-build-exec" "coreutils-cat" "source-built coreutils cat" "3d9de9e7f81612a74e75312f22d038b665e69c5d9590662c293843e9711cf020",
    tinycc_generated "coreutils-build-exec" "coreutils-chmod" "source-built coreutils chmod" "8fe8251243646c8e44ffe50e827cb27f2197eed344c498b31c365c7d27e07779",
    tinycc_generated "coreutils-build-exec" "coreutils-cp" "source-built coreutils cp" "8c11ead9af84f2371178ce892afbac6daa416c445f2575a2300b2b5ffadf17f3",
    tinycc_generated "coreutils-build-exec" "coreutils-echo" "source-built coreutils echo" "3ce3cfec07fda35f97485b30fddd8a14f4cb04bd8427d2aa3acc4e94fd650a3f",
    tinycc_generated "coreutils-build-exec" "coreutils-install" "source-built coreutils install" "7f332bcf208b458595c202f4b1f2faf3fffadd80f7cd34ff67c711ee6183b210",
    tinycc_generated "coreutils-build-exec" "coreutils-ln" "source-built coreutils ln" "8146091298782331503b99b3f0bf02b0a92de93a5fe7b57fba08b5d870087c33",
    tinycc_generated "coreutils-build-exec" "coreutils-ls" "source-built coreutils ls" "32e2fcd7893befd1096a15b0a44992dcfd522044aed0601df96f434b39a60683",
    tinycc_generated "coreutils-build-exec" "coreutils-mkdir" "source-built coreutils mkdir" "b94fc26390ef29d55b71e608b800fa0451ff477b33c15ff3d2be3c8a2a06621b",
    tinycc_generated "coreutils-build-exec" "coreutils-mv" "source-built coreutils mv" "d765750db422dcaf98b1203911d1bbd0569a1cbe5a2e7006c2e26a476e420886",
    tinycc_generated "coreutils-build-exec" "coreutils-rm" "source-built coreutils rm" "f1bf6e9330c0b9e1a88b85185f5834a38b2b89526b4ca95bbb095930e57714da",
    tinycc_generated "coreutils-build-exec" "coreutils-rmdir" "source-built coreutils rmdir" "0a8235d566edcd18d68ed94df5ed2449e0fa9a4d37394367cb832536fa554dbc",
    tinycc_generated "coreutils-build-exec" "coreutils-sort" "source-built coreutils sort" "8b861f615fa0f7e868107b5c788562ba1d4dc2fff2ed9e9bca0f7d7df6c455bf",
    tinycc_generated "coreutils-build-exec" "coreutils-test" "source-built coreutils test" "87baf4f61820a2861bbb1a08b3bd9b4c7a5b8baed66ddb0e152d7a981ae27669",
    tinycc_generated "coreutils-build-exec" "coreutils-true" "source-built coreutils true" "deb8e7848bca7728c6594600c9d5a1b40d64beb6729fec2e71785236af2cc1e9",
    tinycc_generated "coreutils-build-exec" "coreutils-false" "source-built coreutils false" "b476958f8326512fe4fa9563cb97bf79a658900b646f64c8f9f84426ae099ac9",
    tinycc_generated "coreutils-build-exec" "coreutils-head" "source-built coreutils head" "71a84fad92471f61509ba9b781414934bb3b0975fd1635cef51709f952bf682b",
    tinycc_generated "coreutils-build-exec" "coreutils-tail" "source-built coreutils tail" "d0840c1a769287b4c1acf913dd8c4cc396294158dd57e9cd5bcf3e7cdca06962",
    tinycc_generated "coreutils-build-exec" "coreutils-wc" "source-built coreutils wc" "bc5ae003d38d9c4ca64568d97fe444d23e62258c93a936015e242806fb028f49",
    tinycc_generated "coreutils-build-exec" "coreutils-basename" "source-built coreutils basename" "e6c5b456a4f2279400f46ff272734dfb68829281c9e702c9fcc1e1c852eb1074",
    tinycc_generated "coreutils-build-exec" "coreutils-dirname" "source-built coreutils dirname" "b21c5d4963d0d336150374699a4398c920fb2b323c265be463baf5264dae940e",
    tinycc_generated "coreutils-build-exec" "coreutils-tr" "source-built coreutils tr" "8c7c183c9af7a4fddcf526b8f0318ffb847faea0be75967de1bb72e0d51b4ea0",
    tinycc_generated "coreutils-build-exec" "coreutils-uniq" "source-built coreutils uniq" "fadb7ca3f75ac06d73d01e8e304c86f06364a91f6001ba13717e69eb1dcbc328",
    tinycc_generated "coreutils-build-exec" "coreutils-expr" "source-built coreutils expr" "75770b1702a18bec30618d11cc55eb07eafc5f1349ca0d1d0567e24534b437f2",
    tinycc_generated "coreutils-build-exec" "coreutils-tee" "source-built coreutils tee" "d5ec19db795aef5f981b4f99e310a4e977c0543dffbf192cc4539bcb0fff3975",
    tinycc_generated "coreutils-build-exec" "coreutils-touch" "source-built coreutils touch" "a2b77b46e85c94390f459f158ebb33c529bbc5eb730c7719f24632f1fd6b6a4c",
    tinycc_generated "coreutils-build-exec" "coreutils-bracket" "source-built coreutils bracket alias" "87baf4f61820a2861bbb1a08b3bd9b4c7a5b8baed66ddb0e152d7a981ae27669",
    tinycc_generated "coreutils-smoke-exec" "coreutils-smoke" "coreutils protected filesystem smoke output" "fd65316f83e13de60a964675b6ab49ee8bd0e40db3aef391c8c1cbee240375d8",
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
    {
      id = "gzip-generator-build-exec",
      name = "TinyCC 0.9.27 materializing the bounded gzip CRC generator",
      source_artifact_ids = ["gzip-makecrc-source"],
      input_artifact_ids = ["tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1", "gnu-patch-smoke"],
      output_artifact_ids = ["gzip-makecrc"],
      digest = blake3 "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39",
    },
    {
      id = "gzip-generator-run-exec",
      name = "source-built gzip CRC generator materializing the checked table",
      source_artifact_ids = [],
      input_artifact_ids = ["gzip-makecrc"],
      output_artifact_ids = ["gzip-crc-table"],
      digest = blake3 "dd10648de513f76a1ecdbfa23cc88953f8c6e46848311a6372de7498aed43c6e",
    },
    {
      id = "gzip-build-exec",
      name = "TinyCC 0.9.27 materializing gzip and gunzip 1.2.4",
      source_artifact_ids = ["gzip-1.2.4-source"],
      input_artifact_ids = ["tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1", "gzip-crc-table"],
      output_artifact_ids = ["gzip-1.2.4", "gunzip-1.2.4"],
      digest = blake3 "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39",
    },
    {
      id = "gzip-smoke-exec",
      name = "gzip and gunzip executing a roundtrip and rejecting malformed input",
      source_artifact_ids = [],
      input_artifact_ids = ["gzip-1.2.4", "gunzip-1.2.4"],
      output_artifact_ids = ["gzip-smoke"],
      digest = blake3 "d2939461715a694222c1a612c0908754e599eb2d268fe012c525845367eff04a",
    },
    {
      id = "tar-build-exec",
      name = "TinyCC 0.9.27 materializing GNU tar 1.12 after gzip",
      source_artifact_ids = ["tar-1.12-source", "tar-getdate-stub-source"],
      input_artifact_ids = ["tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1", "gzip-smoke"],
      output_artifact_ids = ["tar-1.12"],
      digest = blake3 "51a5345bd89dbdb0537340f90151c663d144f34343a60c56c0110cd5a15dee39",
    },
    {
      id = "tar-smoke-exec",
      name = "GNU tar 1.12 executing an archive roundtrip and rejecting malformed input",
      source_artifact_ids = [],
      input_artifact_ids = ["tar-1.12"],
      output_artifact_ids = ["tar-smoke"],
      digest = blake3 "57c861b961bfc2c1047ab7f175c0f16024ea91787ba5a5e43600c721b5eff6ca",
    },
    {
      id = "sed-build-exec",
      name = "TinyCC 0.9.27 materializing GNU sed 4.0.9 after GNU tar",
      source_artifact_ids = ["sed-4.0.9-source"],
      input_artifact_ids = ["tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1", "tar-smoke"],
      output_artifact_ids = ["sed-4.0.9"],
      digest = blake3 "3e96d4952cf6c52707082fc120d9a372def665c73bef7ec13609abbfc201b99c",
    },
    {
      id = "sed-smoke-exec",
      name = "GNU sed 4.0.9 executing a substitution and rejecting malformed input",
      source_artifact_ids = [],
      input_artifact_ids = ["sed-4.0.9"],
      output_artifact_ids = ["sed-smoke"],
      digest = blake3 "cc4bc898fbad65d9c0a07eab85c61c99abdb9d7d7d747948067bea7d9954618a",
    },
    {
      id = "bzip2-rewrite-exec",
      name = "GNU sed 4.0.9 applying bounded bzip2 compatibility rewrites",
      source_artifact_ids = ["bzip2-1.0.8-source", "bzip2-utime-header-source"],
      input_artifact_ids = ["sed-4.0.9", "sed-smoke"],
      output_artifact_ids = ["bzip2-configured-source"],
      digest = blake3 "a3e8d977c0652889d2714f7eb9d1cfbe48581c05c7b7e3b79405334504fedd73",
    },
    {
      id = "bzip2-build-exec",
      name = "TinyCC 0.9.27 materializing bzip2 1.0.8",
      source_artifact_ids = [],
      input_artifact_ids = ["bzip2-configured-source", "tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1"],
      output_artifact_ids = ["bzip2-1.0.8", "bzip2recover-1.0.8"],
      digest = blake3 "a3e8d977c0652889d2714f7eb9d1cfbe48581c05c7b7e3b79405334504fedd73",
    },
    {
      id = "bzip2-smoke-exec",
      name = "bzip2 1.0.8 executing a compression roundtrip and rejecting malformed input",
      source_artifact_ids = [],
      input_artifact_ids = ["bzip2-1.0.8"],
      output_artifact_ids = ["bzip2-smoke"],
      digest = blake3 "b6911b1a367e8b159264078e7976d363e3cb9d268a1dd074ddca51a76916b8a7",
    },
    {
      id = "coreutils-patch-exec",
      name = "GNU patch 2.5.9 applying bounded coreutils compatibility patches",
      source_artifact_ids = [
        "coreutils-5.0-source", "coreutils-5.0-config-header-source",
        "coreutils-modechange-patch-source", "coreutils-mbstate-patch-source",
        "coreutils-ls-strcmp-patch-source", "coreutils-touch-getdate-patch-source",
        "coreutils-tac-uint64-patch-source", "coreutils-expr-strcmp-patch-source",
        "coreutils-sort-locale-patch-source", "coreutils-hash-tcc-patch-source",
      ],
      input_artifact_ids = ["gnu-patch-2.5.9"],
      output_artifact_ids = ["coreutils-configured-source"],
      digest = blake3 "232ccf0c23c2e8a028c5ca42cbb46f3afa1aad664a5d5187e11fecdcb04334d4",
    },
    {
      id = "coreutils-build-exec",
      name = "TinyCC 0.9.27 materializing the bounded coreutils 5.0 utility set",
      source_artifact_ids = [],
      input_artifact_ids = ["coreutils-configured-source", "tinycc-0.9.27", "tinycc27-libc", "tinycc-crt1", "tinycc-libtcc1"],
      output_artifact_ids = [
        "coreutils-cat", "coreutils-chmod", "coreutils-cp", "coreutils-echo", "coreutils-install",
        "coreutils-ln", "coreutils-ls", "coreutils-mkdir", "coreutils-mv", "coreutils-rm",
        "coreutils-rmdir", "coreutils-sort", "coreutils-test", "coreutils-true", "coreutils-false",
        "coreutils-head", "coreutils-tail", "coreutils-wc", "coreutils-basename", "coreutils-dirname",
        "coreutils-tr", "coreutils-uniq", "coreutils-expr", "coreutils-tee", "coreutils-touch", "coreutils-bracket",
      ],
      digest = blake3 "232ccf0c23c2e8a028c5ca42cbb46f3afa1aad664a5d5187e11fecdcb04334d4",
    },
    {
      id = "coreutils-smoke-exec",
      name = "coreutils 5.0 executing bounded filesystem smokes and rejecting a missing copy source",
      source_artifact_ids = [],
      input_artifact_ids = ["coreutils-mkdir", "coreutils-echo", "coreutils-cp", "coreutils-cat", "coreutils-test"],
      output_artifact_ids = ["coreutils-smoke"],
      digest = blake3 "fd65316f83e13de60a964675b6ab49ee8bd0e40db3aef391c8c1cbee240375d8",
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
    { from_node_id = "seed", to_node_id = "gzip-1.2.4-source" },
    { from_node_id = "seed", to_node_id = "gzip-makecrc-source" },
    { from_node_id = "seed", to_node_id = "tar-1.12-source" },
    { from_node_id = "seed", to_node_id = "tar-getdate-stub-source" },
    { from_node_id = "seed", to_node_id = "sed-4.0.9-source" },
    { from_node_id = "seed", to_node_id = "bzip2-1.0.8-source" },
    { from_node_id = "seed", to_node_id = "bzip2-utime-header-source" },
    { from_node_id = "seed", to_node_id = "coreutils-5.0-source" },
    { from_node_id = "seed", to_node_id = "coreutils-5.0-config-header-source" },
    { from_node_id = "seed", to_node_id = "coreutils-modechange-patch-source" },
    { from_node_id = "seed", to_node_id = "coreutils-mbstate-patch-source" },
    { from_node_id = "seed", to_node_id = "coreutils-ls-strcmp-patch-source" },
    { from_node_id = "seed", to_node_id = "coreutils-touch-getdate-patch-source" },
    { from_node_id = "seed", to_node_id = "coreutils-tac-uint64-patch-source" },
    { from_node_id = "seed", to_node_id = "coreutils-expr-strcmp-patch-source" },
    { from_node_id = "seed", to_node_id = "coreutils-sort-locale-patch-source" },
    { from_node_id = "seed", to_node_id = "coreutils-hash-tcc-patch-source" },
  ],
} | Manifest
