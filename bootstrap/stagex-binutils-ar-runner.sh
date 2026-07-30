set -e

MEMBER_COUNT_MAX=256
HEADER_ARG_COUNT=2
GLOBAL_HEADER_BYTES=8
SHORT_NAME_BYTES_MAX=15
MEMBER_NAME_BYTES_MAX=255
MEMBER_BYTES_MAX=67108864
STRING_TABLE_BYTES_MAX=1048576
OUTPUT_BYTES_MAX=536870912
ALIGNMENT_BYTES=2
LONG_NAME_SUFFIX_BYTES=2
INVOCATION_COUNT_MAX=64
AUTHORITY_FAILURE=125
invocation_count=0
member_count=0
archive_bytes=0
staged_archive=''

reject_archive() {
  rejection_reason=$1
  if test "$invocation_count" -gt 0; then
    printf '%s\t%s\t0\t%s\trejected:%s\n' "$invocation_count" "$member_count" "${archive_target:-${archive:-none}}" "$rejection_reason" >> "$MANTLE_STAGE_X_AR_AUDIT"
  fi
  printf '%s\n' "stagex-binutils-ar: $rejection_reason" >&2
  exit "$AUTHORITY_FAILURE"
}

require_absolute_file() {
  test -n "$2" || reject_archive "missing $1 authority"
  case "$2" in /*) ;; *) reject_archive "non-absolute $1 authority" ;; esac
  test -f "$2" || reject_archive "unavailable $1 authority"
}

safe_relative_path() {
  candidate=$1
  case "$candidate" in
    ''|/*|..|../*|*/..|*/../*|*//*|*[!A-Za-z0-9._+/-]*) return 1 ;;
    *) return 0 ;;
  esac
}

reserve_invocation() {
  invocation_count=0
  if test -f "$MANTLE_STAGE_X_AR_COUNT"; then
    invocation_count=$($MANTLE_STAGE_X_AR_CAT "$MANTLE_STAGE_X_AR_COUNT") || reject_archive count-read-failed
  fi
  invocation_count=${invocation_count//[[:space:]]/}
  case "$invocation_count" in ''|*[!0-9]*) reject_archive invalid-invocation-count ;; esac
  test "$invocation_count" -lt "$INVOCATION_COUNT_MAX" || reject_archive invocation-budget-exhausted
  invocation_count=$((invocation_count + 1))
  printf '%s\n' "$invocation_count" > "$MANTLE_STAGE_X_AR_COUNT" || reject_archive count-write-failed
}

require_absolute_file basename "$MANTLE_STAGE_X_AR_BASENAME"
require_absolute_file cat "$MANTLE_STAGE_X_AR_CAT"
require_absolute_file mv "$MANTLE_STAGE_X_AR_MV"
require_absolute_file rm "$MANTLE_STAGE_X_AR_RM"
require_absolute_file wc "$MANTLE_STAGE_X_AR_WC"
cleanup_archive() {
  if test -n "$staged_archive"; then $MANTLE_STAGE_X_AR_RM -f "$staged_archive"; fi
}
trap cleanup_archive EXIT HUP INT TERM
case "$MANTLE_STAGE_X_AR_AUDIT" in /*) ;; *) reject_archive non-absolute-audit-authority ;; esac
case "$MANTLE_STAGE_X_AR_COUNT" in /*) ;; *) reject_archive non-absolute-count-authority ;; esac
case "$MANTLE_STAGE_X_AR_SCRATCH" in /*) ;; *) reject_archive non-absolute-scratch-authority ;; esac
test -d "$MANTLE_STAGE_X_AR_SCRATCH" || reject_archive unavailable-scratch-authority
test "$#" -ge "$HEADER_ARG_COUNT" || reject_archive missing-arguments
flags=$1
archive=$2
shift "$HEADER_ARG_COUNT"
flags=${flags#-}
case "$flags" in rc|cru) ;; *) reject_archive unsupported-mode ;; esac
safe_relative_path "$archive" || reject_archive unsafe-archive-path
test ! -L "$archive" || reject_archive symlink-archive
test ! -e "$archive" || reject_archive existing-archive
archive_target=$archive
member_count=$#
test "$member_count" -gt 0 || reject_archive empty-member-set
test "$member_count" -le "$MEMBER_COUNT_MAX" || reject_archive member-budget-exhausted
reserve_invocation
string_table=$MANTLE_STAGE_X_AR_SCRATCH/long-names-$invocation_count
$MANTLE_STAGE_X_AR_RM -f "$string_table"
: > "$string_table"
seen_names='|'
for member in "$@"; do
  safe_relative_path "$member" || reject_archive unsafe-member-path
  test "$member" != "$archive" || reject_archive archive-is-member
  test -f "$member" || reject_archive missing-member
  test ! -L "$member" || reject_archive symlink-member
  member_name=$($MANTLE_STAGE_X_AR_BASENAME "$member") || reject_archive basename-failed
  case "$member_name" in ''|*[!A-Za-z0-9._+-]*) reject_archive unsafe-member-name ;; esac
  case "$seen_names" in *"|$member_name|"*) reject_archive duplicate-member-name ;; esac
  seen_names="$seen_names$member_name|"
  member_name_bytes=$(printf '%s' "$member_name" | $MANTLE_STAGE_X_AR_WC -c) || reject_archive member-name-size-unavailable
  member_name_bytes=${member_name_bytes//[[:space:]]/}
  case "$member_name_bytes" in ''|*[!0-9]*) reject_archive invalid-member-name-size ;; esac
  test "$member_name_bytes" -le "$MEMBER_NAME_BYTES_MAX" || reject_archive member-name-too-long
  member_bytes=$($MANTLE_STAGE_X_AR_WC -c < "$member") || reject_archive member-size-unavailable
  member_bytes=${member_bytes//[[:space:]]/}
  case "$member_bytes" in ''|*[!0-9]*) reject_archive invalid-member-size ;; esac
  test "$member_bytes" -le "$MEMBER_BYTES_MAX" || reject_archive member-too-large
  if test "$member_name_bytes" -gt "$SHORT_NAME_BYTES_MAX"; then
    printf '%s/\n' "$member_name" >> "$string_table"
  fi
done

write_header() {
  header_name=$1
  payload_bytes=$2
  printf '%-16s%-12s%-6s%-6s%-8s%-10s`\n' "$header_name" 0 0 0 100644 "$payload_bytes" >> "$staged_archive"
}

append_padding() {
  payload_bytes=$1
  if test $((payload_bytes % ALIGNMENT_BYTES)) -ne 0; then printf '\n' >> "$staged_archive"; fi
}

staged_archive=$MANTLE_STAGE_X_AR_SCRATCH/archive-$invocation_count.tmp
test ! -e "$staged_archive" || reject_archive existing-staged-archive
( set -C; : > "$staged_archive" ) || reject_archive create-new-staging-failed
printf '!<arch>\n' >> "$staged_archive"
test "$($MANTLE_STAGE_X_AR_WC -c < "$staged_archive")" -eq "$GLOBAL_HEADER_BYTES" || reject_archive invalid-global-header
string_table_bytes=$($MANTLE_STAGE_X_AR_WC -c < "$string_table") || reject_archive string-table-size-unavailable
string_table_bytes=${string_table_bytes//[[:space:]]/}
case "$string_table_bytes" in ''|*[!0-9]*) reject_archive invalid-string-table-size ;; esac
test "$string_table_bytes" -le "$STRING_TABLE_BYTES_MAX" || reject_archive string-table-too-large
if test "$string_table_bytes" -gt 0; then
  write_header // "$string_table_bytes"
  $MANTLE_STAGE_X_AR_CAT "$string_table" >> "$staged_archive"
  append_padding "$string_table_bytes"
fi
long_name_offset=0
member_index=0
for member in "$@"; do
  member_name=$($MANTLE_STAGE_X_AR_BASENAME "$member") || reject_archive basename-failed
  member_name_bytes=$(printf '%s' "$member_name" | $MANTLE_STAGE_X_AR_WC -c) || reject_archive member-name-size-unavailable
  member_name_bytes=${member_name_bytes//[[:space:]]/}
  member_bytes=$($MANTLE_STAGE_X_AR_WC -c < "$member") || reject_archive member-size-unavailable
  member_bytes=${member_bytes//[[:space:]]/}
  if test "$member_name_bytes" -gt "$SHORT_NAME_BYTES_MAX"; then
    header_name="/$long_name_offset"
    long_name_offset=$((long_name_offset + member_name_bytes + LONG_NAME_SUFFIX_BYTES))
  else
    header_name="$member_name/"
  fi
  write_header "$header_name" "$member_bytes"
  $MANTLE_STAGE_X_AR_CAT "$member" >> "$staged_archive"
  append_padding "$member_bytes"
  member_index=$((member_index + 1))
done
$MANTLE_STAGE_X_AR_RM -f "$string_table"
test "$member_index" -eq "$member_count" || reject_archive member-count-mismatch
archive_bytes=$($MANTLE_STAGE_X_AR_WC -c < "$staged_archive") || reject_archive output-size-unavailable
archive_bytes=${archive_bytes//[[:space:]]/}
case "$archive_bytes" in ''|*[!0-9]*) reject_archive invalid-output-size ;; esac
test "$archive_bytes" -le "$OUTPUT_BYTES_MAX" || reject_archive output-too-large
test -s "$staged_archive" || reject_archive output-missing
test ! -e "$archive_target" || reject_archive publication-target-exists
$MANTLE_STAGE_X_AR_MV "$staged_archive" "$archive_target" || reject_archive publication-failed
staged_archive=''
test -s "$archive_target" || reject_archive published-output-missing
printf '%s\t%s\t%s\t%s\tok\n' "$invocation_count" "$member_count" "$archive_bytes" "$archive_target" >> "$MANTLE_STAGE_X_AR_AUDIT"
