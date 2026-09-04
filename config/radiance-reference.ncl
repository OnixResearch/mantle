# Reviewable source, build, and proof profile for the optional Radiance fixture.
let Sha256 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in
let Blake3 = Sha256 in
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let SourceRole = std.contract.from_predicate (fun value =>
  value == "radiance" || value == "bootstrap-compiler" || value == "emulator"
) in
let NativeBuildRole = std.contract.from_predicate (fun value =>
  value == "bootstrap-compiler" || value == "emulator"
) in
let Source = {
  role | SourceRole,
  repository_url | NonEmpty,
  revision | Sha256,
  content_blake3 | Blake3,
  projection | std.contract.Equal ".",
  snapshot_profile | std.contract.Equal "canonical-tree-v1",
  license_spdx | std.contract.Equal "MIT",
  license_blake3 | Blake3,
} in
let NativeBuild = {
  role | NativeBuildRole,
  sources | Array NonEmpty,
  flags | Array NonEmpty,
  linker_flags | Array NonEmpty,
  output_name | NonEmpty,
} in
let NativeBuildSet = {
  bootstrap_compiler | NativeBuild,
  emulator | NativeBuild,
} in
let Profile = {
  schema | std.contract.Equal "mantle-radiance-reference-profile-v1",
  sources | Array Source,
  native_builds | NativeBuildSet,
  emulator_arguments | Array NonEmpty,
  compiler_arguments | Array NonEmpty,
  expected_fixed_point_blake3 | Blake3,
  expected_fixed_point_bytes | Number,
  non_claim | NonEmpty,
} in

let license_digest = "d0f870e9de0345cb08c6ecc3367d99c12a89756d23cfec7f8260aaf1dcc888b6" in
let linker_stack_bytes = 33554432 in
let emulator_memory_bytes = 385024 in
let emulator_data_bytes = 348160 in
let emulator_stack_bytes = 512 in
let fixed_point_bytes = 1285492 in

{
  schema = "mantle-radiance-reference-profile-v1",
  sources = [
    {
      role = "radiance",
      repository_url = "https://code.radiant.computer/radiance",
      revision = "0d8a2d4fe8d0ba488e22c8ed83df1e53a5d23d69489c5d9e7fa0646b1c29c444",
      content_blake3 = "135c2f75eea21199b4538f5f0096e3e329c6b8b70943017644f9cdef05573c66",
      projection = ".",
      snapshot_profile = "canonical-tree-v1",
      license_spdx = "MIT",
      license_blake3 = license_digest,
    },
    {
      role = "bootstrap-compiler",
      repository_url = "https://code.radiant.computer/radiance.s0",
      revision = "7834d3a9d44fb48ae3d3c06da992922f3e46b580b3d92df36372081b2fe475c3",
      content_blake3 = "8c49dadd15e6834dd3204a35f4be00cf1cc01c9672c235a78a0c909b97aa8f1b",
      projection = ".",
      snapshot_profile = "canonical-tree-v1",
      license_spdx = "MIT",
      license_blake3 = license_digest,
    },
    {
      role = "emulator",
      repository_url = "https://code.radiant.computer/emulator",
      revision = "92cdb0c5293447964be053214fac403b49193ac3ec07c902576346ebaa205535",
      content_blake3 = "a6e91a2a6edc0aa4d5ca3c870012fdfb4f14748fed2829c46f6466efab3a27ec",
      projection = ".",
      snapshot_profile = "canonical-tree-v1",
      license_spdx = "MIT",
      license_blake3 = license_digest,
    },
  ],
  native_builds = {
    bootstrap_compiler = {
      role = "bootstrap-compiler",
      sources = [
        "ast.c", "desugar.c", "gen.c", "io.c", "module.c", "options.c",
        "parser.c", "radiance.c", "ralloc.c", "resolver.c", "riscv.c",
        "scanner.c", "strings.c", "symtab.c", "gen/data.c", "gen/emit.c",
      ],
      flags = [
        "-fvisibility=hidden", "-std=c99", "-Os", "-Wall", "-Wextra",
        "-Wpedantic", "-Wformat=2", "-Wformat-security", "-Wnull-dereference",
        "-Wno-format-nonliteral", "-Wcast-align", "-Wunused", "-Wuninitialized",
        "-Wmissing-field-initializers", "-fno-common", "-fstack-protector-all",
        "-mcmodel=medium",
      ],
      linker_flags = ["-Wl,-z,stack-size=%{linker_stack_bytes}"],
      output_name = "radiance.s0",
    },
    emulator = {
      role = "emulator",
      sources = ["emulator.c", "jit.c", "riscv/debug.c", "io.c", "riscv.c"],
      flags = [
        "-fvisibility=hidden", "-std=c99", "-O3", "-Wall", "-Wextra",
        "-Wpedantic", "-Wformat=2", "-Wformat-security", "-Wnull-dereference",
        "-Wno-format-nonliteral", "-Wcast-align", "-Wunused", "-Wuninitialized",
        "-Wmissing-field-initializers", "-fno-common", "-fstack-protector-all",
        "-mcmodel=medium", "-flto",
      ],
      linker_flags = ["-Wl,-z,stack-size=%{linker_stack_bytes}"],
      output_name = "emulator",
    },
  },
  emulator_arguments = [
    "-memory-size=%{emulator_memory_bytes}", "-data-size=%{emulator_data_bytes}",
    "-stack-size=%{emulator_stack_bytes}",
    "-count-instructions", "-run",
  ],
  compiler_arguments = [
    "-pkg", "std", "@std.lib", "-pkg", "radiance",
    "-mod", "compiler/radiance.rad", "-entry", "radiance",
  ],
  expected_fixed_point_blake3 = "a06905539bd81c9581218ca648e98fb7a55c5a7e79847b6c840181a3e2605b07",
  expected_fixed_point_bytes = fixed_point_bytes,
  non_claim = "external reference convergence does not prove compiler correctness, seed trust, semantic equivalence, or universal reproducibility",
} | Profile
