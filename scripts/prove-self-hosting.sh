#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly DEFAULT_TOOLCHAIN="nightly"
readonly PROOF_COMMAND=(cargo test -p crunch --test self_hosting -- --ignored --nocapture)
readonly MIN_TMP_FREE_KIB=4194304
readonly MIN_TMP_FREE_MIB=4096
readonly DEFAULT_BUNDLE_ROOT="$REPO_ROOT/target/self-hosting-proof"
readonly PROOF_BUNDLE_ENV="CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR"
readonly PROOF_MODE_ENV="CRUNCH_SELF_HOSTING_PROOF_MODE"
readonly PROOF_STAGE0_INVENTORY_DOC_ENV="CRUNCH_SELF_HOSTING_STAGE0_INVENTORY_DOC"
readonly PROOF_MODE_FIXED_POINT="fixed-point"
readonly PROOF_MODE_NON_NIX_HOST="non-nix-host"

path_prefix=""
tmp_dir=""
tmp_free_kib="0"
proof_bundle_dir=""
mode="run"
proof_mode="$PROOF_MODE_FIXED_POINT"

usage() {
  cat <<'EOF'
Usage: ./scripts/prove-self-hosting.sh [--check] [--non-nix-host] [--bundle-dir DIR]

  --check            validate prerequisites, print the proof command, and exit
  --non-nix-host     run proof with stage0 PATH scrubbed of nix-build/nix-store/nix-shell/nix
  --bundle-dir DIR   write the proof bundle to DIR (default: target/self-hosting-proof/run-...)
EOF
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

note() {
  printf '%s\n' "$*" >&2
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

  # Host-convenience discovery only. This helper may use an already-installed
  # static busybox from common NixOS/Nix locations, but it must not realize one
  # through `nix-build` or any other hidden fallback.
  for candidate in \
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
    shell_path="$requested_shell"
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

read_free_space_kib() {
  local dir="${1:?directory is required}"
  local stat_output
  local free_blocks
  local block_size
  local free_bytes

  stat_output="$(stat -f -c '%a %S' "$dir" 2>/dev/null || true)"
  if [[ -z "$stat_output" ]]; then
    die "failed to read free space for temporary directory: $dir"
  fi

  read -r free_blocks block_size <<< "$stat_output"
  if [[ ! "$free_blocks" =~ ^[0-9]+$ ]]; then
    die "failed to parse free blocks for temporary directory: $dir"
  fi
  if [[ ! "$block_size" =~ ^[0-9]+$ ]]; then
    die "failed to parse filesystem block size for temporary directory: $dir"
  fi

  free_bytes=$(( free_blocks * block_size ))
  printf '%s\n' "$(( free_bytes / 1024 ))"
}

require_tmp_space() {
  tmp_dir="${TMPDIR:-/tmp}"
  if [[ ! -d "$tmp_dir" ]]; then
    die "temporary directory does not exist: $tmp_dir"
  fi
  if [[ ! -w "$tmp_dir" ]]; then
    die "temporary directory is not writable: $tmp_dir"
  fi

  tmp_free_kib="$(read_free_space_kib "$tmp_dir")"
  if (( tmp_free_kib < MIN_TMP_FREE_KIB )); then
    die "only $(( tmp_free_kib / 1024 )) MiB free in $tmp_dir; need at least ${MIN_TMP_FREE_MIB} MiB for the proof"
  fi
}

default_proof_bundle_dir() {
  local timestamp

  printf -v timestamp '%(%Y%m%dT%H%M%SZ)T' -1
  printf '%s/run-%s-%s\n' "$DEFAULT_BUNDLE_ROOT" "$timestamp" "$$"
}

normalize_bundle_dir() {
  local bundle_dir_raw="${1:?bundle dir is required}"

  if [[ "$bundle_dir_raw" == /* ]]; then
    printf '%s\n' "$bundle_dir_raw"
    return
  fi

  printf '%s/%s\n' "$REPO_ROOT" "$bundle_dir_raw"
}

resolve_proof_bundle_dir() {
  if [[ -n "$proof_bundle_dir" ]]; then
    normalize_bundle_dir "$proof_bundle_dir"
    return
  fi

  default_proof_bundle_dir
}

show_check_summary() {
  local bundle_dir
  bundle_dir="$(resolve_proof_bundle_dir)"

  note "self-hosting proof check passed"
  note "proof mode: $proof_mode"
  note "repo: $REPO_ROOT"
  note "cargo: $(command -v cargo)"
  note "rustc: $(command -v rustc)"
  note "clang: $(command -v clang)"
  note "mold: $(command -v mold)"
  note "pkg-config: $(command -v pkg-config)"
  note "bwrap: $(command -v bwrap)"
  note "openssl: $(pkg-config --modversion openssl)"
  note "SNIX_BUILD_SANDBOX_SHELL: $SNIX_BUILD_SANDBOX_SHELL"
  note "tmpdir: $tmp_dir"
  note "tmp free: $(( tmp_free_kib / 1024 )) MiB"
  note "proof bundle dir: $bundle_dir"
  note "stage0 inventory doc: $REPO_ROOT/docs/bootstrap-stage0-inventory.md"
  note "proof command: ${PROOF_COMMAND[*]}"
}

parse_args() {
  mode="run"
  proof_mode="$PROOF_MODE_FIXED_POINT"

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
  require_tmp_space

  cd "$REPO_ROOT"

  if [[ "$mode" == "check" ]]; then
    show_check_summary
    exit 0
  fi

  bundle_dir="$(resolve_proof_bundle_dir)"
  mkdir -p "$(dirname -- "$bundle_dir")"
  export "$PROOF_BUNDLE_ENV=$bundle_dir"
  export "$PROOF_MODE_ENV=$proof_mode"
  export "$PROOF_STAGE0_INVENTORY_DOC_ENV=$REPO_ROOT/docs/bootstrap-stage0-inventory.md"

  if "${PROOF_COMMAND[@]}"; then
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

main "$@"
