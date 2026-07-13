{
  onixPkgs,
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
  productionShell = onixPkgs.writeShellApplication {
    name = "mantle-kernelscript-production";
    runtimeInputs = with onixPkgs; [
      bash
      coreutils
      diffutils
      findutils
      gnugrep
    ];
    excludeShellChecks = [ "SC2016" ];
    text = ''
      set -euo pipefail
      readonly MAX_FILES=8
      readonly MAX_FILE_BYTES=16777216
      readonly OUTPUT_ROOT="''${1:-''${out:?output root is required}}"
      readonly WORK_ROOT="''${TMPDIR:?TMPDIR is required}/mantle-kernelscript-production"
      readonly COMPILER="${compiler}/bin/kernelscript"
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
      require_file() {
        local path="$1"
        local size_bytes
        test -f "$path" || fail "missing regular file: $path"
        test ! -L "$path" || fail "unexpected symlink: $path"
        size_bytes="$(stat -c %s "$path")"
        test "$size_bytes" -gt 0 || fail "empty file: $path"
        test "$size_bytes" -le "$MAX_FILE_BYTES" || fail "oversize file: $path"
      }
      exact_shape() {
        local directory="$1"
        local expected="$2"
        local count
        find "$directory" -maxdepth 1 -type f -printf '%f\n' | LC_ALL=C sort > "$WORK_ROOT/actual.txt"
        diff -u "$expected" "$WORK_ROOT/actual.txt" || fail "generated shape drift"
        count="$(wc -l < "$WORK_ROOT/actual.txt")"
        test "$count" -le "$MAX_FILES" || fail "generated file count exceeds bound"
        while IFS= read -r name; do require_file "$directory/$name"; done < "$WORK_ROOT/actual.txt"
      }

      rm -rf "$WORK_ROOT"
      mkdir -p "$WORK_ROOT/probe" "$WORK_ROOT/kfunc" "$WORK_ROOT/build" "$OUTPUT_ROOT"
      cp "${fixtureRoot}/probe_do_exit.ks" "$WORK_ROOT/probe/"
      cp "${fixtureRoot}/private_kfunc.ks" "${fixtureRoot}/xdp.kh" "$WORK_ROOT/kfunc/"
      (cd "$WORK_ROOT/probe" && "$COMPILER" compile probe_do_exit.ks --output generated --btf-vmlinux-path "$KERNEL_BTF")
      (cd "$WORK_ROOT/kfunc" && "$COMPILER" compile private_kfunc.ks --output generated --btf-vmlinux-path "$KERNEL_BTF")
      printf '%s\n' Makefile probe_do_exit.c probe_do_exit.ebpf.c | LC_ALL=C sort > "$WORK_ROOT/probe.expected"
      printf '%s\n' Kbuild Makefile private_kfunc.c private_kfunc.ebpf.c private_kfunc.mod.c | LC_ALL=C sort > "$WORK_ROOT/kfunc.expected"
      exact_shape "$WORK_ROOT/probe/generated" "$WORK_ROOT/probe.expected"
      exact_shape "$WORK_ROOT/kfunc/generated" "$WORK_ROOT/kfunc.expected"

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
        require_file "$WORK_ROOT/build/$artifact"
      done

      mkdir -p "$OUTPUT_ROOT/generated/probe" "$OUTPUT_ROOT/generated/kfunc" "$OUTPUT_ROOT/artifacts" "$OUTPUT_ROOT/evidence"
      cp "$WORK_ROOT/probe/generated"/* "$OUTPUT_ROOT/generated/probe/"
      cp "$WORK_ROOT/kfunc/generated"/* "$OUTPUT_ROOT/generated/kfunc/"
      cp "$WORK_ROOT/build/probe_do_exit.ebpf.o" "$WORK_ROOT/build/probe_do_exit.skel.h" \
        "$WORK_ROOT/build/probe_do_exit" "$OUTPUT_ROOT/artifacts/"
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
        '{
          schema:"mantle-kernelscript-production-evidence-v1",
          enabled_by_default:false,
          authority:{
            onixos_commit:"444be98d44c847daa32a3219285eebba83f51a84",
            onixos_tree:"d31d4b597cbe988a230208f8c620b3e0b6bb6673",
            onixos_flake_lock_blob:"5d79ac01408bdefb272f66a083c3656f25becdf9",
            onixos_kernel_contract_blob:"6b4f930efb03e15a6d2536ef9a93d25c856a9e63",
            nixpkgs_revision:"6201e203d09599479a3b3450ed24fa81537ebc4e",
            nixpkgs_nar_hash:"sha256-ZojAnPuCdy657PbTq5V0Y+AHKhZAIwSIT2cb8UgAz/U="
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
          module_status:"blocked-pending-authoritative-module-build-inputs",
          runtime_status:"separate-exact-kernel-vm-gate-required",
          non_claims:["not-language-soundness","not-compiler-soundness","not-kernel-safety","not-production-default","not-onix-deployment","not-chaoscontrol-evidence","not-release-eligibility"]
        }' > "$OUTPUT_ROOT/evidence/production-evidence.json"
      digest "$OUTPUT_ROOT/evidence/production-evidence.json" > "$OUTPUT_ROOT/evidence/production-evidence.blake3"
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
      .schema == "mantle-kernelscript-production-evidence-v1"
      and .enabled_by_default == false
      and .compiler.upstream_binary_used == false
      and .compiler.ambient_opam_used == false
      and .target.running_host_btf_used == false
      and .generated_makefile_executed == false
      and .generated_kbuild_executed == false
      and .module_status == "blocked-pending-authoritative-module-build-inputs"
    ' ${artifacts}/evidence/production-evidence.json >/dev/null
    test "$(${onixPkgs.b3sum}/bin/b3sum --no-names ${artifacts}/evidence/production-evidence.json)" = "$(cat ${artifacts}/evidence/production-evidence.blake3)"
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
