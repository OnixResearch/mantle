# Design

The v4 receipt updates only the nested `source_frontier_reduction` section in `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`. It records that the focused c-parse make attempt reaches the real `c-parse.o` compile, then the diagnostic matrix shows:

- `autohost_defines_84_system` fails while `autohost_defines_84_skip84_system` and `autohost_defines_84_undef_need64_system` pass, identifying `NEED_64BIT_HOST_WIDE_INT` as a bounded include-recursion/error frontier.
- `autohost_defines_96_undef_need64_gid_system` passes, while adding define 97 (`inline`) with the same undef set segfaults, and adding `#undef inline` passes.
- `autohost_defines_100_undef_need64_gid_inline_rlim_ssize_uid_system` passes, showing the bounded macro set can move through the local autohost window but does not prove full c-parse.

Validation requires these exact diagnostic source markers and stale v3 wording rejection.
