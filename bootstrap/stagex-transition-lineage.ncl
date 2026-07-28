# Typed source for the protected seed-to-mini-Stage0 transition.
# This file does not describe Mes, TinyCC, or the normalized StageX provider.

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
  patches | Array Dyn,
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
      "7e93ccc7a29bacc1da6f75c10ae90655ee83afd0d95ca282c8d270c225372f45"
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
  ],
  patches = [],
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
  ],
} | Manifest
