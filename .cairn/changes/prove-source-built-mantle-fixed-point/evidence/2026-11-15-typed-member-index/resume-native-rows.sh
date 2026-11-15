#!/usr/bin/env bash
set -euo pipefail
RUN=/home/brittonr/sbfp-20261113-berdKH
QUALIFICATION=/home/brittonr/sbfp-20261113-berdKH/native-row-qualification-20261115.axozfy
SOURCE=/home/brittonr/sbfp-20261113-berdKH/runner-source-8fa84260-aws
MANTLE=/home/brittonr/sbfp-20261113-berdKH/bin/mantle-8fa84260-aws
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
test ! -e "$QUALIFICATION/resume.log"
exec > "$QUALIFICATION/resume.log" 2>&1
finished=0
trap 'status=$?; if [ "$status" -eq 0 ] && [ "$finished" -ne 1 ]; then status=1; fi; printf "native_qualification_exit=%s\n" "$status"; printf "%s\n" "$status" > "$QUALIFICATION/resume.exit"; exit "$status"' EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
set -x
date -Is
hostname
df -B1 "$QUALIFICATION"
free -b
cd "$SOURCE"
test "$("$B3" --no-names "$MANTLE")" = 9f8443f0e1cde27bbab085db844e70078695dbf927af2dc01f83adcecf99ebb7
source_digest() { tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$SOURCE" . | "$B3" --no-names; }
SOURCE_BEFORE=$(source_digest)
printf 'native_source_before_blake3=%s\n' "$SOURCE_BEFORE"
test "$SOURCE_BEFORE" = 462f9c8e68382c0876eede90b1b858386391878718fe7b9b9a3e2f01d3bcce09
test -f "$QUALIFICATION/config/signing-key"
test -f "$QUALIFICATION/config/public-key"
test "$(cat "$QUALIFICATION/config/public-key")" = "$(cat "$QUALIFICATION/key-show-public.txt")"
PUBLIC_KEY=$(cat "$QUALIFICATION/config/public-key")
export CRUNCH_CONFIG_DIR="$QUALIFICATION/config" CRUNCH_NO_FUSE=1 TMPDIR="$QUALIFICATION/tmp"
export PATH="/run/wrappers/bin:$PATH"
unset RUST_LOG RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CRUNCH_STATE_DIR
export QUALIFICATION SOURCE MANTLE PUBLIC_KEY
/run/current-system/sw/bin/nix develop --builders '' --no-write-lock-file -c bash -c '
set -euo pipefail
cd "$SOURCE"
for recipe in gcc-4.7 gcc-10 musl-full gcc-10-final binutils-full; do
  date -Is
  "$MANTLE" --store "$QUALIFICATION/store" --state-dir "$QUALIFICATION/state" --store-prefix /mantle/store build "bootstrap/$recipe.ncl" --no-substitute --jobs 16 --signing-key "$QUALIFICATION/config/signing-key" --trusted-public-keys "$PUBLIC_KEY" > "$QUALIFICATION/evidence/$recipe.stdout" 2> "$QUALIFICATION/evidence/$recipe.stderr"
  mapfile -t roots < "$QUALIFICATION/evidence/$recipe.stdout"
  test "${#roots[@]}" -eq 1
  physical="${roots[0]}"
  case "$physical" in "$QUALIFICATION/store/"*) ;; *) printf "unexpected physical output: %s\n" "$physical" >&2; exit 1 ;; esac
  logical="/mantle/store/${physical##*/}"
  printf "%s\n" "$logical" > "$QUALIFICATION/evidence/$recipe.logical-path"
  "$MANTLE" --store "$QUALIFICATION/store" --state-dir "$QUALIFICATION/state" --store-prefix /mantle/store --json attest show "$logical" > "$QUALIFICATION/evidence/$recipe.attestation.json"
  "$MANTLE" --store "$QUALIFICATION/store" --state-dir "$QUALIFICATION/state" --store-prefix /mantle/store --json attest verify artifact "$logical" > "$QUALIFICATION/evidence/$recipe.attestation-verify.json"
  printf "native_component_complete=%s\n" "$recipe"
done
'
SOURCE_AFTER=$(source_digest)
printf 'native_source_after_blake3=%s\n' "$SOURCE_AFTER"
test "$SOURCE_BEFORE" = "$SOURCE_AFTER"
printf '%s\n' 'non_claim: bounded derivation builds and artifact observations only; no general compiler correctness, source-built fixed-point, seed/provider admission, or release eligibility'
date -Is
finished=1
printf 'native_qualification_complete=1\n'
