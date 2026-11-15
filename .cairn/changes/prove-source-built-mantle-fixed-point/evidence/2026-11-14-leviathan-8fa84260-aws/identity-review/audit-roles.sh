#!/usr/bin/env bash
# Read-only declared role, route, frozen-helper, archive-source and empty-output checks.
set -euo pipefail
r=/home/brittonr/sbfp-20261113-berdKH
base="$r/prepared-cohort-8fa84260-aws"
f="$r/runner-source-8fa84260-aws"
plan="$base/source-program-preparation-input-plan.json"
report="$base/source-program-preparation.json"
b3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
jq=/run/current-system/sw/bin/jq
tar=/run/current-system/sw/bin/tar
fail(){ printf 'BLOCKER %s\n' "$*" >&2; exit 3; }
value(){ "$b3" --no-names "$1"; }
[[ $("$jq" -r .input_profile_blake3 "$report") == $("$jq" -r .output_manifest_blake3 "$r/source-refresh-8fa84260-aws.json") ]] || fail 'report input profile does not match refreshed authority identity'
[[ $("$jq" -r .output_profile_blake3 "$report") == 1ce22ef7bf87d93e94d6473a14979ce2b4390f8f4312478344c1dc08a1ac78c4 ]] || fail 'unexpected output profile identifier'
[[ $("$jq" -r '[.source_inputs[].role]|unique|length' "$plan") -eq 7 ]] || fail 'plan does not name seven distinct source roles'
for role in mantle-source native-source-bundle rust-source-archive-set stagex-lineage stagex-seed stagex-source-bundle vendor-inputs; do
  row=$("$jq" -r --arg role "$role" '.source_inputs[]|select(.role==$role)|[.kind,.digest_blake3,.size_bytes]|@tsv' "$plan")
  [[ -n $row ]] || fail "absent plan role $role"
  IFS=$'\t' read -r kind digest size <<< "$row"
  case "$role" in
    mantle-source) path="$base/inputs/mantle-source" ;;
    native-source-bundle) path="$base/native-source-closure.json" ;;
    rust-source-archive-set) path="$base/inputs/rust-source-archives" ;;
    stagex-lineage) path="$base/inputs/stagex-lineage/stagex-transition-lineage.json" ;;
    stagex-seed) path="$base/inputs/stagex-seed/hex0-seed" ;;
    stagex-source-bundle) path="$base/inputs/stagex-source-bundle/stagex-source-closure.json" ;;
    vendor-inputs) path="$base/inputs/vendor-deps" ;;
  esac
  if [[ $kind == regular-file ]]; then
    [[ -f $path && ! -L $path && $(stat -c %s "$path") -eq $size && $(value "$path") == "$digest" ]] || fail "plan file role $role differs from physical pre-authoring input"
    printf 'PLAN_FILE_ROLE %s raw_b3=%s bytes=%s\n' "$role" "$digest" "$size"
  else
    [[ $kind == directory && -d $path && ! -L $path ]] || fail "plan directory role $role missing/invalid"
    printf 'PLAN_DIRECTORY_ROLE %s preauthor_digest=%s preauthor_payload_bytes=%s actual_path=%s\n' "$role" "$digest" "$size" "$path"
  fi
done
# Materialization mutates Mantle/Rust/native authority during source preparation; plan digests remain prior input identities.
for pair in 'source_built_mantle_programs_aws_helper.rs:aws-source-helper.rs' 'source_built_mantle_programs_native_helper.rs:source_built_mantle_native_helper.rs'; do
  source=${pair%%:*}; target=${pair#*:}
  cmp -s "$f/src/$source" "$base/program-sources/source-built-mantle-programs/$target" || fail "frozen helper differs: $source"
  printf 'FROZEN_HELPER %s program_member=%s bytes=%s blake3=%s\n' "$source" "$target" "$(stat -c %s "$f/src/$source")" "$(value "$f/src/$source")"
done
for file in source-built/rust/program.json cairn/rust/compiler-environment.rs; do
  cmp -s "$base/program-sources/fixed-point-rust-source-program/$file" "$base/inputs/rust-source-archives/$file" || fail "Rust authored role member differs: $file"
  printf 'RUST_COPIED_AUTHORITY %s bytes=%s blake3=%s\n' "$file" "$(stat -c %s "$base/inputs/rust-source-archives/$file")" "$(value "$base/inputs/rust-source-archives/$file")"
done
selector="$base/inputs/mantle-source/bootstrap/source-built-cold-route.json"
"$jq" -ce '.schema == "mantle-source-built-cold-route-v1" and .stagex_transition == {kind:"manifest-source",role:"native-source-bundle",record_identity:"source-built-stagex-programs",member:"stagex-transition.json"} and .stagex_publication == {kind:"manifest-source",role:"native-source-bundle",record_identity:"source-built-stagex-programs",member:"stagex-provider-publication.json"} and .native == {record_identity:"fixed-point-native-source-program",recipe_member:"source-built/native/program.json",index_member:"source-built/native/program.expansion.json"} and .rust_provider == {kind:"source",role:"rust-source-archive-set",path:"source-built/rust/program.json"} and .mantle_recipes == ["bootstrap/source-built/mantle-stage1.json","bootstrap/source-built/mantle-stage2.json"]' "$selector" >/dev/null || fail 'cold route selects unexpected source records or paths'
for file in "$base/program-sources/source-built-stagex-programs/stagex-transition.json" "$base/program-sources/source-built-stagex-programs/stagex-provider-publication.json" "$base/program-sources/fixed-point-native-source-program/source-built/native/program.json" "$base/program-sources/fixed-point-native-source-program/source-built/native/program.expansion.json" "$base/inputs/mantle-source/bootstrap/source-built/mantle-stage1.json" "$base/inputs/mantle-source/bootstrap/source-built/mantle-stage2.json"; do
 [[ -f $file && ! -L $file ]] || fail "selected route member absent $file"
done
printf 'COLD_ROUTE b3=%s physically_selected_members=6 role_consumption=explicit\n' "$(value "$selector")"
for name in native-store native-state cold-runtime rust-provider stagex-transition outputs transcripts; do
 [[ ! -e "$base/$name" && ! -L "$base/$name" ]] || fail "preparation already has output authority $name"
done
printf 'OUTPUT_AUTHORITY absent_output_roots=7; no provider/build/recipe executed\n'
actual=$("$tar" --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner --format=gnu --create --file - --directory "$f" . | "$b3" --no-names)
[[ $actual == 462f9c8e68382c0876eede90b1b858386391878718fe7b9b9a3e2f01d3bcce09 ]] || fail "FROZEN normalized tar differs: $actual"
printf 'FROZEN_GNU_TAR blake3=%s\n' "$actual"
