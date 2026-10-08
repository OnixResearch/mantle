# Real-binary bootstrap-pin smoke on the rebased branch (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/cli-smoke-20261008T013540Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-cli-smoke.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T01:35:41Z free=627343003648 floor=214748364800 peak_reserve=53687091200 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins head=0f80aa9a4e6e412737df9065ddebcec3afdeb519 CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-cli-smoke.sh
phase=running launcher=2725634 pgid=2725634
phase=child_exited exit=0 elapsed_s=140 free=615747424256
phase=final exit=0 group_survivors=0 end=2026-10-08T01:38:01Z elapsed_s=141 free=615733321728
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
head=0f80aa9a4e6e412737df9065ddebcec3afdeb519 mantle=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle work=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z
=== A. Picolibc: unchanged world, empty PATH (no nickel), conditional cache
+ env PATH=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/empty-path /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin check --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico --cache-dir /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico-cache --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico-plan.json
pico_check1_exit=0
--- pico_check1 stdout
{
  "schema": "mantle-bootstrap-pin-plan-v1",
  "entries": [
    {
      "source": "picolibc",
      "preimage": "f114eda07b6da85c1481e3c039da6b4e60caffad82cd7dcb712ee1bee66a5c47",
      "current": "1.8.12",
      "decision": {
        "status": "current"
      }
    }
  ],
  "digest": "3213c87ba32a09647ae6e7606c97c15c1e750d5897e290cd40dd713fc94262fe"
}
--- pico_check1 stderr
nickel_on_empty_path=absent
+ env PATH=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/empty-path /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin check --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico --cache-dir /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico-cache --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico-plan.json
pico_check2_exit=0
--- pico_check2 stdout
{
  "schema": "mantle-bootstrap-pin-plan-v1",
  "entries": [
    {
      "source": "picolibc",
      "preimage": "f114eda07b6da85c1481e3c039da6b4e60caffad82cd7dcb712ee1bee66a5c47",
      "current": "1.8.12",
      "decision": {
        "status": "current"
      }
    }
  ],
  "digest": "3213c87ba32a09647ae6e7606c97c15c1e750d5897e290cd40dd713fc94262fe"
}
--- pico_check2 stderr
cache_mtime_before=2026-10-07 21:36:55.435815170 -0400 cache_mtime_after=2026-10-07 21:36:55.435815170 -0400 cache_mtime_unchanged=yes
pico_check_stdout_identical=yes
{"hook_identity":"eedd43573c255f9886504690d1b712a3fedd004c42640275baa293172c5f8803","etag":"W/\"adf5415f0a8810eae670ad5d1e1c74831c92676a7f0f00b2acfe23c68765871e\"","last_modified":"Sat, 01 Aug 2026 00:23:55 GMT","version":"1.8.12","release_date":"2026-08-01"}
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin apply --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico-plan.json
pico_apply_unreviewed_exit=2
--- pico_apply_unreviewed stdout
--- pico_apply_unreviewed stderr
error: the following required arguments were not provided:
  --reviewed

Usage: mantle bootstrap-pin apply --plan <PLAN> --reviewed --root <ROOT>

For more information, try '--help'.
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin apply --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico-plan.json --reviewed
pico_apply_exit=0
--- pico_apply stdout
{"schema":"mantle-bootstrap-pin-apply-v1","applied_sources":0}
--- pico_apply stderr
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin apply --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/pico-tampered.json --reviewed
pico_apply_tampered_exit=3
--- pico_apply_tampered stdout
--- pico_apply_tampered stderr
error: mutated or incomplete bootstrap pin plan
pico_root_bytes_unchanged=yes
=== B. CMake: pending candidate, no-write controls, reviewed apply
cmake_export_before_exit=0
cmake_toml_b3_before=5ce71abf2ad00adc99e7b2346edb992bcd36dfa96c5eac07393c3de0c293c207 bytes=613/735
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin check --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake --cache-dir /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake-cache --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake-plan.json
cmake_check_exit=3
--- cmake_check stdout
{
  "schema": "mantle-bootstrap-pin-plan-v1",
  "entries": [
    {
      "source": "cmake",
      "preimage": "5ce71abf2ad00adc99e7b2346edb992bcd36dfa96c5eac07393c3de0c293c207",
      "current": "3.31.8",
      "decision": {
        "status": "candidate",
        "candidate": {
          "version": "4.4.4",
          "release_date": "2026-10-02",
          "hashes": {
            "source": "sha256-eTXQsAodZnA2d3+Sc7fXJzyvp84ftsIMTdfVMMjLCMc="
          }
        }
      }
    }
  ],
  "digest": "7c8f232e110028bb877095aeef9fd1ad08019b12dab41a813e51fae456b08686"
}
--- cmake_check stderr
error: bootstrap source updates pending or resolution failed
recomputed_plan_digest=7c8f232e110028bb877095aeef9fd1ad08019b12dab41a813e51fae456b08686 stored=7c8f232e110028bb877095aeef9fd1ad08019b12dab41a813e51fae456b08686
candidate_version=4.4.4
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin apply --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake-unresealed.json --reviewed
neg_unresealed_exit=3
--- neg_unresealed stdout
--- neg_unresealed stderr
error: mutated or incomplete bootstrap pin plan
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin apply --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake-plan.json --reviewed
neg_stale_preimage_exit=3
--- neg_stale_preimage stdout
--- neg_stale_preimage stderr
error: cmake: source pin changed since check
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin apply --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake-unknown.json --reviewed
neg_unknown_source_exit=3
--- neg_unknown_source stdout
--- neg_unknown_source stderr
error: unknown source unknown-source
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin apply --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake-mismatch.json --reviewed
neg_hash_mismatch_exit=3
--- neg_hash_mismatch stdout
--- neg_hash_mismatch stderr
error: cmake: upstream hash mismatch for source
cmake_root_bytes_unchanged_after_negatives=yes
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin apply --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake-plan.json --reviewed
cmake_apply_exit=0
--- cmake_apply stdout
{"schema":"mantle-bootstrap-pin-apply-v1","applied_sources":1}
--- cmake_apply stderr
--- changed files after reviewed apply
./bootstrap/pins/cmake.toml
./bootstrap/pins/generated/cmake.json
cmake_toml_b3_after=5e4e5d8cfa35e43d26d36263d6706ae26dc8e70a9708329dd7f0c23163cc2fa4 bytes=610/732
cmake_recipe_bytes_unchanged=yes
cmake_export_after_exit=0
after_fetch=[{"url":"https://github.com/Kitware/CMake/releases/download/v4.4.4/cmake-4.4.4.tar.gz","fixed_output":{"algo":"sha256","hash":"sha256-eTXQsAodZnA2d3+Sc7fXJzyvp84ftsIMTdfVMMjLCMc=","mode":"recursive"}}]
before_fetch=[{"url":"https://github.com/Kitware/CMake/releases/download/v3.31.8/cmake-3.31.8.tar.gz","fixed_output":{"algo":"sha256","hash":"sha256-1sQqWhxydhTeQ0PUpW/wToX/u4kVmd6YE97O40G2sV8=","mode":"recursive"}}]
cmake_export_equal_after_version_and_hash_substitution=yes
+ /home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins/debug/mantle bootstrap-pin check --root /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake --cache-dir /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake-cache --plan /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.p2vL4e/cli-smoke-20261008T013654Z/cmake-plan-after.json
cmake_check_after_exit=0
--- cmake_check_after stdout
{
  "schema": "mantle-bootstrap-pin-plan-v1",
  "entries": [
    {
      "source": "cmake",
      "preimage": "5e4e5d8cfa35e43d26d36263d6706ae26dc8e70a9708329dd7f0c23163cc2fa4",
      "current": "4.4.4",
      "decision": {
        "status": "current"
      }
    }
  ],
  "digest": "0a2f0ff6dc8324d84af2c9b99917aeb96ae378b0dcd2266a7d1f2ff545410548"
}
--- cmake_check_after stderr
smoke_done
```
