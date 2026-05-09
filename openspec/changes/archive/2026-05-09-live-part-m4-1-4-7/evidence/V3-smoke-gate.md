# V3 m4 1.4.7 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.m4.1.4.7

No full direct GNU m4 runtime smoke success is claimed.

The derivation now contains fail-closed installed bridge smokes:

- `define(FOO,bar)FOO` expands to `bar`
- `dnl ignored` emits no output

These checks must pass for any produced output, but they do not promote the bridge as a complete GNU m4 replacement.
