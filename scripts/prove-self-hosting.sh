#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly DEFAULT_TOOLCHAIN="nightly"
readonly PROOF_COMMAND=(cargo test -p crunch --test self_hosting -- --ignored --nocapture)
readonly MIN_TMP_FREE_KIB=4194304
readonly MIN_TMP_FREE_MIB=4096

path_prefix=""
tmp_dir=""
tmp_free_kib="0"

usage() {
  cat <<'EOF'
Usage: ./scripts/prove-self-hosting.sh [--check]

  --check  validate prerequisites, print the proof command, and exit
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

configure_path() {
  path_prefix=""

  prepend_optional_dir "/run/wrappers/bin"
  prepend_optional_dir "$HOME/.cargo/bin"
  prepend_nightly_toolchain
  prepend_tool_dir clang
  prepend_tool_dir mold
  prepend_tool_dir pkg-config
  prepend_tool_dir git
  prepend_tool_dir tar
  prepend_tool_dir xz
  prepend_tool_dir cp
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

configure_sandbox_shell() {
  local shell_path="${SNIX_BUILD_SANDBOX_SHELL:-/bin/sh}"

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

require_tmp_space() {
  local -a df_lines
  local df_line

  tmp_dir="${TMPDIR:-/tmp}"
  if [[ ! -d "$tmp_dir" ]]; then
    die "temporary directory does not exist: $tmp_dir"
  fi
  if [[ ! -w "$tmp_dir" ]]; then
    die "temporary directory is not writable: $tmp_dir"
  fi

  mapfile -t df_lines < <(df -Pk "$tmp_dir")
  if [[ "${#df_lines[@]}" -lt 2 ]]; then
    die "failed to read free space for temporary directory: $tmp_dir"
  fi

  df_line="${df_lines[1]}"
  read -r _ _ _ tmp_free_kib _ <<< "$df_line"
  if [[ -z "$tmp_free_kib" ]]; then
    die "failed to parse free space for temporary directory: $tmp_dir"
  fi
  if (( tmp_free_kib < MIN_TMP_FREE_KIB )); then
    die "only $(( tmp_free_kib / 1024 )) MiB free in $tmp_dir; need at least ${MIN_TMP_FREE_MIB} MiB for the proof"
  fi
}

show_check_summary() {
  note "self-hosting proof check passed"
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
  note "proof command: ${PROOF_COMMAND[*]}"
}

parse_mode() {
  case "${1:-run}" in
    run)
      printf 'run\n'
      ;;
    --check)
      printf 'check\n'
      ;;
    -h|--help)
      printf 'help\n'
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
}

main() {
  local mode
  mode="$(parse_mode "${1:-run}")"

  if [[ "$mode" == "help" ]]; then
    usage
    exit 0
  fi

  require_linux
  require_repo_root
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

  exec "${PROOF_COMMAND[@]}"
}

main "$@"
