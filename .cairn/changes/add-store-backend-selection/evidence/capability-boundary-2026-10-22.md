# Store capability boundary check on the current tree (2026-10-22)

The focused acceptance for this change runs the deterministic store capability
boundary guard, `tools/check_store_capability_boundary.rs`, over the first-party
production sources. The recorded invocation in `commands.txt` was
`cargo run --bin check_store_capability_boundary`, which is not a bin target:
the guard is a cargo script, so the correct invocation is
`cargo -Zscript tools/check_store_capability_boundary.rs`. The command file now
uses the correct invocation.

With the correct invocation the guard does not pass on the current tree. It
reports:

```
raw_service_escape_count=11
writable_authority_escape_count=1
handle_construction_escape_count=16
casita_type_escape_count=0
casita_session_escape_count=0
```

Reported sites include `src/user_profile_cmd/tests.rs`, `crates/crunch-pipeline/src/watch.rs`,
`crates/crunch-android/src/bin/admit.rs`, and further first-party sites listed
by the guard. The earlier record in `evidence/t43-2026-10-12.md` reported five
raw-service escapes and four handle-construction escapes, so the number of
escapes has grown with the tree since that run.

This is a boundary defect against ADR 0058, not a property of the store backend
selection change. The change therefore stays active: its focused acceptance
cannot pass until the reported escapes are repaired. The repair is tracked as
issue 29.
