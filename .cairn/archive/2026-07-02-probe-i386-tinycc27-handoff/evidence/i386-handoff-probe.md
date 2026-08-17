# i386 TinyCC 0.9.27 handoff probe evidence

Task-ID: probe-i386-tinycc27-handoff
Covers: r[i386_tinycc27.handoff_probe]
Date: 2026-07-02

## Decision

The focused sibling probe now reaches real TinyCC 0.9.27 i386 object emission with real i386 Mes runtime artifacts. This remains diagnostic evidence only: it does not switch production bootstrap routing, and it keeps link/runtime and GNU Make below the next-action boundary.

The probe also repaired the pre-probe source fetch URL for `mescc-tools` from Savannah's redirecting dumb endpoint to the smart HTTPS endpoint (`https://https.git.savannah.gnu.org/git/mescc-tools.git`). Before that repair, pueue tasks 34 and 36 failed before the i386 derivation with `Didn't find 'application/x-git-upload-pack-advertisement' header`; pueue task 45 reached the smart endpoint but hit a transient server IO error; pueue task 47 succeeded.

## Focused probe

Command shape (pueue task 47):

```text
PATH="$BWRAP_DIR:/run/wrappers/bin:/run/current-system/sw/bin:$PATH" \
SNIX_BUILD_SANDBOX_SHELL="$BUSYBOX" \
target/debug/mantle --store /tmp/mantle-i386-handoff-store-v3 \
  --state-dir /tmp/mantle-i386-handoff-state-v3 \
  build bootstrap/spike-i386-mes-runtime-layout.ncl \
  --no-substitute -j 2 --verbose --log-level info
```

Relevant pueue log tail:

```text
i386-tcc27-make: RESULT tcc27_compile_libtcc_unit rc=0
i386-tcc27-make: RESULT tcc27_compile_tccgen_unit rc=0
i386-tcc27-make: RESULT tcc27_decl_initializer_stub_tccgen rc=0
i386-tcc27-make: RESULT tcc27_decl_initializer_stub_full rc=0
i386-tcc27-make: RESULT tcc27_compile_object rc=0
i386-tcc27-make: RESULT tcc27_object_exists rc=0
hermeticity: practical (no degraded facts)
/tmp/mantle-i386-handoff-store-v3/fl32465lp3c8m3ms04f1pj8d0qx3fk67-spike-i386-mes-runtime-layout
```

Output summary:

```text
schema=mantle-i386-tinycc27-handoff-summary-v1
claim=diagnostic-only
selected_input=tcc-0.9.27/tcc.c
selected_object=share/spike-i386-mes-runtime-layout/artifacts/tcc27.o
runtime_artifacts=real-i386-mes-runtime
runtime_libtcc1_object=real
runtime_libtcc1_archive=real
runtime_libc_archive=real
host_i386_execution=supported-by-spike-i386-tinycc26-cross-smoke
make_scope=out-of-scope-until-tcc27-object-handoff-succeeds
status=object-emission-success
blocked_step=none
blocked_rc=0
blocked_signal=none
object_path=share/spike-i386-mes-runtime-layout/artifacts/tcc27.o
object_digest_sha256=dcccba2b238849ccba6acf7187d04edf9c6fc660d99472a9fe5fa52e862f49a4
smoke_boundary=tcc26-i386 -c selected TinyCC 0.9.27 source to i386 object with real Mes runtime inputs
next_action=link/runtime handoff remains separate; GNU Make stays out of scope for this proof
non_claim=object-emission diagnostic only; does not switch production bootstrap routing
```

Digest cross-check (pueue task 156):

```text
dcccba2b238849ccba6acf7187d04edf9c6fc660d99472a9fe5fa52e862f49a4  /tmp/mantle-i386-handoff-store-v3/fl32465lp3c8m3ms04f1pj8d0qx3fk67-spike-i386-mes-runtime-layout/share/spike-i386-mes-runtime-layout/artifacts/tcc27.o
object_digest_sha256=dcccba2b238849ccba6acf7187d04edf9c6fc660d99472a9fe5fa52e862f49a4
```

The summary uses SHA-256 because the bootstrap sandbox exposes `sha256sum` through the stage0 runtime; the algorithm is explicit in the field name. No production content-addressing contract is changed.

## Summary checker

Positive and negative checker self-tests after formatting (pueue task 177):

```text
./scripts/check-i386-tinycc27-handoff-summary.sh --self-test
i386 handoff summary self-test: ok
```

Actual summary validation after formatting (pueue task 178):

```text
./scripts/check-i386-tinycc27-handoff-summary.sh --summary /tmp/mantle-i386-handoff-store-v3/fl32465lp3c8m3ms04f1pj8d0qx3fk67-spike-i386-mes-runtime-layout/share/spike-i386-mes-runtime-layout/summary.txt
i386 handoff summary: valid (object-emission-success)
```

The self-test accepts real object-emission success, signal-derived first-blocker summaries, and host-unsupported blocked summaries. It rejects placeholder runtime overclaims, missing success digests, and blocked summaries that name `blocked_step=none`.

## V3 lifecycle checks

Bootstrap blocker inventory report-only with self-tests (pueue task 166):

```text
./scripts/check-bootstrap-blocker-inventory.sh --report-only --self-test
bootstrap blocker inventory: 40 findings across 4 classes, 396 evidence-backed suppressions, 0 promotion claims, enforce=false
```

Whitespace diff check (pueue task 167):

```text
git diff --check
# completed successfully with no output
```

Cairn validation and gates (pueue task 171):

```text
## validate
  "valid": true
## gate proposal
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
## gate design
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
## gate tasks
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
```

Post-sync validation (pueue task 182):

```text
## validate
  "valid": true
## gate tasks
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
```

Post-archive validation (pueue task 185):

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle | grep -E '"(valid|verdict)"'
  "valid": true
```
