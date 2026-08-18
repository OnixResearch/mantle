set -e

SPOOL_BYTES_MAX=8388608
SPOOL_READ_BYTES_MAX=$((SPOOL_BYTES_MAX + 1))
INVOCATION_COUNT_MAX=4096
LOCK_ATTEMPT_COUNT_MAX=100000
AUTHORITY_FAILURE=125
SED_OPTION_PAIR_ARGUMENT_COUNT=2
YLWRAP_SED_ARGUMENT_COUNT=9
YLWRAP_SED_PROGRAM_COUNT=4
invocation_count=0
input_bytes=0
input_file_count=0

reject_bridge() {
  rejection_reason=$1
  case "${MANTLE_STAGE_X_SED_BRIDGE_AUDIT:-}" in
    /*) if test "$invocation_count" -gt 0; then
      printf '%s\t%s\t0\t%s\trejected:%s\n' "$invocation_count" "$input_bytes" "$input_file_count" "$rejection_reason" >> "$MANTLE_STAGE_X_SED_BRIDGE_AUDIT"
    fi ;;
  esac
  printf '%s\n' "stagex-sed-bridge: $rejection_reason" >&2
  exit "$AUTHORITY_FAILURE"
}

require_absolute_file() {
  test -n "$2" || reject_bridge "missing $1 authority"
  case "$2" in /*) ;; *) reject_bridge "non-absolute $1 authority" ;; esac
  test -f "$2" || reject_bridge "unavailable $1 authority"
}

reserve_invocation() {
  lock_path=$MANTLE_STAGE_X_SED_SPOOL_ROOT/invocation.lock
  attempt_count=0
  while ! "$MANTLE_STAGE_X_SED_BRIDGE_MKDIR" "$lock_path" 2>/dev/null; do
    attempt_count=$((attempt_count + 1))
    test "$attempt_count" -lt "$LOCK_ATTEMPT_COUNT_MAX" || reject_bridge invocation-lock-exhausted
  done
  lock_owned=1
  count_path=$MANTLE_STAGE_X_SED_SPOOL_ROOT/invocation.count
  invocation_count=0
  if test -f "$count_path"; then invocation_count=$("$MANTLE_STAGE_X_SED_BRIDGE_CAT" "$count_path"); fi
  invocation_count=${invocation_count//[[:space:]]/}
  case "$invocation_count" in ''|*[!0-9]*) reject_bridge invalid-invocation-count ;; esac
  test "$invocation_count" -lt "$INVOCATION_COUNT_MAX" || reject_bridge invocation-budget-exhausted
  invocation_count=$((invocation_count + 1))
  printf '%s\n' "$invocation_count" > "$count_path"
  "$MANTLE_STAGE_X_SED_BRIDGE_RMDIR" "$lock_path"
  lock_owned=0
}

is_ylwrap_rewrite() {
  test "$#" -eq "$YLWRAP_SED_ARGUMENT_COUNT" || return 1
  test "$1" = -e || return 1
  test "$2" = '/^#/!b' || return 1
  shift "$SED_OPTION_PAIR_ARGUMENT_COUNT"
  program_index=1
  while test "$program_index" -lt "$YLWRAP_SED_PROGRAM_COUNT"; do
    test "$1" = -e || return 1
    shift "$SED_OPTION_PAIR_ARGUMENT_COUNT"
    program_index=$((program_index + 1))
  done
  test "$#" -eq 1 || return 1
}

classify_arguments() {
  script_seen=0
  script_argument_expected=0
  input_file_count=0
  options_ended=0
  for argument in "$@"; do
    if test "$script_argument_expected" -eq 1; then
      script_seen=1
      script_argument_expected=0
      continue
    fi
    if test "$options_ended" -eq 1; then
      if test "$script_seen" -eq 0; then script_seen=1; else input_file_count=$((input_file_count + 1)); fi
      continue
    fi
    case "$argument" in
      --) options_ended=1 ;;
      -e|-f|--expression|--file) script_argument_expected=1 ;;
      -e?*|-f?*|--expression=*|--file=*) script_seen=1 ;;
      -n|-r|-s|-u|--quiet|--silent|--regexp-extended|--separate|--unbuffered|--posix) ;;
      -*) reject_bridge unsupported-option-shape ;;
      *) if test "$script_seen" -eq 0; then script_seen=1; else input_file_count=$((input_file_count + 1)); fi ;;
    esac
  done
  test "$script_argument_expected" -eq 0 || reject_bridge missing-script-argument
  test "$script_seen" -eq 1 || reject_bridge missing-sed-script
}

require_absolute_file sed "$MANTLE_STAGE_X_SED_BRIDGE_TARGET"
require_absolute_file ylwrap-sed "$MANTLE_STAGE_X_YLWRAP_SED"
require_absolute_file cat "$MANTLE_STAGE_X_SED_BRIDGE_CAT"
require_absolute_file head "$MANTLE_STAGE_X_SED_BRIDGE_HEAD"
require_absolute_file emit "$MANTLE_STAGE_X_SED_BRIDGE_EMIT"
require_absolute_file mkdir "$MANTLE_STAGE_X_SED_BRIDGE_MKDIR"
require_absolute_file rm "$MANTLE_STAGE_X_SED_BRIDGE_RM"
require_absolute_file rmdir "$MANTLE_STAGE_X_SED_BRIDGE_RMDIR"
require_absolute_file wc "$MANTLE_STAGE_X_SED_BRIDGE_WC"
case "$MANTLE_STAGE_X_SED_SPOOL_ROOT" in /*) ;; *) reject_bridge non-absolute-spool-authority ;; esac
test -d "$MANTLE_STAGE_X_SED_SPOOL_ROOT" || reject_bridge unavailable-spool-authority
case "$MANTLE_STAGE_X_SED_BRIDGE_AUDIT" in /*) ;; *) reject_bridge non-absolute-audit-authority ;; esac

lock_path=
lock_owned=0
input_path=
output_path=
cleanup_bridge() {
  if test -n "$input_path" || test -n "$output_path"; then
    "$MANTLE_STAGE_X_SED_BRIDGE_RM" -f "$input_path" "$output_path"
  fi
  if test "$lock_owned" -eq 1; then "$MANTLE_STAGE_X_SED_BRIDGE_RMDIR" "$lock_path"; fi
}
trap cleanup_bridge EXIT
trap 'exit "$AUTHORITY_FAILURE"' HUP INT TERM
classify_arguments "$@"
reserve_invocation
input_path=$MANTLE_STAGE_X_SED_SPOOL_ROOT/input-$invocation_count
output_path=$MANTLE_STAGE_X_SED_SPOOL_ROOT/output-$invocation_count

"$MANTLE_STAGE_X_SED_BRIDGE_HEAD" -c "$SPOOL_READ_BYTES_MAX" > "$input_path"
input_bytes=$("$MANTLE_STAGE_X_SED_BRIDGE_WC" -c < "$input_path")
input_bytes=${input_bytes//[[:space:]]/}
case "$input_bytes" in ''|*[!0-9]*) reject_bridge invalid-input-size ;; esac
test "$input_bytes" -le "$SPOOL_BYTES_MAX" || reject_bridge input-size-exceeded
if test "$input_bytes" -gt 0; then
  test "$input_file_count" -eq 0 || reject_bridge mixed-input-authority
  set -- "$@" "$input_path"
fi

selected_target=$MANTLE_STAGE_X_SED_BRIDGE_TARGET
producer_kind=protected-sed
if is_ylwrap_rewrite "$@"; then
  selected_target=$MANTLE_STAGE_X_YLWRAP_SED
  producer_kind=ylwrap-sed
fi
"$selected_target" "$@" > "$output_path"
child_status=$?
test "$child_status" -eq 0 || exit "$child_status"
output_bytes=$("$MANTLE_STAGE_X_SED_BRIDGE_WC" -c < "$output_path")
output_bytes=${output_bytes//[[:space:]]/}
case "$output_bytes" in ''|*[!0-9]*) reject_bridge invalid-output-size ;; esac
test "$output_bytes" -le "$SPOOL_BYTES_MAX" || reject_bridge output-size-exceeded
printf '%s\t%s\t%s\t%s\tok:%s\n' "$invocation_count" "$input_bytes" "$output_bytes" "$input_file_count" "$producer_kind" >> "$MANTLE_STAGE_X_SED_BRIDGE_AUDIT"
(
  cd "$MANTLE_STAGE_X_SED_SPOOL_ROOT"
  "$MANTLE_STAGE_X_SED_BRIDGE_EMIT" "output-$invocation_count"
)
