# ADR 0087: Reuse the local Rust cache daemon for C/C++ compile objects

## Status

Proposed (2026-10-06). The isolated real-daemon GCC/G++ driver path now has
local native/cold/hit/relocated object and depfile byte parity and classified
probe-result replay checks. The timed uncached representative bootstrap
baseline, representative-chain cache-on/off byte parity, real cache-off
source-built fixed point, and strict release-proof gates have not been
executed for this change.

## Context

Bootstrap StageX and GCC compiles dominate iteration time. The existing Rust cache has a per-user daemon, Unix socket, receipts, and a local `rust-unit-cache` state directory. The C/C++ bootstrap driver must preserve protected execution's declared executable inventory and must not turn a mutable local cache into a derivation input. Argument-only keys cannot detect changed headers; the C compiler's depfile supplies a learned dependency set. Cache locality and content hashing do not confer compiler correctness or writer trust.

## Decision Drivers

- Reuse existing per-user transport and bounded storage instead of operating a second daemon.
- Preserve unchanged derivation identity, normal compiler semantics, and protected-exec inventory auditing when caching is unavailable or an invocation cannot be classified.
- Invalidate a hit on changed tool, source, normalized flags, platform, include/source root tree, or learned depfile input without keying on ephemeral store paths.
- Keep cached objects and receipts outside strict proof authority.

## Decision

The proposed bootstrap admission covers explicitly inventoried
`x86_64-linux-musl-*` and stage TinyCC/GCC compiler invocations, not
arbitrary PATH interception. The standalone driver accepts an absolute
compiler executable and a deliberately narrow classified flag grammar;
it does **not** establish that StageX/GCC builds or protected inventories
are wired to it. Compiler executable, Unix socket, receipt path, and
platform digest are explicit CLI facts; production admission must
additionally bind both driver and real compiler to declared executable
inventory entries. Never insert a driver by ambient PATH mutation.

The current cache classifier accepts only a narrow GNU GCC/G++ object grammar;
stage TinyCC or other unsupported commands forward unchanged.

The typed `mantle-cc-cache-policy-v2` distinguishes `off`, `local-read`,
and `local-read-write`, but defaults **off** until bootstrap inventory
and proof exclusions are actually wired. Off mode carries empty paths
and digests. On-mode admission requires absolute driver, compiler, socket,
and receipt paths plus lowercase 64-hex BLAKE3
`driver_digest_blake3`, `compiler_digest_blake3`, and `platform_digest`.
The daemon MUST check actual driver and compiler executable bytes against
the declared digests before enabling C wire operations. Unknown modes and
ambient path policies are rejected by the contract. Unclassified commands
and daemon outages forward the **exact original arguments** to the real
compiler rather than guessing a cache key; a relative compiler path is
rejected without execution.

Reuse `mantle-rust-cache-daemon`'s same-UID peer check, private per-user state, and bounded JSON protocol with an eight-byte length prefix. Add C `manifest`, `read`, and `publish` operations. The C object cap is 4,194,304 bytes and the frame cap is 8,388,608 bytes; roots, dependencies, and arguments are bounded as well. A published object is a private content-addressed **local file in the Rust unit cache state**, associated with a depfile manifest. This decision does **not** imply castore blob retention or a trusted store PathInfo.

For a classified GNU compile, the v2 action key binds compiler and source
content, explicit platform digest, normalized arguments without store paths,
and whole-tree digests of source/include roots. The learned depfile manifest
retains prerequisites in compiler encounter order as root-relative labels and
content digests; a hit verifies the complete current manifest and reproduces
GCC's 72-column depfile wrapping alongside the object bytes. Relative artifact
paths and unrecognized make syntax do not enter this cache. The v2 action,
record, request, and response schemas prevent v1 records from being reused.

An explicit canonical `--probe-script`, `LC_ALL=C`, and deterministic
diagnostic flags select the bounded configure-compile probe path. Its script,
compiler, source, platform, semantic flags, and whole-root content bind
distinct success-object and failure-result keys. A failed compile publishes
a typed result only after a complete depfile and bounded diagnostics (at most
262,144 bytes per stream) are verified; a hit revalidates those inputs,
the exact script/compiler bytes, artifact paths, and marker before replaying
the exit status and both diagnostic streams. Unknown manifests, oversized
diagnostics, and unclassified invocations run the real compiler instead of
replaying a guessed failure. Arbitrary non-probe compile failures are not cached.

The driver receipt remains `mantle-cc-driver-receipt-v1`; normal dispositions
are `hit`, `published`, `compiler-only`, `compiler-failure`, `fallback`, and
`rejected`. Explicit probes additionally use `probe-failure-hit`,
`probe-failure-published`, or `probe-compiler-failure`, and record each attempted
failure/object `manifest` or `read` as `hit`, `miss`, `invalid`, or `unavailable`.
These are acceleration receipts, not strict proof evidence.

The cache/socket mapping is execution-time machinery, never a derivation input or hash-changing environment binding. Fixed-point and release proofs MUST explicitly exclude the cache or independently verify content identity; their transcripts MUST record cache mode. Receipts document acceleration, not proof of a rebuilt stage. A writer with access to this per-user cache can inject object code. A content digest detects accidental corruption or mismatch against the cached record, not malicious publication or compiler correctness.

## Local verification (not StageX acceptance)

The final isolated driver BLAKE3 is
`b28c2cd5efd88e236de5233b82acf80ae4d3292eefc08b50c6a780ad99598f80`.
The standalone daemon admitted that executable and the real GCC/G++ compiler
executables under on-mode policy. Direct compiler, cold published, same-root
hit, and rebuilt-identical relocated hit artifacts matched byte for byte:
GCC retained object BLAKE3
`cd9629ace6e497ba5563848f2d77321c36eb6a317a206d3e5e15921febb58e30`
and action key
`fa57df779497d360888b07242fc5dba2b16ca1aa5772b7482f158fec900b4508`;
G++ retained object BLAKE3
`3f886902a41fbf4ed66429ceba4c6b35284ec0c6711249d1100484cb9198fadf`
and action key
`57acef584f7cf975f3576b81c439a370aa6cc3e28b887328aad8b27e5bbc058c`.
Each relocated depfile matched the direct compiler at that root, rather than
reusing the original root's path bytes. Changed C header content published
a different object and key; a z-before-a include fixture also reproduced
the direct GCC prerequisite order and 72-column wrapping.

Real daemon classified-probe failure cold publication and subsequent replay
matched native GCC exit status, diagnostic streams, and depfile, including a
hit after daemon restart. Script, dependency content, platform, flags, and
GCC-to-G++ compiler changes each produced distinct published failure keys.
Missing dependency manifests and diagnostics exceeding 262,144 bytes were
not replayed; oversized failures still streamed the full native diagnostic.
A classified successful configure compile published then hit with exact
native object and depfile bytes. Without its required deterministic diagnostic
flags, a success smoke forwarded as
`fallback/unclassified-probe-environment`, not as a guessed cache hit.
These are local compile/probe observations only, not representative bootstrap
or fixed-point proof evidence.

## Consequences

Reusing the daemon avoids a second local authority and keeps builds functional without daemon availability, but requires wire compatibility, bounded manifest learning, and protected-inventory validation. An identical-content dependency under another store path can reuse an object, whereas changed root-tree or dependency content must miss. Until a timed uncached baseline, a representative cache-on/off byte comparison, and an actual cache-off fixed-point/release evidence pass are recorded, no speedup, output-equivalence, or proof-completion claim follows from this ADR or from the typed policy.
