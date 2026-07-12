# Typed planning-only profile fixture.
#
# Official source release identities are real. Compiler dependency/tool and
# kernel-object identities are fixture values and carry no execution claim.

let mantle = import "../../../lib/lib.ncl" in
let make_artifact = fun digest_value => {
  artifact_ref = "mantle://blake3/%{digest_value}",
  digest_blake3 = digest_value,
  size_bytes = 64,
} in
let make_tool = fun role_value digest_value => {
  role = role_value,
  version = "fixture-only",
  executable_or_library = make_artifact digest_value,
  configuration_blake3 = digest_value,
} in
let make_kernel_input = fun role_value digest_value => {
  role = role_value,
  artifact = make_artifact digest_value,
  architecture = 'x86-64,
  kernel_release = "6.12.0-onix-fixture",
  kernel_build_identity = "onix:blake3:kernel-build:71ab5fe37dd010e284119879478a025fa85e07895041e3ddd91627b76609ccbc",
} in

{
  schema = "mantle-kernelscript-experiment-v1",
  experiment_id = "fixture-only-kfunc-module",
  beta = true,
  enabled_by_default = false,
  source = {
    relative_path = "program.ks",
    digest_blake3 = "f801e1880c1e8b8a215f225589bbc2aff2fb9928c6f7da320284e3ac8f704ee8",
    size_bytes = 162,
  },
  compiler = {
    version = "0.1.2",
    source_revision = "0c80d4e4ac0029d34cbc9d65e76d78c075b64555",
    source_archive_url = "https://github.com/multikernel/kernelscript/releases/download/v0.1.2/kernelscript-0.1.2-source.tar.gz",
    source_archive_sha256 = "9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1",
    source_archive_blake3 = "439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57",
    dependency_lock = {
      format = "opam-lock",
      dependency_count = 4,
      dependencies = [
        { package = "cmdliner", version = "fixture-only", artifact = make_artifact "1212121212121212121212121212121212121212121212121212121212121212" },
        { package = "dune", version = "fixture-only", artifact = make_artifact "1313131313131313131313131313131313131313131313131313131313131313" },
        { package = "menhir", version = "fixture-only", artifact = make_artifact "1414141414141414141414141414141414141414141414141414141414141414" },
        { package = "menhirLib", version = "fixture-only", artifact = make_artifact "1515151515151515151515151515151515151515151515151515151515151515" },
      ],
      lock_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    },
    compiler_executable = make_artifact "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    closure_blake3 = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  },
  toolchain = [
    make_tool 'ocaml "1111111111111111111111111111111111111111111111111111111111111111",
    make_tool 'dune "2222222222222222222222222222222222222222222222222222222222222222",
    make_tool 'menhir "3333333333333333333333333333333333333333333333333333333333333333",
    make_tool 'clang "4444444444444444444444444444444444444444444444444444444444444444",
    make_tool 'c-compiler "5555555555555555555555555555555555555555555555555555555555555555",
    make_tool 'bpftool "6666666666666666666666666666666666666666666666666666666666666666",
    make_tool 'libbpf "7777777777777777777777777777777777777777777777777777777777777777",
    make_tool 'elf-library "8888888888888888888888888888888888888888888888888888888888888888",
    make_tool 'zlib "9999999999999999999999999999999999999999999999999999999999999999",
    make_tool 'kernel-build "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  ],
  target = {
    architecture = 'x86-64,
    kernel_release = "6.12.0-onix-fixture",
    kernel_build_identity = "onix:blake3:kernel-build:71ab5fe37dd010e284119879478a025fa85e07895041e3ddd91627b76609ccbc",
    btf = make_kernel_input 'btf "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    headers = make_kernel_input 'headers "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    config = make_kernel_input 'config "0000000000000000000000000000000000000000000000000000000000000000",
  },
  output_classes = [
    'generated-source-bundle,
    'userspace-loader,
    'ebpf-object,
    'kernel-module,
    'test-binary,
    'candidate-pack-manifest,
  ],
  expected_generated_files = [
    { relative_path = "demo.c", class = 'userspace-c, max_bytes = 65536 },
    { relative_path = "demo.ebpf.c", class = 'ebpf-c, max_bytes = 65536 },
    { relative_path = "demo.mod.c", class = 'module-c, max_bytes = 65536 },
    { relative_path = "demo.test.c", class = 'test-c, max_bytes = 65536 },
    { relative_path = "Makefile", class = 'makefile-evidence, max_bytes = 65536 },
    { relative_path = "Kbuild", class = 'kbuild-evidence, max_bytes = 65536 },
  ],
  bpf_compiler_flags = ["-target", "bpf", "-O2", "-Wall", "-Wextra", "-g", "-fno-builtin"],
  userspace_compiler_flags = ["-Wall", "-Wextra", "-O2", "-fPIC", "-lbpf", "-lelf", "-lz"],
  module_compiler_flags = ["-Werror"],
  bounds = {
    max_generated_files = 16,
    max_generated_file_bytes = 65536,
    max_generated_total_bytes = 262144,
    max_compilation_steps = 16,
    max_output_files = 8,
    max_output_bytes = 1048576,
    max_elf_sections = 64,
    max_section_name_bytes = 128,
    max_receipt_blockers = 32,
    max_text_bytes = 1024,
  },
  network_policy = 'denied,
} | mantle.KernelScriptExperimentProfile
