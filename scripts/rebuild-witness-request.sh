#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly DEFAULT_TOOLCHAIN="nightly"
readonly WITNESS_SCRATCH_ENV="CRUNCH_WITNESS_SCRATCH_DIR"
readonly WITNESS_CLI_BIN_ENV="CRUNCH_WITNESS_REBUILD_CLI_BIN"
readonly TMP_SUBDIR="tmp"
readonly CARGO_TARGET_SUBDIR="cargo-target"
readonly MAX_PATH_DIRS=128

path_prefix=""
check_only=0
request_dir=""
scratch_root=""
requested_scratch_root=""
declared_request_dir=""
cli_command_mode="cargo"

usage() {
  cat <<'EOF'
Usage: ./scripts/rebuild-witness-request.sh [--check] [--scratch-dir DIR] <request-dir> [release witness-rebuild args...]

Examples:
  ./scripts/rebuild-witness-request.sh request \
    --identity witness-a \
    --system x86_64-linux \
    --toolchain rust-1.91.1 \
    --host-class nixos-25.05

  ./scripts/rebuild-witness-request.sh --check request
EOF
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

note() {
  printf '%s\n' "$*" >&2
}

normalize_path() {
  local raw_path="${1:?path is required}"

  if [[ "$raw_path" == /* ]]; then
    printf '%s\n' "$raw_path"
    return
  fi

  printf '%s/%s\n' "$PWD" "$raw_path"
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
  local rustup_path=""
  local cargo_path=""
  local rustc_path=""
  local candidate_bin

  rustup_path="$(command -v -- rustup 2>/dev/null || true)"
  if [[ -z "$rustup_path" && -x "$HOME/.cargo/bin/rustup" ]]; then
    rustup_path="$HOME/.cargo/bin/rustup"
  fi

  if [[ -n "$rustup_path" ]]; then
    cargo_path="$($rustup_path which --toolchain "$toolchain" cargo 2>/dev/null || true)"
    rustc_path="$($rustup_path which --toolchain "$toolchain" rustc 2>/dev/null || true)"
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

configure_path() {
  path_prefix=""

  prepend_optional_dir "/run/wrappers/bin"
  prepend_optional_dir "$HOME/.cargo/bin"
  prepend_nightly_toolchain
  prepend_tool_dir clang
  prepend_tool_dir mold
  prepend_tool_dir pkg-config
  prepend_tool_dir git
  prepend_tool_dir bwrap

  export PATH="$path_prefix${PATH:+:$PATH}"
}

discover_static_sandbox_shell() {
  local candidate

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
    shell_path="$(normalize_path "$requested_shell")"
  else
    shell_path="$(discover_static_sandbox_shell 2>/dev/null || true)"
  fi

  if [[ -z "$shell_path" ]]; then
    die "static busybox shell not found. Set SNIX_BUILD_SANDBOX_SHELL to an installed static busybox before running the witness rebuild helper"
  fi
  if [[ ! -x "$shell_path" ]]; then
    die "SNIX_BUILD_SANDBOX_SHELL must be executable: $shell_path"
  fi

  export SNIX_BUILD_SANDBOX_SHELL="$shell_path"
}

require_writable_dir() {
  local dir="${1:?directory is required}"
  local label="${2:?label is required}"

  if ! mkdir -p -- "$dir" 2>/dev/null; then
    die "$label is not usable: $dir. Set $WITNESS_SCRATCH_ENV to redirect witness scratch"
  fi
  if [[ ! -d "$dir" ]]; then
    die "$label is not a directory: $dir. Set $WITNESS_SCRATCH_ENV to redirect witness scratch"
  fi
  if [[ ! -w "$dir" ]]; then
    die "$label is not writable: $dir. Set $WITNESS_SCRATCH_ENV to redirect witness scratch"
  fi
}

resolve_default_scratch_root() {
  local request_dir_abs="${1:?request dir is required}"

  printf '%s.work\n' "$request_dir_abs"
}

configure_scratch_env() {
  local request_dir_abs="${1:?request dir is required}"
  local tmp_dir
  local cargo_target_dir

  if [[ -n "$requested_scratch_root" ]]; then
    scratch_root="$(normalize_path "$requested_scratch_root")"
  elif [[ -n "${!WITNESS_SCRATCH_ENV:-}" ]]; then
    scratch_root="$(normalize_path "${!WITNESS_SCRATCH_ENV}")"
  else
    scratch_root="$(resolve_default_scratch_root "$request_dir_abs")"
  fi

  tmp_dir="$scratch_root/$TMP_SUBDIR"
  cargo_target_dir="$scratch_root/$CARGO_TARGET_SUBDIR"

  if [[ "$check_only" -eq 0 ]]; then
    require_writable_dir "$scratch_root" "witness scratch root"
    require_writable_dir "$tmp_dir" "witness TMPDIR"
    require_writable_dir "$cargo_target_dir" "witness CARGO_TARGET_DIR"
  fi

  export "$WITNESS_SCRATCH_ENV=$scratch_root"
  export TMPDIR="$tmp_dir"
  export CARGO_TARGET_DIR="$cargo_target_dir"
}

resolve_cli_command() {
  local override_bin="${CRUNCH_WITNESS_REBUILD_CLI_BIN:-}"

  if [[ -n "$override_bin" ]]; then
    cli_command_mode="binary"
    CLI_COMMAND=("$(normalize_path "$override_bin")")
    return
  fi

  cli_command_mode="cargo"
  CLI_COMMAND=(cargo run --quiet --)
}

parse_args() {
  local -a forwarded=()

  while (($# > 0)); do
    case "$1" in
      --help|-h)
        usage
        exit 0
        ;;
      --check)
        check_only=1
        forwarded+=("$1")
        shift
        ;;
      --scratch-dir)
        if (($# < 2)); then
          die "--scratch-dir requires a value"
        fi
        requested_scratch_root="$2"
        forwarded+=("$1" "$2")
        shift 2
        ;;
      --)
        shift
        while (($# > 0)); do
          forwarded+=("$1")
          shift
        done
        break
        ;;
      -*)
        forwarded+=("$1")
        shift
        if (($# > 0)) && [[ "${forwarded[-1]}" != --check ]] && [[ "$1" != -* ]]; then
          case "${forwarded[-1]}" in
            --identity|--system|--toolchain|--host-class|--signing-key)
              forwarded+=("$1")
              shift
              ;;
          esac
        fi
        ;;
      *)
        if [[ -z "$declared_request_dir" ]]; then
          declared_request_dir="$1"
          shift
          continue
        fi
        forwarded+=("$1")
        shift
        ;;
    esac
  done

  if [[ -z "$declared_request_dir" ]]; then
    die "request-dir is required"
  fi

  request_dir="$(normalize_path "$declared_request_dir")"
  FORWARDED_ARGS=("${forwarded[@]}")
}

main() {
  local -a FORWARDED_ARGS=()
  local -a CLI_COMMAND=()

  parse_args "$@"
  configure_path
  configure_sandbox_shell
  configure_scratch_env "$request_dir"
  resolve_cli_command

  if [[ ! -d "$request_dir" ]]; then
    die "request-dir is not a directory: $request_dir"
  fi

  note "witness request: $request_dir"
  note "witness scratch root: $scratch_root"
  note "SNIX_BUILD_SANDBOX_SHELL: $SNIX_BUILD_SANDBOX_SHELL"
  note "bwrap: $(command -v bwrap)"

  if [[ "$check_only" -eq 1 ]]; then
    note "check-only mode: this validates prerequisites and request parsing only; it does not produce publishable witness sidecars"
  fi

  "${CLI_COMMAND[@]}" release witness-rebuild "$request_dir" "${FORWARDED_ARGS[@]}"
}

main "$@"
