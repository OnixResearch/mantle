#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly DEFAULT_TOOLCHAIN="nightly"
readonly PROOF_COMMAND=(cargo test -p crunch --test self_hosting -- --ignored --nocapture)
readonly MIN_PROOF_SCRATCH_FREE_KIB=4194304
readonly MIN_PROOF_SCRATCH_FREE_MIB=4096
readonly DEFAULT_BUNDLE_ROOT="$REPO_ROOT/target/self-hosting-proof"
readonly PROOF_BUNDLE_ENV="CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR"
readonly PROOF_SCRATCH_ENV="CRUNCH_PROOF_SCRATCH_DIR"
readonly PROOF_MODE_ENV="CRUNCH_SELF_HOSTING_PROOF_MODE"
readonly PROOF_STAGE0_INVENTORY_DOC_ENV="CRUNCH_SELF_HOSTING_STAGE0_INVENTORY_DOC"
readonly PROOF_NO_HOST_TOOLS_ENV="CRUNCH_SELF_HOSTING_NO_HOST_TOOLS"
readonly PROOF_STAGE0_INVENTORY_ENV="CRUNCH_SELF_HOSTING_STAGE0_INVENTORY"
readonly PROOF_BLOCKED_HOST_TOOLS_ENV="CRUNCH_SELF_HOSTING_BLOCKED_HOST_TOOLS"
readonly PROOF_LATER_STAGE_HERMETICITY_ENV="CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE"
readonly PROOF_MODE_FIXED_POINT="fixed-point"
readonly PROOF_MODE_NON_NIX_HOST="non-nix-host"
readonly PROOF_LATER_STAGE_HERMETICITY_DEFAULT="strict"
readonly DEFAULT_SCRATCH_ROOT="$DEFAULT_BUNDLE_ROOT/work"
readonly DEFAULT_SCRATCH_SOURCE="default repo-local policy"
readonly SCRATCH_TMP_SUBDIR="tmp"
readonly SCRATCH_CARGO_TARGET_SUBDIR="cargo-target"
readonly MAX_PATH_SOURCE_DIRS=128
readonly MAX_PATH_LINKS=8192
readonly BLOCKED_NIX_BINARIES=(nix-build nix-store nix-shell nix)
readonly BLOCKED_HOST_TOOL_BINARIES=(git tar cp sh cargo nix-build nix-store nix-shell nix)

path_prefix=""
scratch_root=""
scratch_source=""
tmp_dir=""
cargo_target_dir=""
scratch_free_kib="0"
proof_bundle_dir=""
proof_path_dir=""
mode="run"
proof_mode="$PROOF_MODE_FIXED_POINT"
proof_later_stage_hermeticity="$PROOF_LATER_STAGE_HERMETICITY_DEFAULT"
no_host_tools="0"
stage0_inventory=""
generate_stage0_inventory="0"
proof_cargo=""

usage() {
  cat <<'EOF'
Usage: ./scripts/prove-self-hosting.sh [--check] [--non-nix-host] [--no-host-tools] [--stage0-inventory FILE] [--generate-stage0-inventory FILE] [--bundle-dir DIR]

  --check                   validate prerequisites, print the proof command, and exit
  --non-nix-host            run proof with proof PATH scrubbed of nix-build/nix-store/nix-shell/nix
  --no-host-tools           run proof with common host tools poisoned for stage0 self-build
  --stage0-inventory FILE   stage0 inventory used by --no-host-tools
  --generate-stage0-inventory FILE
                            generate the no-host-tools inventory from explicit CRUNCH_STAGE0_SEED_* paths, then use it
  --bundle-dir DIR          write the proof bundle to DIR (default: target/self-hosting-proof/run-...)
EOF
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

note() {
  printf '%s\n' "$*" >&2
}

cleanup() {
  if [[ -n "$proof_path_dir" && -d "$proof_path_dir" ]]; then
    rm -rf -- "$proof_path_dir"
  fi
}

blocked_nix_tool() {
  local tool_name="${1:?tool name is required}"
  local blocked

  for blocked in "${BLOCKED_NIX_BINARIES[@]}"; do
    if [[ "$tool_name" == "$blocked" ]]; then
      return 0
    fi
  done

  return 1
}

blocked_host_tool() {
  local tool_name="${1:?tool name is required}"
  local blocked

  for blocked in "${BLOCKED_HOST_TOOL_BINARIES[@]}"; do
    if [[ "$tool_name" == "$blocked" ]]; then
      return 0
    fi
  done

  return 1
}

prepend_path_dir() {
  local dir="${1:?directory is required}"

  if [[ ! -d "$dir" ]]; then
    die "required PATH directory is missing: $dir"
  fi

  case ":$path_prefix:" in
    *":$dir:"*)
      return
      ;;
  esac

  if [[ -z "$path_prefix" ]]; then
    path_prefix="$dir"
    return
  fi

  path_prefix="$dir:$path_prefix"
}

prepend_optional_dir() {
  local dir="${1:?directory is required}"

  if [[ -d "$dir" ]]; then
    prepend_path_dir "$dir"
  fi
}

resolve_tool_path() {
  local tool="${1:?tool is required}"
  local candidate
  local -a candidates

  # These path probes are host-convenience discovery only. They help the
  # checked-in self-hosting helper find already-installed tools on NixOS-like
  # machines, but they are not evidence that first bootstrap is Nix-free.
  candidate="$(command -v -- "$tool" 2>/dev/null || true)"
  if [[ -n "$candidate" ]]; then
    printf '%s\n' "$candidate"
    return
  fi

  candidates=(
    "/run/current-system/sw/bin/$tool"
    "$HOME/.nix-profile/bin/$tool"
  )

  if [[ -n "${USER:-}" ]]; then
    candidates+=("/etc/profiles/per-user/$USER/bin/$tool")
  fi

  case "$tool" in
    clang)
      candidates+=(/nix/store/*-clang-wrapper-*/bin/clang)
      ;;
    mold)
      candidates+=(/nix/store/*-mold-*/bin/mold)
      ;;
    pkg-config)
      candidates+=(/nix/store/*-pkg-config-wrapper-*/bin/pkg-config)
      ;;
    bwrap)
      candidates+=(/run/wrappers/bin/bwrap /nix/store/*-bubblewrap-*/bin/bwrap)
      ;;
  esac

  for candidate in "${candidates[@]}"; do
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  done

  return 1
}

prepend_tool_dir() {
  local tool="${1:?tool is required}"
  local tool_path

  tool_path="$(resolve_tool_path "$tool" 2>/dev/null || true)"
  if [[ -z "$tool_path" ]]; then
    die "required tool '$tool' not found"
  fi

  prepend_path_dir "$(dirname -- "$tool_path")"
}

prepend_nightly_toolchain() {
  local toolchain="${CRUNCH_PROOF_RUSTUP_TOOLCHAIN:-$DEFAULT_TOOLCHAIN}"
  local rustup_path
  local cargo_path
  local rustc_path
  local candidate_bin

  rustup_path="$(command -v -- rustup 2>/dev/null || true)"
  if [[ -z "$rustup_path" && -x "$HOME/.cargo/bin/rustup" ]]; then
    rustup_path="$HOME/.cargo/bin/rustup"
  fi

  if [[ -n "$rustup_path" ]]; then
    cargo_path="$("$rustup_path" which --toolchain "$toolchain" cargo 2>/dev/null || true)"
    rustc_path="$("$rustup_path" which --toolchain "$toolchain" rustc 2>/dev/null || true)"
  else
    cargo_path=""
    rustc_path=""
  fi

  if [[ -n "$cargo_path" && -n "$rustc_path" ]]; then
    prepend_path_dir "$(dirname -- "$cargo_path")"
    prepend_path_dir "$(dirname -- "$rustc_path")"
    return
  fi

  for candidate_bin in \
    "$HOME/.rustup/toolchains/$toolchain/bin" \
    "$HOME/.rustup/toolchains/$toolchain-"*/bin
  do
    if [[ -x "$candidate_bin/cargo" && -x "$candidate_bin/rustc" ]]; then
      prepend_path_dir "$candidate_bin"
      return
    fi
  done

  die "nightly toolchain '$toolchain' is not installed under ~/.rustup/toolchains"
}

require_linux() {
  local os_name
  os_name="$(uname -s)"

  if [[ "$os_name" != "Linux" ]]; then
    die "self-hosting proof currently requires Linux; found $os_name"
  fi
}

require_repo_root() {
  if [[ ! -f "$REPO_ROOT/Cargo.toml" ]]; then
    die "expected Cargo.toml at repo root: $REPO_ROOT"
  fi

  if [[ ! -d "$REPO_ROOT/bootstrap" ]]; then
    die "expected bootstrap/ at repo root: $REPO_ROOT"
  fi
}

require_stage0_inventory_doc() {
  local inventory_doc="$REPO_ROOT/docs/bootstrap-stage0-inventory.md"

  if [[ ! -f "$inventory_doc" ]]; then
    die "missing stage0 inventory doc: $inventory_doc"
  fi
}

configure_path() {
  path_prefix=""

  prepend_optional_dir "/run/wrappers/bin"
  prepend_optional_dir "$HOME/.cargo/bin"
  prepend_nightly_toolchain
  prepend_tool_dir clang
  prepend_tool_dir mold
  prepend_tool_dir pkg-config
  prepend_tool_dir git
  prepend_tool_dir stat
  prepend_tool_dir bwrap

  export PATH="$path_prefix${PATH:+:$PATH}"
}

configure_c_compiler() {
  local clang_path

  clang_path="$(command -v -- clang 2>/dev/null || true)"
  if [[ -z "$clang_path" ]]; then
    die "clang not found after PATH setup"
  fi

  export CC="${CC:-$clang_path}"
}

discover_static_sandbox_shell() {
  local candidate
  local override_candidate="${CRUNCH_PROOF_STATIC_BUSYBOX_CANDIDATE:-}"

  if [[ -n "$override_candidate" ]]; then
    candidate="$(normalize_repo_relative_path "$override_candidate")"
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
    return 1
  fi

  # Host-convenience discovery only. This helper may use an already-installed
  # static busybox from common NixOS/Nix locations, but it must not realize one
  # through `nix-build` or any other hidden fallback.
  for candidate in \
    "$REPO_ROOT/target/proof-busybox-static/bin/busybox" \
    "/run/current-system/sw/bin/busybox-static" \
    "/bin/busybox.static" \
    /nix/store/*-busybox-static-*/bin/busybox
  do
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  done

  return 1
}

configure_sandbox_shell() {
  local requested_shell="${SNIX_BUILD_SANDBOX_SHELL:-}"
  local shell_path=""

  if [[ -n "$requested_shell" && "$requested_shell" != "/bin/sh" ]]; then
    shell_path="$(normalize_repo_relative_path "$requested_shell")"
  else
    shell_path="$(discover_static_sandbox_shell 2>/dev/null || true)"
  fi

  if [[ -z "$shell_path" ]]; then
    die "static busybox shell not found. Set SNIX_BUILD_SANDBOX_SHELL to an installed static busybox before running the proof helper"
  fi
  if [[ ! -x "$shell_path" ]]; then
    die "SNIX_BUILD_SANDBOX_SHELL must be executable: $shell_path"
  fi

  export SNIX_BUILD_SANDBOX_SHELL="$shell_path"
}

resolve_openssl_pkgconfig_dir() {
  local candidate
  local -a candidates

  if [[ -n "${CRUNCH_PROOF_OPENSSL_PKGCONFIG:-}" ]]; then
    printf '%s\n' "$CRUNCH_PROOF_OPENSSL_PKGCONFIG"
    return
  fi

  candidates=(
    "$HOME/.nix-profile/lib/pkgconfig"
    "/run/current-system/sw/lib/pkgconfig"
    /nix/store/*-openssl-*-dev/lib/pkgconfig
  )

  if [[ -n "${USER:-}" ]]; then
    candidates+=("/etc/profiles/per-user/$USER/lib/pkgconfig")
  fi

  for candidate in "${candidates[@]}"; do
    if [[ -f "$candidate/openssl.pc" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  done

  return 1
}

configure_openssl_lookup() {
  if pkg-config --exists openssl; then
    return
  fi

  local override_dir
  override_dir="$(resolve_openssl_pkgconfig_dir 2>/dev/null || true)"
  if [[ -z "$override_dir" ]]; then
    die "pkg-config cannot find openssl. Set CRUNCH_PROOF_OPENSSL_PKGCONFIG=/path/to/openssl/lib/pkgconfig"
  fi

  if [[ ! -d "$override_dir" ]]; then
    die "OpenSSL pkg-config directory is not a directory: $override_dir"
  fi

  export PKG_CONFIG_PATH="$override_dir${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

  if ! pkg-config --exists openssl; then
    die "pkg-config still cannot find openssl after adding $override_dir"
  fi
}

require_nightly_rustc() {
  local rustc_version

  rustc_version="$(rustc --version)"
  case "$rustc_version" in
    *nightly*)
      ;;
    *)
      die "nightly rustc required, found: $rustc_version"
      ;;
  esac
}

configure_rust_build_env() {
  local rustc_path

  rustc_path="$(command -v -- rustc 2>/dev/null || true)"
  if [[ -z "$rustc_path" ]]; then
    die "rustc not found after PATH setup"
  fi

  # Keep witness and proof runs independent from stale caller-local wrappers.
  # In particular, a previous non-Nix proof can leave a dead rustc shim path in
  # the parent environment; cargo+sccache otherwise try to reuse it.
  export RUSTC="$rustc_path"
  export RUSTC_WRAPPER=""
  export CARGO_BUILD_RUSTC_WRAPPER=""
  unset CARGO_BUILD_RUSTC
  unset CARGO_ENCODED_RUSTFLAGS
  unset RUSTFLAGS
}

read_free_space_kib() {
  local dir="${1:?directory is required}"
  local stat_output
  local free_blocks
  local block_size
  local free_bytes

  stat_output="$(stat -f -c '%a %S' "$dir" 2>/dev/null || true)"
  if [[ -z "$stat_output" ]]; then
    die "failed to read free space for proof scratch root: $dir"
  fi

  read -r free_blocks block_size <<< "$stat_output"
  if [[ ! "$free_blocks" =~ ^[0-9]+$ ]]; then
    die "failed to parse free blocks for proof scratch root: $dir"
  fi
  if [[ ! "$block_size" =~ ^[0-9]+$ ]]; then
    die "failed to parse filesystem block size for proof scratch root: $dir"
  fi

  free_bytes=$(( free_blocks * block_size ))
  printf '%s\n' "$(( free_bytes / 1024 ))"
}

normalize_repo_relative_path() {
  local path_raw="${1:?path is required}"

  if [[ "$path_raw" == /* ]]; then
    printf '%s\n' "$path_raw"
    return
  fi

  printf '%s/%s\n' "$REPO_ROOT" "$path_raw"
}

require_writable_dir() {
  local dir="${1:?directory is required}"
  local label="${2:?label is required}"

  if ! mkdir -p -- "$dir" 2>/dev/null; then
    die "$label is not usable: $dir. Set $PROOF_SCRATCH_ENV to redirect proof scratch"
  fi
  if [[ ! -d "$dir" ]]; then
    die "$label is not a directory: $dir. Set $PROOF_SCRATCH_ENV to redirect proof scratch"
  fi
  if [[ ! -w "$dir" ]]; then
    die "$label is not writable: $dir. Set $PROOF_SCRATCH_ENV to redirect proof scratch"
  fi
}

resolve_proof_scratch_root() {
  local requested_root="${CRUNCH_PROOF_SCRATCH_DIR:-}"

  if [[ -n "$requested_root" ]]; then
    scratch_root="$(normalize_repo_relative_path "$requested_root")"
    scratch_source="$PROOF_SCRATCH_ENV"
    require_writable_dir "$scratch_root" "proof scratch root from $PROOF_SCRATCH_ENV"
    return
  fi

  scratch_root="$DEFAULT_SCRATCH_ROOT"
  scratch_source="$DEFAULT_SCRATCH_SOURCE"
  require_writable_dir "$scratch_root" "default proof scratch root"
}

configure_proof_scratch() {
  resolve_proof_scratch_root

  tmp_dir="$scratch_root/$SCRATCH_TMP_SUBDIR"
  cargo_target_dir="$scratch_root/$SCRATCH_CARGO_TARGET_SUBDIR"
  require_writable_dir "$tmp_dir" "proof TMPDIR under $scratch_root"
  require_writable_dir "$cargo_target_dir" "proof CARGO_TARGET_DIR under $scratch_root"

  export TMPDIR="$tmp_dir"
  export CARGO_TARGET_DIR="$cargo_target_dir"

  scratch_free_kib="$(read_free_space_kib "$scratch_root")"
  if (( scratch_free_kib < MIN_PROOF_SCRATCH_FREE_KIB )); then
    die "only $(( scratch_free_kib / 1024 )) MiB free in proof scratch root $scratch_root; need at least ${MIN_PROOF_SCRATCH_FREE_MIB} MiB for the proof. Set $PROOF_SCRATCH_ENV to a larger filesystem"
  fi
}

configure_non_nix_path() {
  local dir_count=0
  local link_count=0
  local dir
  local candidate
  local tool_name
  local -a source_dirs

  if [[ "$proof_mode" != "$PROOF_MODE_NON_NIX_HOST" && "$no_host_tools" != "1" ]]; then
    return
  fi

  IFS=':' read -r -a source_dirs <<< "${PATH:-}"
  if (( ${#source_dirs[@]} == 0 )); then
    die "PATH is empty before non-Nix-host scrubbing"
  fi

  proof_path_dir="$tmp_dir/crunch-proof-path-$$"
  rm -rf -- "$proof_path_dir"
  mkdir -p -- "$proof_path_dir"

  shopt -s nullglob
  for dir in "${source_dirs[@]}"; do
    if [[ -z "$dir" ]]; then
      continue
    fi
    dir_count=$(( dir_count + 1 ))
    if (( dir_count > MAX_PATH_SOURCE_DIRS )); then
      shopt -u nullglob
      die "PATH has too many source directories for non-Nix-host proof: $dir_count"
    fi
    if [[ ! -d "$dir" ]]; then
      continue
    fi

    for candidate in "$dir"/*; do
      if [[ ! -x "$candidate" || -d "$candidate" ]]; then
        continue
      fi
      tool_name="$(basename -- "$candidate")"
      if [[ "$proof_mode" == "$PROOF_MODE_NON_NIX_HOST" ]] && blocked_nix_tool "$tool_name"; then
        continue
      fi
      if [[ "$no_host_tools" == "1" ]] && blocked_host_tool "$tool_name"; then
        continue
      fi
      if [[ -e "$proof_path_dir/$tool_name" ]]; then
        continue
      fi
      ln -s -- "$candidate" "$proof_path_dir/$tool_name"
      link_count=$(( link_count + 1 ))
      if (( link_count > MAX_PATH_LINKS )); then
        shopt -u nullglob
        die "non-Nix-host proof PATH exceeded link budget: $link_count"
      fi
    done
  done
  shopt -u nullglob

  export PATH="$proof_path_dir"

  if [[ "$proof_mode" == "$PROOF_MODE_NON_NIX_HOST" ]]; then
    for tool_name in "${BLOCKED_NIX_BINARIES[@]}"; do
      if command -v -- "$tool_name" >/dev/null 2>&1; then
        die "non-Nix-host proof PATH still exposes blocked tool: $tool_name"
      fi
    done
  fi

  if [[ "$no_host_tools" == "1" ]]; then
    for tool_name in "${BLOCKED_HOST_TOOL_BINARIES[@]}"; do
      if command -v -- "$tool_name" >/dev/null 2>&1; then
        die "no-host-tools proof PATH still exposes blocked tool: $tool_name"
      fi
    done
  fi
}

default_proof_bundle_dir() {
  local timestamp

  printf -v timestamp '%(%Y%m%dT%H%M%SZ)T' -1
  printf '%s/run-%s-%s\n' "$DEFAULT_BUNDLE_ROOT" "$timestamp" "$$"
}

normalize_bundle_dir() {
  local bundle_dir_raw="${1:?bundle dir is required}"

  normalize_repo_relative_path "$bundle_dir_raw"
}

resolve_proof_bundle_dir() {
  if [[ -n "$proof_bundle_dir" ]]; then
    normalize_bundle_dir "$proof_bundle_dir"
    return
  fi

  if [[ -n "${CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR:-}" ]]; then
    normalize_bundle_dir "$CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR"
    return
  fi

  default_proof_bundle_dir
}

show_scratch_summary() {
  local bundle_dir="${1:-}"

  note "proof mode: $proof_mode"
  note "proof scratch root: $scratch_root"
  note "proof scratch source: $scratch_source"
  note "proof TMPDIR: $tmp_dir"
  note "proof CARGO_TARGET_DIR: $cargo_target_dir"
  note "proof scratch free: $(( scratch_free_kib / 1024 )) MiB"
  if [[ -n "$bundle_dir" ]]; then
    note "proof bundle dir: $bundle_dir"
  fi
  if [[ "$proof_mode" == "$PROOF_MODE_NON_NIX_HOST" || "$no_host_tools" == "1" ]]; then
    note "proof PATH dir: $proof_path_dir"
  fi
  if [[ "$no_host_tools" == "1" ]]; then
    note "no-host-tools stage0 inventory: $stage0_inventory"
    if [[ "$generate_stage0_inventory" == "1" ]]; then
      note "stage0 inventory generation: explicit CRUNCH_STAGE0_SEED_* paths only"
    fi
    note "blocked host tools: ${BLOCKED_HOST_TOOL_BINARIES[*]}"
  fi
}

resolve_later_stage_hermeticity() {
  local requested="${CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE:-$PROOF_LATER_STAGE_HERMETICITY_DEFAULT}"

  case "$requested" in
    practical|strict)
      printf '%s\n' "$requested"
      ;;
    *)
      die "unsupported later-stage hermeticity mode: $requested"
      ;;
  esac
}

resolve_proof_cargo() {
  proof_cargo="$(command -v -- cargo 2>/dev/null || true)"
  if [[ -z "$proof_cargo" ]]; then
    die "cargo not found before proof PATH scrubbing"
  fi
  if [[ ! -x "$proof_cargo" ]]; then
    die "resolved cargo is not executable: $proof_cargo"
  fi
}

proof_command_display() {
  printf '%s test -p crunch --test self_hosting -- --ignored --nocapture\n' "$proof_cargo"
}

show_check_summary() {
  local bundle_dir
  bundle_dir="$(resolve_proof_bundle_dir)"

  note "self-hosting proof check passed"
  note "repo: $REPO_ROOT"
  note "cargo: $proof_cargo"
  note "rustc: $(command -v rustc)"
  note "clang: $(command -v clang)"
  note "mold: $(command -v mold)"
  note "pkg-config: $(command -v pkg-config)"
  note "bwrap: $(command -v bwrap)"
  note "openssl: $(pkg-config --modversion openssl)"
  note "SNIX_BUILD_SANDBOX_SHELL: $SNIX_BUILD_SANDBOX_SHELL"
  note "later proof-stage hermeticity: $proof_later_stage_hermeticity"
  show_scratch_summary "$bundle_dir"
  note "stage0 inventory doc: $REPO_ROOT/docs/bootstrap-stage0-inventory.md"
  if [[ "$no_host_tools" == "1" ]]; then
    note "no-host-tools stage0 inventory: $stage0_inventory"
    note "blocked host tools: ${BLOCKED_HOST_TOOL_BINARIES[*]}"
  fi
  note "proof command: $(proof_command_display)"
}

parse_args() {
  mode="run"
  proof_mode="$PROOF_MODE_FIXED_POINT"
  proof_later_stage_hermeticity="$PROOF_LATER_STAGE_HERMETICITY_DEFAULT"
  no_host_tools="0"
  stage0_inventory=""
  generate_stage0_inventory="0"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --check)
        mode="check"
        shift
        ;;
      --non-nix-host)
        proof_mode="$PROOF_MODE_NON_NIX_HOST"
        shift
        ;;
      --no-host-tools)
        no_host_tools="1"
        shift
        ;;
      --stage0-inventory)
        shift
        [[ $# -gt 0 ]] || die "--stage0-inventory requires a file"
        [[ "$1" != -* ]] || die "--stage0-inventory requires a file, got option-like value: $1"
        stage0_inventory="$1"
        shift
        ;;
      --generate-stage0-inventory)
        shift
        [[ $# -gt 0 ]] || die "--generate-stage0-inventory requires a file"
        [[ "$1" != -* ]] || die "--generate-stage0-inventory requires a file, got option-like value: $1"
        [[ -z "$stage0_inventory" ]] || die "--generate-stage0-inventory cannot be combined with --stage0-inventory"
        no_host_tools="1"
        generate_stage0_inventory="1"
        stage0_inventory="$1"
        shift
        ;;
      --bundle-dir)
        shift
        [[ $# -gt 0 ]] || die "--bundle-dir requires a directory"
        [[ "$1" != -* ]] || die "--bundle-dir requires a directory, got option-like value: $1"
        proof_bundle_dir="$1"
        shift
        ;;
      -h|--help)
        mode="help"
        return
        ;;
      *)
        die "unknown argument: $1"
        ;;
    esac
  done

  if [[ "$no_host_tools" != "1" && -n "$stage0_inventory" ]]; then
    die "--stage0-inventory requires --no-host-tools"
  fi
  if [[ "$no_host_tools" == "1" && -z "$stage0_inventory" ]]; then
    die "--no-host-tools requires --stage0-inventory FILE"
  fi
  if [[ "$no_host_tools" == "1" ]]; then
    stage0_inventory="$(normalize_repo_relative_path "$stage0_inventory")"
    if [[ "$generate_stage0_inventory" == "0" ]]; then
      [[ -f "$stage0_inventory" ]] || die "stage0 inventory file does not exist: $stage0_inventory"
    fi
  fi
}

generate_stage0_inventory_if_requested() {
  if [[ "$generate_stage0_inventory" != "1" ]]; then
    return
  fi

  note "generating no-host-tools stage0 inventory: $stage0_inventory"
  note "stage0 inventory seed policy: explicit CRUNCH_STAGE0_SEED_* paths only; no PATH or /nix/store discovery"
  "$proof_cargo" run -p crunch -- stage0-inventory --output "$stage0_inventory"
  [[ -f "$stage0_inventory" ]] || die "stage0 inventory generation did not create: $stage0_inventory"
}

update_latest_bundle_link() {
  local bundle_dir="${1:?bundle_dir is required}"
  local latest_link="$DEFAULT_BUNDLE_ROOT/latest"

  mkdir -p "$DEFAULT_BUNDLE_ROOT"
  ln -sfn "$bundle_dir" "$latest_link"
}

main() {
  local bundle_dir
  local proof_status

  parse_args "$@"

  if [[ "$mode" == "help" ]]; then
    usage
    exit 0
  fi

  require_linux
  require_repo_root
  require_stage0_inventory_doc
  configure_path
  configure_c_compiler
  configure_sandbox_shell
  configure_openssl_lookup
  require_nightly_rustc
  resolve_proof_cargo
  proof_later_stage_hermeticity="$(resolve_later_stage_hermeticity)"
  configure_proof_scratch
  configure_non_nix_path
  configure_rust_build_env

  cd "$REPO_ROOT"
  generate_stage0_inventory_if_requested

  if [[ "$mode" == "check" ]]; then
    show_check_summary
    exit 0
  fi

  bundle_dir="$(resolve_proof_bundle_dir)"
  mkdir -p "$(dirname -- "$bundle_dir")"
  export "$PROOF_BUNDLE_ENV=$bundle_dir"
  export "$PROOF_MODE_ENV=$proof_mode"
  export "$PROOF_STAGE0_INVENTORY_DOC_ENV=$REPO_ROOT/docs/bootstrap-stage0-inventory.md"
  export "$PROOF_LATER_STAGE_HERMETICITY_ENV=$proof_later_stage_hermeticity"
  if [[ "$no_host_tools" == "1" ]]; then
    export "$PROOF_NO_HOST_TOOLS_ENV=1"
    export "$PROOF_STAGE0_INVENTORY_ENV=$stage0_inventory"
    export "$PROOF_BLOCKED_HOST_TOOLS_ENV=${BLOCKED_HOST_TOOL_BINARIES[*]}"
  else
    unset "$PROOF_NO_HOST_TOOLS_ENV"
    unset "$PROOF_STAGE0_INVENTORY_ENV"
    unset "$PROOF_BLOCKED_HOST_TOOLS_ENV"
  fi
  show_scratch_summary "$bundle_dir"

  if "$proof_cargo" test -p crunch --test self_hosting -- --ignored --nocapture; then
    proof_status=0
  else
    proof_status=$?
  fi
  if (( proof_status != 0 )); then
    exit "$proof_status"
  fi

  update_latest_bundle_link "$bundle_dir"
  note "proof mode: $proof_mode"
  note "proof bundle: $bundle_dir"
  note "proof manifest: $bundle_dir/manifest.json"
  note "proof summary: $bundle_dir/summary.txt"
  note "latest bundle: $DEFAULT_BUNDLE_ROOT/latest"
}

trap cleanup EXIT

main "$@"
