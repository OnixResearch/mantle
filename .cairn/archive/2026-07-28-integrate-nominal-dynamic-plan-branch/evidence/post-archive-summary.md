# Post-archive summary

Pueue task `3472` ran this command:

```text
CAIRN_ARCHIVE_DATE=2026-07-28 nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- archive integrate-nominal-dynamic-plan-branch --root . --execute
```

The command moved the package to `cairn/archive/2026-07-28-integrate-nominal-dynamic-plan-branch/`.

The archive receipt is `b3c316a6ad508bf1fd4230bb58101946b8a0f10e1132460ffd9c5f34938064d4`. Its `reasons` list is empty.

The archive command did not add the integration requirement to the canonical specification. The requirement was copied from the archived delta into `cairn/specs/build-correctness/spec.md`.

`post-archive-validation-2026-07-28.txt` contains the exact post-archive output. `git diff --check` returned zero. Cairn validation returned `valid: true`.

The active integration change no longer exists. The unrelated StageX materialization change remains active.

No command pushed a branch or moved `main` during this lifecycle action.
