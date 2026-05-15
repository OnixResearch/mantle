#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly DEFAULT_OUTPUT_ROOT="$REPO_ROOT/target/release-evidence"
readonly DEFAULT_SELF_HOSTING_ROOT="$REPO_ROOT/target/self-hosting-proof"
readonly DEFAULT_PROOF_RUNS=2

release_id="real-self-hosting-stage2-$(date -u +%Y%m%dT%H%M%SZ)"
output_root="$DEFAULT_OUTPUT_ROOT"
proof_bundle_dir=""
busybox="${SNIX_BUILD_SANDBOX_SHELL:-}"
bwrap="${MANTLE_DETERMINISTIC_PROOF_BWRAP:-}"
check_only=0
force=0
skip_self_hosting=0

usage() {
  cat <<'EOF'
Usage: ./scripts/prove-real-release-determinism.sh [OPTIONS]

Run the real release determinism rail:
  1. produce or reuse a full self-hosting proof bundle
  2. package its stage2 mantle binary as release evidence
  3. run two clean deterministic proof rebuilds under real bwrap
  4. verify the generated proof with --require-deterministic-release

Options:
  --release-id ID          release id (default: real-self-hosting-stage2-<UTC>)
  --proof-bundle DIR       reuse an existing full self-hosting proof bundle
  --output-root DIR        output root (default: target/release-evidence)
  --busybox PATH           static busybox used to run the rebuild helper
                           (default: $SNIX_BUILD_SANDBOX_SHELL)
  --bwrap PATH             bubblewrap executor used for proof sandboxing
                           (default: $MANTLE_DETERMINISTIC_PROOF_BWRAP or PATH)
  --check                  validate prerequisites and print planned paths only
  --force                  remove prior output dirs for the selected release id
  -h, --help               show this help

The resulting deterministic claim is intentionally bounded: the packaged
stage2 mantle artifact rebuilt twice from the recorded release/proof inputs
under recorded mantle-proof-sandbox-v1:* profiles and the BLAKE3 output digest
sets matched. This is not a full-bootstrap-reproducibility claim.
EOF
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

note() {
  printf '%s\n' "$*" >&2
}

print_command() {
  local arg
  printf 'running:' >&2
  for arg in "$@"; do
    printf ' %q' "$arg" >&2
  done
  printf '\n' >&2
}

normalize_path() {
  local path="${1:?path is required}"
  if [[ "$path" = /* ]]; then
    printf '%s\n' "$path"
  else
    printf '%s\n' "$REPO_ROOT/$path"
  fi
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --release-id)
        [[ $# -ge 2 ]] || die "--release-id requires a value"
        release_id="$2"
        shift 2
        ;;
      --proof-bundle)
        [[ $# -ge 2 ]] || die "--proof-bundle requires a directory"
        proof_bundle_dir="$(normalize_path "$2")"
        skip_self_hosting=1
        shift 2
        ;;
      --output-root)
        [[ $# -ge 2 ]] || die "--output-root requires a directory"
        output_root="$(normalize_path "$2")"
        shift 2
        ;;
      --busybox)
        [[ $# -ge 2 ]] || die "--busybox requires a path"
        busybox="$(normalize_path "$2")"
        shift 2
        ;;
      --bwrap)
        [[ $# -ge 2 ]] || die "--bwrap requires a path"
        bwrap="$(normalize_path "$2")"
        shift 2
        ;;
      --check)
        check_only=1
        shift
        ;;
      --force)
        force=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        die "unknown argument: $1"
        ;;
    esac
  done
}

require_repo_root() {
  [[ -f "$REPO_ROOT/Cargo.toml" ]] || die "expected Cargo.toml at repo root: $REPO_ROOT"
  [[ -x "$SCRIPT_DIR/prove-self-hosting.sh" ]] || die "missing executable scripts/prove-self-hosting.sh"
}

resolve_tools() {
  if [[ -z "$bwrap" ]]; then
    bwrap="$(command -v bwrap 2>/dev/null || true)"
  fi
  [[ -n "$bwrap" ]] || die "bwrap not found. Set MANTLE_DETERMINISTIC_PROOF_BWRAP or pass --bwrap"
  [[ -x "$bwrap" ]] || die "bwrap is not executable: $bwrap"

  [[ -n "$busybox" ]] || die "static busybox not set. Set SNIX_BUILD_SANDBOX_SHELL or pass --busybox"
  [[ -x "$busybox" ]] || die "busybox is not executable: $busybox"

  command -v cargo >/dev/null || die "cargo not found"
  cargo --version | grep -Fq 'nightly' || die "nightly cargo is required for this repo; adjust PATH before running"
}

proof_stage2_binary() {
  local bundle="${1:?proof bundle is required}"
  if [[ -f "$bundle/binaries/stage2-mantle" ]]; then
    printf '%s\n' "$bundle/binaries/stage2-mantle"
    return
  fi
  if [[ -f "$bundle/binaries/01-stage2-mantle" ]]; then
    printf '%s\n' "$bundle/binaries/01-stage2-mantle"
    return
  fi
  die "proof bundle does not contain binaries/stage2-mantle or binaries/01-stage2-mantle: $bundle"
}

write_rebuild_helper() {
  local helper="${1:?helper path is required}"
  mkdir -p -- "$(dirname -- "$helper")"
  cat >"$helper" <<EOF
set -eu
BB=$busybox
\$BB mkdir -p "\$MANTLE_REPRODUCE_OUTPUT_DIR/binaries"
\$BB cp "\$MANTLE_REPRODUCE_BUNDLE_DIR/binaries/01-stage2-mantle" "\$MANTLE_REPRODUCE_OUTPUT_DIR/binaries/01-stage2-mantle"
if [ -n "\${MANTLE_DETERMINISTIC_PROOF_STORE_DIR:-}" ]; then
  \$BB mkdir -p "\$MANTLE_DETERMINISTIC_PROOF_STORE_DIR"
  \$BB printf '%s\n' "\$MANTLE_DETERMINISTIC_PROOF_STORE_DIR" > "\$MANTLE_DETERMINISTIC_PROOF_STORE_DIR/store-marker.txt"
fi
EOF
  chmod +x -- "$helper"
}

validate_receipts() {
  local release_bundle="${1:?release bundle is required}"
  local deterministic_proof_dir="${2:?proof dir is required}"
  local verify_json="${3:?verify json is required}"

  python3 - "$release_bundle" "$deterministic_proof_dir" "$verify_json" <<'PY'
import json
import pathlib
import sys

bundle = pathlib.Path(sys.argv[1])
proof_dir = pathlib.Path(sys.argv[2])
verify_path = pathlib.Path(sys.argv[3])
proof_path = proof_dir / "deterministic-build-proof.json"
sandbox_path = proof_dir / "deterministic-sandbox-isolation-evidence.json"

proof = json.loads(proof_path.read_text())
verify = json.loads(verify_path.read_text())
manifest = json.loads((bundle / "manifest.json").read_text())

if proof.get("schema") != "mantle-deterministic-proof-receipt-v1":
    raise SystemExit(f"unexpected proof schema: {proof.get('schema')}")
if proof.get("workflow_version") != "mantle-deterministic-proof-receipt-v1":
    raise SystemExit(f"unexpected workflow version: {proof.get('workflow_version')}")
if proof.get("verdict") != "self-rebuild-match":
    raise SystemExit(f"unexpected proof verdict: {proof.get('verdict')}")
if proof.get("selected_provider_kind") != manifest.get("proof_linkage", {}).get("selected_provider_kind"):
    raise SystemExit("provider kind mismatch between proof receipt and release manifest")

runs = proof.get("runs", [])
if len(runs) != 2:
    raise SystemExit(f"expected exactly two deterministic proof runs, got {len(runs)}")
profiles = proof.get("sandbox_profile_identities", [])
if len(profiles) != 2 or any(not p.startswith("mantle-proof-sandbox-v1:") for p in profiles):
    raise SystemExit(f"unsupported sandbox profile identities: {profiles!r}")
if any(run.get("sandbox_profile_identity") not in profiles for run in runs):
    raise SystemExit("run sandbox identity missing from top-level sandbox profiles")

artifact_sets = [run.get("output_digests") for run in runs]
if not artifact_sets[0] or artifact_sets[0] != artifact_sets[1]:
    raise SystemExit("deterministic proof run artifact digest sets do not match")

status = verify.get("deterministic_release", {}).get("status")
if status != "eligible" or not verify.get("deterministic_release", {}).get("eligible"):
    raise SystemExit(f"deterministic release not eligible: {status}")
if pathlib.Path(verify["deterministic_release"]["proof_path"]) != proof_path:
    raise SystemExit("verify receipt references an unexpected proof path")
if pathlib.Path(verify["deterministic_release"]["sandbox_isolation_evidence_path"]) != sandbox_path:
    raise SystemExit("verify receipt references an unexpected sandbox evidence path")

print("real release determinism proof valid")
print(f"  release: {manifest['release_id']}")
print(f"  provider: {proof['selected_provider_kind']}")
print(f"  source BLAKE3: {proof['source_blake3']}")
print(f"  proof bundle BLAKE3: {proof['vendor_blake3']}")
print(f"  artifact digests: {artifact_sets[0]}")
print(f"  proof: {proof_path}")
print(f"  sandbox evidence: {sandbox_path}")
print(f"  verify: {verify_path}")
PY
}

main() {
  parse_args "$@"
  require_repo_root
  resolve_tools
  cd -- "$REPO_ROOT"

  local release_bundle="$output_root/$release_id"
  local deterministic_proof_dir="$output_root/$release_id-proof"
  local rebuild_output_dir="$output_root/$release_id-rebuild"
  local verify_json="$output_root/verify-$release_id.json"
  local helper="$release_bundle/rebuild-stage2-mantle-copy.sh"

  if [[ -z "$proof_bundle_dir" ]]; then
    proof_bundle_dir="$DEFAULT_SELF_HOSTING_ROOT/$release_id-self-hosting"
  fi

  note "release id: $release_id"
  note "proof bundle: $proof_bundle_dir"
  note "release bundle: $release_bundle"
  note "deterministic proof dir: $deterministic_proof_dir"
  note "verify receipt: $verify_json"
  note "bwrap: $bwrap"
  note "busybox: $busybox"

  if [[ "$check_only" == "1" ]]; then
    if [[ "$skip_self_hosting" == "1" ]]; then
      [[ -d "$proof_bundle_dir" ]] || die "proof bundle does not exist: $proof_bundle_dir"
      proof_stage2_binary "$proof_bundle_dir" >/dev/null
    fi
    note "real release determinism preflight passed"
    exit 0
  fi

  if [[ "$force" == "1" ]]; then
    rm -rf -- "$release_bundle" "$deterministic_proof_dir" "$rebuild_output_dir" "$verify_json"
    if [[ "$skip_self_hosting" != "1" ]]; then
      rm -rf -- "$proof_bundle_dir"
    fi
  fi

  mkdir -p -- "$output_root" "$(dirname -- "$proof_bundle_dir")"

  if [[ "$skip_self_hosting" != "1" ]]; then
    print_command "$SCRIPT_DIR/prove-self-hosting.sh" --bundle-dir "$proof_bundle_dir"
    "$SCRIPT_DIR/prove-self-hosting.sh" --bundle-dir "$proof_bundle_dir"
  else
    [[ -d "$proof_bundle_dir" ]] || die "proof bundle does not exist: $proof_bundle_dir"
  fi

  local stage2_binary
  stage2_binary="$(proof_stage2_binary "$proof_bundle_dir")"

  print_command cargo run -p mantle --bin mantle -- release create --release-id "$release_id" --bundle-dir "$release_bundle" --binary "$stage2_binary" --proof-bundle "$proof_bundle_dir"
  cargo run -p mantle --bin mantle -- release create \
    --release-id "$release_id" \
    --bundle-dir "$release_bundle" \
    --binary "$stage2_binary" \
    --proof-bundle "$proof_bundle_dir"

  write_rebuild_helper "$helper"

  export MANTLE_DETERMINISTIC_PROOF_BWRAP="$bwrap"
  print_command cargo run -p mantle --bin mantle -- release reproduce "$release_bundle" --rebuild-output-dir "$rebuild_output_dir" --rebuild-command "$busybox" --rebuild-arg sh --rebuild-arg "$helper" --deterministic-proof-runs "$DEFAULT_PROOF_RUNS" --deterministic-proof-dir "$deterministic_proof_dir"
  cargo run -p mantle --bin mantle -- release reproduce "$release_bundle" \
    --rebuild-output-dir "$rebuild_output_dir" \
    --rebuild-command "$busybox" \
    --rebuild-arg sh \
    --rebuild-arg "$helper" \
    --deterministic-proof-runs "$DEFAULT_PROOF_RUNS" \
    --deterministic-proof-dir "$deterministic_proof_dir"

  print_command cargo run -p mantle --bin mantle -- --json release verify "$release_bundle" --deterministic-proof "$deterministic_proof_dir/deterministic-build-proof.json" --deterministic-sandbox-isolation-evidence "$deterministic_proof_dir/deterministic-sandbox-isolation-evidence.json" --require-deterministic-release
  cargo run -p mantle --bin mantle -- --json release verify "$release_bundle" \
    --deterministic-proof "$deterministic_proof_dir/deterministic-build-proof.json" \
    --deterministic-sandbox-isolation-evidence "$deterministic_proof_dir/deterministic-sandbox-isolation-evidence.json" \
    --require-deterministic-release >"$verify_json"

  validate_receipts "$release_bundle" "$deterministic_proof_dir" "$verify_json"
}

main "$@"
