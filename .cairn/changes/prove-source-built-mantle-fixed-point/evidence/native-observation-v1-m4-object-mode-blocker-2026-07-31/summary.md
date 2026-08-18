# Native observation v1: M4 object-mode blocker

## Result

Pueue task `7180` failed closed after `5,270s`.
The proof retained its staging directory and `attempt-status.json`.
It did not reach native-provider admission.

The protected GNU M4 link reported the existing `lib/cloexec.o` as missing.
The failing objects had process-dependent modes:

```text
1150 lib/cloexec.o
1140 lib/close-stream.o
1150 src/m4.o
```

These modes lacked owner-read permission.
TinyCC therefore reported the first object as missing during the link.
This is the same bounded TinyCC musl-v2 mode defect already observed for diffutils.

## Repair

`stagex_m4` now rejects non-regular object outputs.
It then normalizes each compiled object to mode `0644` before the link.
The shell checks the resulting mode before it continues.

Positive and negative unit tests cover mode normalization and non-regular input rejection.
Pueue task `7279` passed all ten active `stagex_m4` tests.

Pueue task `7281` rebuilt the retained failed M4 input with the repair.
All 29 objects had mode `0644` before linking.
The M4 output retained BLAKE3:

```text
3dfd2a1223ff6e0c2c540bc2507a097948ab24e2465f232e2363944e5b704741
```

Strict first-party binary Clippy and `git diff --check` also passed in pueue task `7277`.

## Non-claim

This repair closes the observed GNU M4 object-read boundary only.
It does not prove the complete StageX transition, native-provider admission, or the Mantle fixed point.
