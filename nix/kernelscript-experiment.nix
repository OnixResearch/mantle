{
  onixPkgs,
  coreAdapter,
  sourceRoot,
}:
let
  kernelRelease = "6.18.20";
  kernel = onixPkgs.linuxPackages.kernel;
  kernelDev = kernel.dev;
  kernelBtf = "${kernelDev}/vmlinux";
  ocamlPackages = onixPkgs.ocaml-ng.ocamlPackages_5_2;
  sourceArchive = onixPkgs.fetchurl {
    url = "https://github.com/multikernel/kernelscript/releases/download/v0.1.2/kernelscript-0.1.2-source.tar.gz";
    hash = "sha256-mgC5bh8SfUgGwosHbycKzcS/SoxVjKY2v9n0kmi0ecE=";
  };
  compiler = ocamlPackages.buildDunePackage rec {
    pname = "kernelscript";
    version = "0.1.2";
    src = sourceArchive;
    nativeBuildInputs = [
      ocamlPackages.menhir
      onixPkgs.pkg-config
    ];
    buildInputs = [
      ocamlPackages.alcotest
      onixPkgs.libbpf
      onixPkgs.elfutils
      onixPkgs.zlib
    ];
    strictDeps = true;
    doCheck = true;
  };
  fixtureRoot = sourceRoot + "/packages/kernelscript-experiment/fixtures";
  toolchainClosureInfo = onixPkgs.closureInfo {
    rootPaths = [
      compiler
      kernel
      kernelDev
      kernel.configfile
      onixPkgs.llvmPackages.clang
      onixPkgs.stdenv.cc
      onixPkgs.binutils
      onixPkgs.bpftools
      onixPkgs.libbpf
      onixPkgs.elfutils
      onixPkgs.zlib
      onixPkgs.b3sum
      onixPkgs.jq
    ];
  };
  generatedFileMaxBytes = 8388608;
  adapterBounds = {
    max_generated_files = 8;
    max_generated_file_bytes = generatedFileMaxBytes;
    max_generated_total_bytes = 33554432;
    max_compilation_steps = 16;
    max_output_files = 8;
    max_output_bytes = 67108864;
    max_elf_sections = 512;
    max_section_name_bytes = 256;
    max_receipt_blockers = 32;
    max_text_bytes = 4096;
  };
  toolRequest = role: version: path: {
    inherit role version;
    path = toString path;
    configuration = "locked-nixpkgs:${role}:${version}";
  };
  commonToolchain = [
    (toolRequest "ocaml" ocamlPackages.ocaml.version "${ocamlPackages.ocaml}/bin/ocaml")
    (toolRequest "dune" ocamlPackages.dune_3.version "${ocamlPackages.dune_3}/bin/dune")
    (toolRequest "menhir" ocamlPackages.menhir.version "${ocamlPackages.menhir}/bin/menhir")
    (toolRequest "clang" onixPkgs.llvmPackages.clang.version "${onixPkgs.llvmPackages.clang}/bin/clang")
    (toolRequest "c-compiler" onixPkgs.stdenv.cc.version "${onixPkgs.stdenv.cc}/bin/gcc")
    (toolRequest "bpftool" onixPkgs.bpftools.version "${onixPkgs.bpftools}/bin/bpftool")
    (toolRequest "libbpf" onixPkgs.libbpf.version "${onixPkgs.libbpf}/lib/libbpf.so.${onixPkgs.libbpf.version}")
    (toolRequest "elf-library" onixPkgs.elfutils.version "${onixPkgs.elfutils.out}/lib/libelf-${onixPkgs.elfutils.version}.so")
    (toolRequest "zlib" onixPkgs.zlib.version "${onixPkgs.zlib}/lib/libz.so.${onixPkgs.zlib.version}")
  ];
  commonAdapterRequest = {
    schema = "mantle-kernelscript-core-adapter-request-v1";
    compiler = {
      version = "0.1.2";
      source_revision = "0c80d4e4ac0029d34cbc9d65e76d78c075b64555";
      source_archive_url = "https://github.com/multikernel/kernelscript/releases/download/v0.1.2/kernelscript-0.1.2-source.tar.gz";
      source_archive_path = toString sourceArchive;
      source_archive_sha256 = "9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1";
      source_archive_blake3 = "439431f81df45b043c218f4f5a41917ddd616e0defa35ff134c1cf5273124a57";
      executable_path = "${compiler}/bin/kernelscript";
      closure_path_set_path = "${toolchainClosureInfo}/store-paths";
      closure_package = "onixos-nixpkgs-toolchain-closure-path-set";
      closure_version = "6201e203d09599479a3b3450ed24fa81537ebc4e";
    };
    toolchain = commonToolchain;
    target = {
      architecture = "x86-64";
      kernel_release = kernelRelease;
      cohort_label = "locked-nixpkgs-6201e203-linux-${kernelRelease}-probe-observation";
      btf_path = kernelBtf;
      headers_marker_path = "${kernelDev}/lib/modules/${kernelRelease}/build/Makefile";
      config_path = toString kernel.configfile;
    };
    bpf_compiler_flags = [
      "-target"
      "bpf"
      "-O2"
      "-Wall"
      "-Wextra"
      "-g"
      "-fno-builtin"
    ];
    userspace_compiler_flags = [
      "-O2"
      "-Wall"
      "-Wextra"
      "-lbpf"
      "-lelf"
      "-lz"
    ];
    module_compiler_flags = [ "-Werror" ];
    bounds = adapterBounds;
    receipt_blockers = [ ];
  };
  expectedFile = relative_path: class: {
    inherit relative_path class;
    required = true;
    max_bytes = generatedFileMaxBytes;
  };
  probeAdapterRequest = onixPkgs.writeText "mantle-kernelscript-probe-core-request.json" (
    builtins.toJSON (
      commonAdapterRequest
      // {
        experiment_id = "kernelscript-v0.1.2-probe-observation";
        source = {
          relative_path = "probe_do_exit.ks";
          path = "provided-by-production-shell";
        };
        output_classes = [
          "generated-source-bundle"
          "userspace-loader"
          "ebpf-object"
        ];
        expected_generated_files = [
          (expectedFile "Makefile" "makefile-evidence")
          (expectedFile "probe_do_exit.c" "userspace-c")
          (expectedFile "probe_do_exit.ebpf.c" "ebpf-c")
        ];
      }
    )
  );
  kfuncAdapterRequest = onixPkgs.writeText "mantle-kernelscript-kfunc-core-request.json" (
    builtins.toJSON (
      commonAdapterRequest
      // {
        experiment_id = "kernelscript-v0.1.2-private-kfunc-observation";
        source = {
          relative_path = "private_kfunc.ks";
          path = "provided-by-production-shell";
        };
        toolchain = commonToolchain ++ [
          (toolRequest "kernel-build" kernel.version "${kernelDev}/lib/modules/${kernelRelease}/build/Makefile")
        ];
        output_classes = [
          "generated-source-bundle"
          "userspace-loader"
          "ebpf-object"
          "kernel-module"
        ];
        expected_generated_files = [
          (expectedFile "Kbuild" "kbuild-evidence")
          (expectedFile "Makefile" "makefile-evidence")
          (expectedFile "private_kfunc.c" "userspace-c")
          (expectedFile "private_kfunc.ebpf.c" "ebpf-c")
          (expectedFile "private_kfunc.mod.c" "module-c")
        ];
        receipt_blockers = [
          {
            code = "module-build-and-vm-gate-absent";
            subject = "kernel-module";
            message = "checked Nix route does not build or VM-load the private/kfunc module case";
          }
        ];
      }
    )
  );
  productionShell = onixPkgs.writeShellApplication {
    name = "mantle-kernelscript-production";
    runtimeInputs = [ coreAdapter ] ++ (with onixPkgs; [
      bash
      coreutils
    ]);
    excludeShellChecks = [ "SC2016" ];
    text = ''
      set -euo pipefail
      readonly MAX_OBSERVATION_FILE_BYTES=16777216
      readonly OUTPUT_ROOT="''${1:-''${out:?output root is required}}"
      readonly WORK_ROOT="''${TMPDIR:?TMPDIR is required}/mantle-kernelscript-production"
      readonly COMPILER="${compiler}/bin/kernelscript"
      readonly CORE_ADAPTER="${coreAdapter}/bin/mantle-kernelscript-core-adapter"
      readonly PROBE_CORE_REQUEST="${probeAdapterRequest}"
      readonly KFUNC_CORE_REQUEST="${kfuncAdapterRequest}"
      readonly CLANG="${onixPkgs.llvmPackages.clang}/bin/clang"
      readonly CC="${onixPkgs.stdenv.cc}/bin/cc"
      readonly READELF="${onixPkgs.binutils}/bin/readelf"
      readonly OBJDUMP="${onixPkgs.binutils}/bin/objdump"
      readonly BPFTOOL="${onixPkgs.bpftools}/bin/bpftool"
      readonly B3SUM="${onixPkgs.b3sum}/bin/b3sum"
      readonly JQ="${onixPkgs.jq}/bin/jq"
      readonly LIBBPF="${onixPkgs.libbpf}"
      readonly ELFUTILS="${onixPkgs.elfutils.out}"
      readonly ZLIB="${onixPkgs.zlib}"
      readonly KERNEL_BTF="${kernelBtf}"

      fail() { printf 'mantle-kernelscript-production: %s\n' "$1" >&2; exit 1; }
      digest() { "$B3SUM" --no-names "$1"; }
      require_observation_file() {
        local path="$1"
        local size_bytes
        test -f "$path" || fail "missing regular observation file: $path"
        test ! -L "$path" || fail "unexpected observation symlink: $path"
        size_bytes="$(stat -c %s "$path")"
        test "$size_bytes" -gt 0 || fail "empty observation file: $path"
        test "$size_bytes" -le "$MAX_OBSERVATION_FILE_BYTES" || fail "oversize observation file: $path"
      }

      rm -rf "$WORK_ROOT"
      mkdir -p "$WORK_ROOT/probe" "$WORK_ROOT/kfunc" "$WORK_ROOT/build" "$OUTPUT_ROOT"
      cp "${fixtureRoot}/probe_do_exit.ks" "$WORK_ROOT/probe/"
      cp "${fixtureRoot}/private_kfunc.ks" "${fixtureRoot}/xdp.kh" "$WORK_ROOT/kfunc/"
      (cd "$WORK_ROOT/probe" && "$COMPILER" compile probe_do_exit.ks --output generated --btf-vmlinux-path "$KERNEL_BTF")
      (cd "$WORK_ROOT/kfunc" && "$COMPILER" compile private_kfunc.ks --output generated --btf-vmlinux-path "$KERNEL_BTF")
      "$CORE_ADAPTER" "$PROBE_CORE_REQUEST" "$WORK_ROOT/probe/probe_do_exit.ks" \
        "$WORK_ROOT/probe/generated" "$WORK_ROOT/probe-core-report.json"
      "$CORE_ADAPTER" "$KFUNC_CORE_REQUEST" "$WORK_ROOT/kfunc/private_kfunc.ks" \
        "$WORK_ROOT/kfunc/generated" "$WORK_ROOT/kfunc-core-report.json"

      "$BPFTOOL" btf dump file "$KERNEL_BTF" format c > "$WORK_ROOT/build/vmlinux.h"
      "$CLANG" -target bpf -O2 -Wall -Wextra -g -fno-builtin -D__TARGET_ARCH_x86 \
        -I"$WORK_ROOT/build" -I"$LIBBPF/include" \
        -c "$WORK_ROOT/probe/generated/probe_do_exit.ebpf.c" \
        -o "$WORK_ROOT/build/probe_do_exit.ebpf.o"
      "$BPFTOOL" gen skeleton "$WORK_ROOT/build/probe_do_exit.ebpf.o" > "$WORK_ROOT/build/probe_do_exit.skel.h"
      "$CC" -O2 -Wall -Wextra -I"$WORK_ROOT/build" -I"$LIBBPF/include" \
        "$WORK_ROOT/probe/generated/probe_do_exit.c" \
        -L"$LIBBPF/lib" -L"$ELFUTILS/lib" -L"$ZLIB/lib" \
        -Wl,-rpath,"$LIBBPF/lib:$ELFUTILS/lib:$ZLIB/lib" \
        -lbpf -lelf -lz -o "$WORK_ROOT/build/probe_do_exit"
      for artifact in probe_do_exit.ebpf.o probe_do_exit.skel.h probe_do_exit; do
        require_observation_file "$WORK_ROOT/build/$artifact"
      done

      mkdir -p "$OUTPUT_ROOT/generated/probe" "$OUTPUT_ROOT/generated/kfunc" "$OUTPUT_ROOT/artifacts" "$OUTPUT_ROOT/evidence"
      cp "$WORK_ROOT/probe/generated"/* "$OUTPUT_ROOT/generated/probe/"
      cp "$WORK_ROOT/kfunc/generated"/* "$OUTPUT_ROOT/generated/kfunc/"
      cp "$WORK_ROOT/build/probe_do_exit.ebpf.o" "$WORK_ROOT/build/probe_do_exit.skel.h" \
        "$WORK_ROOT/build/probe_do_exit" "$OUTPUT_ROOT/artifacts/"
      cp "$WORK_ROOT/probe-core-report.json" "$OUTPUT_ROOT/evidence/probe-core-report.json"
      cp "$WORK_ROOT/kfunc-core-report.json" "$OUTPUT_ROOT/evidence/kfunc-core-report.json"
      "$READELF" -h -S -r "$OUTPUT_ROOT/artifacts/probe_do_exit.ebpf.o" > "$OUTPUT_ROOT/evidence/readelf.txt"
      "$OBJDUMP" -h "$OUTPUT_ROOT/artifacts/probe_do_exit.ebpf.o" > "$OUTPUT_ROOT/evidence/objdump.txt"
      "$BPFTOOL" btf dump file "$OUTPUT_ROOT/artifacts/probe_do_exit.ebpf.o" format raw > "$OUTPUT_ROOT/evidence/btf.txt"

      "$JQ" --null-input --sort-keys \
        --arg kernel_blake3 "$(digest "${kernel}/bzImage")" \
        --arg config_blake3 "$(digest "${kernel.configfile}")" \
        --arg btf_blake3 "$(digest "$KERNEL_BTF")" \
        --arg compiler_blake3 "$(digest "$COMPILER")" \
        --arg source_blake3 "$(digest "${sourceArchive}")" \
        --arg closure_blake3 "$(digest "${toolchainClosureInfo}/store-paths")" \
        --arg object_blake3 "$(digest "$OUTPUT_ROOT/artifacts/probe_do_exit.ebpf.o")" \
        --arg loader_blake3 "$(digest "$OUTPUT_ROOT/artifacts/probe_do_exit")" \
        --arg probe_core_report_blake3 "$(digest "$OUTPUT_ROOT/evidence/probe-core-report.json")" \
        --arg kfunc_core_report_blake3 "$(digest "$OUTPUT_ROOT/evidence/kfunc-core-report.json")" \
        --arg probe_core_receipt_blake3 "$("$JQ" --raw-output '.receipt.receipt_identity_blake3' "$OUTPUT_ROOT/evidence/probe-core-report.json")" \
        --arg kfunc_core_receipt_blake3 "$("$JQ" --raw-output '.receipt.receipt_identity_blake3' "$OUTPUT_ROOT/evidence/kfunc-core-report.json")" \
        '{
          schema:"mantle-kernelscript-probe-observation-v1",
          enabled_by_default:false,
          authority_status:"reported-onixos-metadata-not-materialized-or-accepted",
          reported_onixos_metadata:{
            commit:"444be98d44c847daa32a3219285eebba83f51a84",
            tree:"d31d4b597cbe988a230208f8c620b3e0b6bb6673",
            flake_lock_blob:"5d79ac01408bdefb272f66a083c3656f25becdf9",
            kernel_contract_blob:"6b4f930efb03e15a6d2536ef9a93d25c856a9e63"
          },
          materialized_nixpkgs_authority:{
            revision:"6201e203d09599479a3b3450ed24fa81537ebc4e",
            nar_hash:"sha256-ZojAnPuCdy657PbTq5V0Y+AHKhZAIwSIT2cb8UgAz/U="
          },
          compiler:{
            source_revision:"0c80d4e4ac0029d34cbc9d65e76d78c075b64555",
            source_archive_sha256:"9a00b96e1f127d4806c28b076f270acdc4bf4a8c558ca636bfd9f49268b479c1",
            source_archive_blake3:$source_blake3,
            binary_blake3:$compiler_blake3,
            closure_store_path_set_blake3:$closure_blake3,
            dependency_lock:"onixos-nixpkgs-flake-lock",
            upstream_binary_used:false,
            ambient_opam_used:false
          },
          target:{
            architecture:"x86_64", kernel_release:"6.18.20",
            kernel_image_blake3:$kernel_blake3, config_blake3:$config_blake3,
            btf_blake3:$btf_blake3, running_host_btf_used:false
          },
          probe:{ebpf_object_blake3:$object_blake3, loader_blake3:$loader_blake3},
          retained_generated_files:[
            "probe/Makefile", "probe/probe_do_exit.c", "probe/probe_do_exit.ebpf.c",
            "kfunc/Kbuild", "kfunc/Makefile", "kfunc/private_kfunc.c",
            "kfunc/private_kfunc.ebpf.c", "kfunc/private_kfunc.mod.c"
          ],
          generated_makefile_executed:false,
          generated_kbuild_executed:false,
          core_admission_status:"generated-shapes-admitted-receipts-blocked-on-external-target-authority",
          core_admission:{
            adapter_schema:"mantle-kernelscript-core-adapter-report-v1",
            probe:{report_blake3:$probe_core_report_blake3,receipt_identity_blake3:$probe_core_receipt_blake3},
            kfunc:{report_blake3:$kfunc_core_report_blake3,receipt_identity_blake3:$kfunc_core_receipt_blake3}
          },
          module_status:"blocked-no-checked-nix-build-or-vm-load-gate",
          runtime_status:"separate-exact-kernel-vm-gate-required",
          non_claims:["not-language-soundness","not-compiler-soundness","not-kernel-safety","not-production-default","not-onix-deployment","not-chaoscontrol-evidence","not-release-eligibility"]
        }' > "$OUTPUT_ROOT/evidence/probe-observation.json"
      digest "$OUTPUT_ROOT/evidence/probe-observation.json" > "$OUTPUT_ROOT/evidence/probe-observation.blake3"
    '';
  };
  artifacts = onixPkgs.runCommand "mantle-kernelscript-production-artifacts" { } ''
    export TMPDIR="$NIX_BUILD_TOP/tmp"
    mkdir -p "$TMPDIR"
    out="$out" ${productionShell}/bin/mantle-kernelscript-production
  '';
  mantleClosureInfo = onixPkgs.closureInfo { rootPaths = [ productionShell ]; };
  cohort = onixPkgs.runCommand "mantle-kernelscript-production-cohort" { } ''
    mkdir -p "$out/share/mantle/kernelscript"
    {
      printf '{\n  production_shell = "%s",\n  closure_inputs = [\n' '${productionShell}/bin/mantle-kernelscript-production'
      while IFS= read -r store_path; do printf '    "%s",\n' "$store_path"; done < ${mantleClosureInfo}/store-paths
      printf '  ],\n}\n'
    } > "$out/share/mantle/kernelscript/kernelscript-cohort.ncl"
    cp ${sourceRoot}/packages/kernelscript-experiment/production.ncl "$out/share/mantle/kernelscript/production.ncl"
  '';
  structuralCheck = onixPkgs.runCommand "mantle-kernelscript-production-structural-check" { } ''
    ${onixPkgs.jq}/bin/jq --exit-status '
      .schema == "mantle-kernelscript-probe-observation-v1"
      and .enabled_by_default == false
      and .authority_status == "reported-onixos-metadata-not-materialized-or-accepted"
      and .compiler.upstream_binary_used == false
      and .compiler.ambient_opam_used == false
      and .target.running_host_btf_used == false
      and .generated_makefile_executed == false
      and .generated_kbuild_executed == false
      and .core_admission_status == "generated-shapes-admitted-receipts-blocked-on-external-target-authority"
      and .core_admission.adapter_schema == "mantle-kernelscript-core-adapter-report-v1"
      and .module_status == "blocked-no-checked-nix-build-or-vm-load-gate"
    ' ${artifacts}/evidence/probe-observation.json >/dev/null
    ${onixPkgs.jq}/bin/jq --exit-status '
      .schema == "mantle-kernelscript-core-adapter-report-v1"
      and .compiler_admission.admitted == true
      and .target_admission.admitted == false
      and any(.target_admission.blockers[]; .code == "kernel-target-observation-only")
      and .receipt.stage_status == "blocked"
      and .receipt.candidate_packs == []
      and (.receipt.non_claims | index("not-production-readiness")) != null
      and [.generated_manifest.members[].relative_path] == ["Makefile", "probe_do_exit.c", "probe_do_exit.ebpf.c"]
    ' ${artifacts}/evidence/probe-core-report.json >/dev/null
    ${onixPkgs.jq}/bin/jq --exit-status '
      .schema == "mantle-kernelscript-core-adapter-report-v1"
      and .compiler_admission.admitted == true
      and .target_admission.admitted == false
      and any(.target_admission.blockers[]; .code == "kernel-target-observation-only")
      and any(.receipt.blockers[]; .code == "module-build-and-vm-gate-absent")
      and .receipt.stage_status == "blocked"
      and .receipt.candidate_packs == []
      and [.generated_manifest.members[].relative_path] == ["Kbuild", "Makefile", "private_kfunc.c", "private_kfunc.ebpf.c", "private_kfunc.mod.c"]
    ' ${artifacts}/evidence/kfunc-core-report.json >/dev/null
    test "$(${onixPkgs.b3sum}/bin/b3sum --no-names ${artifacts}/evidence/probe-observation.json)" = "$(cat ${artifacts}/evidence/probe-observation.blake3)"
    test "$(${onixPkgs.b3sum}/bin/b3sum --no-names ${artifacts}/evidence/probe-core-report.json)" = "$(${onixPkgs.jq}/bin/jq --raw-output '.core_admission.probe.report_blake3' ${artifacts}/evidence/probe-observation.json)"
    test "$(${onixPkgs.b3sum}/bin/b3sum --no-names ${artifacts}/evidence/kfunc-core-report.json)" = "$(${onixPkgs.jq}/bin/jq --raw-output '.core_admission.kfunc.report_blake3' ${artifacts}/evidence/probe-observation.json)"
    test "$(${onixPkgs.jq}/bin/jq --raw-output '.receipt.receipt_identity_blake3' ${artifacts}/evidence/probe-core-report.json)" = "$(${onixPkgs.jq}/bin/jq --raw-output '.core_admission.probe.receipt_identity_blake3' ${artifacts}/evidence/probe-observation.json)"
    test "$(${onixPkgs.jq}/bin/jq --raw-output '.receipt.receipt_identity_blake3' ${artifacts}/evidence/kfunc-core-report.json)" = "$(${onixPkgs.jq}/bin/jq --raw-output '.core_admission.kfunc.receipt_identity_blake3' ${artifacts}/evidence/probe-observation.json)"
    touch "$out"
  '';
  runtimeCheck = onixPkgs.testers.runNixOSTest {
    name = "mantle-kernelscript-production-runtime";
    nodes.machine = {
      boot.kernelPackages = onixPkgs.linuxPackages;
      environment.systemPackages = [ onixPkgs.bpftools ];
      virtualisation.memorySize = 1024;
    };
    testScript = ''
      start_all()
      machine.wait_for_unit("multi-user.target")
      machine.succeed("test $(uname -r) = ${kernelRelease}")
      machine.succeed("mkdir -p /sys/fs/bpf && (mountpoint -q /sys/fs/bpf || mount -t bpf bpf /sys/fs/bpf)")
      machine.succeed("${onixPkgs.bpftools}/bin/bpftool prog load ${artifacts}/artifacts/probe_do_exit.ebpf.o /sys/fs/bpf/mantle-probe type kprobe")
      machine.succeed("test -e /sys/fs/bpf/mantle-probe && rm /sys/fs/bpf/mantle-probe")
      output = machine.succeed("${artifacts}/artifacts/probe_do_exit")
      assert "attached to do_exit successfully" in output
      assert "probe program detached" in output
    '';
  };
in
{
  inherit
    artifacts
    cohort
    compiler
    productionShell
    runtimeCheck
    structuralCheck
    ;
}
