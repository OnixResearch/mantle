#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(dirname -- "$(realpath -- "${BASH_SOURCE[0]}")")"
readonly RUNNER_SOURCE="$SCRIPT_DIR/check-no-std-core.sh"
readonly BASH_BIN_DIR="$(dirname -- "$(command -v bash)")"
readonly SYSTEM_PATH="$BASH_BIN_DIR:/usr/bin:/bin"
readonly BOOTSTRAP_SCENARIO="bootstrap-wasm-target"
readonly MISSING_SCENARIO="missing-wasm-target"
readonly WASM_TARGET="wasm32-unknown-unknown"
readonly LOG_FILE_NAME="runner-events.log"
readonly TARGET_SENTINEL_NAME="wasm-target-installed"
readonly STDOUT_FILE_NAME="stdout.txt"
readonly STDERR_FILE_NAME="stderr.txt"
readonly HELPER_SCRIPTS=(
  check-no-std-core-deps.sh
  check-no-std-core-purity.sh
  check-no-std-core-scope.sh
  check-no-std-core-api-shape.sh
  check-no-std-core-ownership.sh
)

note() {
  printf '%s\n' "$*" >&2
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

assert_file_contains() {
  local needle="$1"
  local path="$2"
  grep -Fq -- "$needle" "$path" || die "expected '$needle' in $path"
}

assert_file_not_contains() {
  local needle="$1"
  local path="$2"
  if [[ -f "$path" ]] && grep -Fq -- "$needle" "$path"; then
    die "did not expect '$needle' in $path"
  fi
}

line_number_of() {
  local needle="$1"
  local path="$2"
  grep -Fn -- "$needle" "$path" | head -n 1 | cut -d: -f1
}

prepare_temp_repo() {
  local temp_dir
  temp_dir="$(mktemp -d)"
  mkdir -p "$temp_dir/bin" "$temp_dir/scripts"
  cp "$RUNNER_SOURCE" "$temp_dir/scripts/check-no-std-core.sh"
  chmod +x "$temp_dir/scripts/check-no-std-core.sh"
  for helper_name in "${HELPER_SCRIPTS[@]}"; do
    cat > "$temp_dir/scripts/$helper_name" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
exit 0
EOF
    chmod +x "$temp_dir/scripts/$helper_name"
  done
  printf '%s\n' "$temp_dir"
}

write_common_fake_tools() {
  local bin_dir="$1"
  cat > "$bin_dir/openspec" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'openspec:%s\n' "$*" >> "$TEST_RUNNER_LOG_FILE"
exit 0
EOF
  chmod +x "$bin_dir/openspec"

  cat > "$bin_dir/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo:%s\n' "$*" >> "$TEST_RUNNER_LOG_FILE"
exit 0
EOF
  chmod +x "$bin_dir/cargo"
}

write_bootstrap_rustup() {
  local bin_dir="$1"
  cat > "$bin_dir/rustup" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
  show)
    [[ $# -eq 2 && "$2" == "active-toolchain" ]] || exit 98
    printf 'fake-nightly (default)\n'
    ;;
  which)
    [[ $# -eq 4 && "$2" == "cargo" && "$3" == "--toolchain" ]] || exit 98
    printf '%s/cargo\n' "$TEST_RUNNER_BIN_DIR"
    ;;
  target)
    if [[ $# -eq 5 && "$2" == "list" && "$3" == "--installed" && "$4" == "--toolchain" ]]; then
      if [[ -f "$TEST_RUNNER_TARGET_FILE" ]]; then
        printf '%s\n' "$TEST_RUNNER_WASM_TARGET"
      fi
      exit 0
    fi
    if [[ $# -eq 5 && "$2" == "add" && "$3" == "--toolchain" && "$5" == "$TEST_RUNNER_WASM_TARGET" ]]; then
      printf 'rustup-target-add:%s\n' "$5" >> "$TEST_RUNNER_LOG_FILE"
      : > "$TEST_RUNNER_TARGET_FILE"
      exit 0
    fi
    exit 98
    ;;
  run)
    [[ $# -ge 4 ]] || exit 98
    shift
    shift
    command_name="$1"
    shift
    exec "$command_name" "$@"
    ;;
  *)
    exit 98
    ;;
esac
EOF
  chmod +x "$bin_dir/rustup"
}

write_missing_target_rustc() {
  local bin_dir="$1"
  cat > "$bin_dir/rustc" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'rustc:%s\n' "$*" >> "$TEST_RUNNER_LOG_FILE"
if [[ $# -eq 4 && "$1" == "--print" && "$2" == "target-libdir" && "$3" == "--target" && "$4" == "$TEST_RUNNER_WASM_TARGET" ]]; then
  printf '%s/missing-target\n' "$TEST_RUNNER_BIN_DIR"
  exit 0
fi
exit 99
EOF
  chmod +x "$bin_dir/rustc"
}

run_bootstrap_scenario() {
  local temp_dir log_file target_file stdout_file stderr_file target_add_line openspec_line
  temp_dir="$(prepare_temp_repo)"
  log_file="$temp_dir/$LOG_FILE_NAME"
  target_file="$temp_dir/$TARGET_SENTINEL_NAME"
  stdout_file="$temp_dir/$STDOUT_FILE_NAME"
  stderr_file="$temp_dir/$STDERR_FILE_NAME"
  export TEST_RUNNER_LOG_FILE="$log_file"
  export TEST_RUNNER_TARGET_FILE="$target_file"
  export TEST_RUNNER_BIN_DIR="$temp_dir/bin"
  export TEST_RUNNER_WASM_TARGET="$WASM_TARGET"
  write_common_fake_tools "$temp_dir/bin"
  write_bootstrap_rustup "$temp_dir/bin"
  PATH="$temp_dir/bin:$SYSTEM_PATH" "$temp_dir/scripts/check-no-std-core.sh" >"$stdout_file" 2>"$stderr_file"
  [[ -f "$target_file" ]] || die "bootstrap scenario did not create wasm target sentinel"
  assert_file_contains "rustup-target-add:$WASM_TARGET" "$log_file"
  assert_file_contains "openspec:validate functional-core" "$log_file"
  assert_file_contains "cargo:check -p crunch-attestation-core" "$log_file"
  target_add_line="$(line_number_of "rustup-target-add:$WASM_TARGET" "$log_file")"
  openspec_line="$(line_number_of "openspec:validate functional-core" "$log_file")"
  [[ -n "$target_add_line" ]] || die "missing rustup target-add log line"
  [[ -n "$openspec_line" ]] || die "missing openspec log line"
  if (( target_add_line >= openspec_line )); then
    die "bootstrap scenario ran openspec before rustup target add"
  fi
  note "bootstrap scenario OK"
}

run_missing_scenario() {
  local temp_dir log_file target_file stdout_file stderr_file status
  temp_dir="$(prepare_temp_repo)"
  log_file="$temp_dir/$LOG_FILE_NAME"
  target_file="$temp_dir/$TARGET_SENTINEL_NAME"
  stdout_file="$temp_dir/$STDOUT_FILE_NAME"
  stderr_file="$temp_dir/$STDERR_FILE_NAME"
  export TEST_RUNNER_LOG_FILE="$log_file"
  export TEST_RUNNER_TARGET_FILE="$target_file"
  export TEST_RUNNER_BIN_DIR="$temp_dir/bin"
  export TEST_RUNNER_WASM_TARGET="$WASM_TARGET"
  write_common_fake_tools "$temp_dir/bin"
  write_missing_target_rustc "$temp_dir/bin"
  set +e
  PATH="$temp_dir/bin:$SYSTEM_PATH" "$temp_dir/scripts/check-no-std-core.sh" >"$stdout_file" 2>"$stderr_file"
  status=$?
  set -e
  if (( status == 0 )); then
    die "missing-target scenario unexpectedly succeeded"
  fi
  assert_file_contains "rustup not found and target $WASM_TARGET is not preinstalled under the active rustc sysroot" "$stderr_file"
  [[ ! -f "$target_file" ]] || die "missing-target scenario should not create wasm target sentinel"
  assert_file_not_contains "openspec:" "$log_file"
  assert_file_not_contains "cargo:" "$log_file"
  note "missing-target scenario OK"
}

main() {
  if [[ $# -ne 1 ]]; then
    die "usage: $0 <$BOOTSTRAP_SCENARIO|$MISSING_SCENARIO>"
  fi
  case "$1" in
    "$BOOTSTRAP_SCENARIO")
      run_bootstrap_scenario
      ;;
    "$MISSING_SCENARIO")
      run_missing_scenario
      ;;
    *)
      die "unknown scenario: $1"
      ;;
  esac
}

main "$@"
