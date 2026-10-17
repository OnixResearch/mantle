# ADR 0111: Isolate APK signing inputs in distinct store objects

- **Status:** Proposed
- **Date:** 2026-10-10

## Context

The direct APK adapter binds an application source tree and optionally a
keystore and password file into staged derivations. Its app source object is
an input to unsigned compilation and packaging stages. Placing a test-only
keystore beneath that tree would make signing material available to stages
which do not need it, even if the `apksigner` derivation also names the key.
Conversely, declaring just a member path from an unrelated store object as a
source input does not declare the complete object the builder must mount.

The reviewed `apksigner` consumes the keystore and private-key passwords as
separate file streams. The real build rejected using the same one-line file
path for both roles, even though the two passwords had the same bytes.

## Decision drivers

- Signing material must not be readable by unsigned build stages.
- The reviewed signing plan declares a `keys/` member relative to a store
  object, not an arbitrary host keystore path.
- The derivation dependency must be a complete store object while the
  `apksigner` command receives the path to its member.
- A path-shape check is not a store signature or proof of execution.

## Decision

`PreparedApk::bind` requires the keystore to be the declared signing member
within a store object **distinct from the application source object**. It
rejects either signing password in that application source object and checks
the keystore and both password files with no-follow filesystem admission
before generating a build graph. The two password paths must differ even
when their bytes match. The `apksigner` derivation declares the complete
keystore and password store objects as inputs; earlier APK stages do not
declare either signing input. The official proof constructs the keystore
under its own separately admitted and signed test-only store object. Store
PathInfo trust and sandbox enforcement remain responsibilities of the
build/store layers, not of the adapter's lexical binder.

Reject the alternatives of copying the key into the app source tree (which
exposes it to unsigned stages) or declaring only the nested key file as a
store input (which fails to bind the complete store object).

## Consequences

Direct signed callers formerly placing a key under the app source root must
provide a separate signing store object with the plan's exact member name,
and distinct keystore-password and private-key-password file paths.
Unsigned builds and pinned-source metadata admission are unchanged. The
isolated fixture and negative binder checks demonstrate the shape boundary;
only a successful separately recorded real SDK build can establish that
these inputs work in Bubblewrap with reviewed Android tools. No content
addressing of a test keystore authorizes use of private production keys.
