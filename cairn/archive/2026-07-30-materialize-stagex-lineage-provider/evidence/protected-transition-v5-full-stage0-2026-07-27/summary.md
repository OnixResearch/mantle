# Protected full Stage0 diagnostic — 2026-07-27

The protected run reached `mescc-tools-extra`. `M2-Mesoplanet` then tried to
execute the newly built `AMD64/bin/M2-Planet`. The policy denied that path
because the first full-Stage0 plan did not preauthorize its exact BLAKE3.

The retained stderr records the fail-closed boundary. The retained plan,
mini-Stage0 inventory, and generated recipes support diagnosis. The next run
added `M2-Planet` and its source-built subprocess closure to the plan. The v6
proof supersedes this failed scratch.
