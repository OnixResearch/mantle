# Wasm Component Consumer Verifier Specification Delta

## MODIFIED Requirements

### Requirement: File verification uses an explicit capability root

r[mantle.wasm_consumer_verifier.file_shell] The standard-library verifier MUST resolve every declared logical member beneath one explicitly opened directory capability, MUST remeasure exact identity and length under named bounds, and MUST reject escapes, parent traversal, descendant symlinks, special files, changed file types, and member substitution. Source-closure MAY be an admitted directory tree; all other roles MUST remain regular files. A logical path shared by roles MUST declare both identical digest and identical length, and MUST NOT grant directory acceptance to a non-source role.

#### Scenario: Every member matches under the root

- GIVEN a structurally accepted complete bundle whose regular files and admitted source-closure tree match declared lengths and identities under the root
- WHEN materialized-byte verification runs
- THEN every declared member MUST have a matching observation
- AND the published schema and fixed non-claims MUST remain unchanged.

#### Scenario: A member escapes or changes

- GIVEN a member escapes the root, traverses a descendant symlink, changes kind, exceeds a bound, or has different bytes
- WHEN materialized-byte verification runs
- THEN the shell MUST NOT emit a passing materialized-byte report.

## ADDED Requirements

### Requirement: Source-tree measurement preserves producer v1 identity

r[mantle.wasm_source_closure_trees.identity] Admitted directory source closures MUST use the exact unchanged producer v1 BFS order, native direct-entry filename sorting, directory and file tags, u64 little-endian UTF-8-lossy relative-path byte-length framing, u64 little-endian file-length framing, and exact content bytes. Empty directories MUST participate; the root MUST NOT add a record; declared size MUST count file content only. The verifier MUST NOT archive, flatten, rename, or rewrite bundle member paths or identities.

#### Scenario: Real producer-shaped tree matches

- GIVEN the producer copied a source tree containing nested and empty directories, regular content, and admitted native filenames
- WHEN the published verifier measures the unchanged source closure
- THEN the source observation MUST match the producer's exact digest and size.

#### Scenario: Tree identity drifts

- GIVEN file bytes, native name, empty directory membership, actual size, or declared size changes
- WHEN source verification runs
- THEN that source member MUST NOT match
- AND the materialized-byte report MUST NOT be verified.

### Requirement: Member resolution retains no-follow directory capabilities

r[mantle.wasm_source_closure_trees.capabilities] The shell MUST retain opened directory capabilities rather than mutable ambient root strings. Explicit caller root opens MAY follow the supplied root, including `/proc/self/fd` bridges. Descendants MUST be descriptor-relative and no-follow. Regular-file opens in both Mantle and shared observation/copy MUST be nonblocking and recheck opened kind and length so FIFO substitution cannot block member opening. The shell MUST NOT perform ambient source reopens, registry, network, or store search.

#### Scenario: Ambient root is retargeted after open

- GIVEN source and destination directory capabilities have been opened
- WHEN their ambient paths are replaced or retargeted through a symlink
- THEN subsequent operations MUST retain the originally opened roots
- AND MUST NOT acquire authority over the replacement roots.

#### Scenario: Descendant is a link or special node

- GIVEN a leaf, parent, or closure-root descendant is a symlink, FIFO, socket, or unsupported node
- WHEN verification or copying observes it
- THEN it MUST fail closed without a matching source observation.

### Requirement: Source-only copying is capability rooted and create-new

r[mantle.wasm_source_closure_trees.copy] The published `copy_source_closure` API MUST measure and copy only the declared source closure into a newly created matching logical path beneath a caller-opened destination capability without clobbering any existing target. It MUST preserve empty directories and exact file bytes, use bounded-tree prepared observation/execution/revalidation for trees, and return role source-closure, the matching declared digest and size, and Some(Matched) only after identity and bounds match. Unavailable, unsupported, mismatched, over-bound, and destination failures MUST be typed. The caller MUST own private destination cleanup on failure. This observation MUST NOT claim whole-bundle structural admission, publication authority, durability, or an atomic source snapshot.

#### Scenario: Exact private copy succeeds

- GIVEN a supported matching source closure and an empty destination logical path
- WHEN the helper succeeds
- THEN producer remeasurement of the copied tree MUST match the original declaration
- AND unrelated bundle members MUST NOT be copied or required by this source-only operation.

#### Scenario: Target exists or traverses a link

- GIVEN the destination target exists or a destination parent is a symlink
- WHEN source copying runs
- THEN the operation MUST fail
- AND MUST NOT overwrite existing content or write through that link.

### Requirement: Source-tree resources and supported paths are explicit

r[mantle.wasm_source_closure_trees.bounds] Tree observation, copying, and revalidation MUST reuse pinned bounded-tree mechanisms rather than another independent general-purpose walker. They MUST enforce at most 10,000 entries, directory depth 64, and 536,870,912 aggregate file content bytes. Files directly under a depth-64 directory MAY have 65 components. Shared paths MUST be bounded to 65 times 256 bytes and MUST fail closed on unsupported shared lexical shapes, including backslash-containing components, without rewriting native bytes. Regular files MUST be streamed with bounded buffers and an EOF probe. This policy MUST NOT be described as accepting every producer-native filename.

#### Scenario: Depth boundary is respected

- GIVEN a depth-64 directory containing a regular file
- WHEN the producer and consumer measure it
- THEN the unchanged admitted tree MUST match
- AND a directory at depth 65 MUST fail closed.

#### Scenario: Count, size, or path policy is exceeded

- GIVEN an entry/count/content bound is exceeded or a native path contains a shared-unsupported backslash component
- WHEN verification or copying runs
- THEN no passing source observation MUST be returned
- AND no substitute identity or normalized filename MUST be invented.

### Requirement: Provider publication includes actual pipeline and consumer evidence

r[mantle.wasm_source_closure_trees.pipeline] Provider acceptance MUST exercise an actual unchanged Mantle pipeline bundle through the published verifier and source-copy API, preserve regular-file fixture compatibility, and record exact commands, immutable source identities, results, and limitations. Mantle's Wasm producer/verifier maintainers MUST own identity maintenance; Molten's private-snapshot intake MUST consume the same admitted pinned tree and complete member agreement. Passing evidence MUST preserve fixed non-claims and MUST NOT claim runtime adoption or cutover, actor/extension installation, effect linking, deadlines, rollback, correctness, runtime authority, or release eligibility.

#### Scenario: Unchanged production bundle is consumed

- GIVEN a real successful pinned pipeline publishes its original materialization bundle and directory source closure
- WHEN the consumer verifies it and producer-remeasures the private source copy
- THEN the materialized-byte report MUST be verified with all declared members matched
- AND copied source digest and size MUST equal the original declaration.

#### Scenario: Evidence is not yet executed

- GIVEN implementation and test code exist without successful recorded commands
- WHEN publication readiness is assessed
- THEN unexecuted checks MUST remain incomplete
- AND structural validation alone MUST NOT be promoted to provider or runtime adoption evidence.

### Requirement: Published file and report compatibility is retained

r[mantle.wasm_source_closure_trees.compatibility] The repair MUST preserve existing file-valued source-closure acceptance, regular-file-only non-source roles, exact per-role digest/length agreement, structural delegation, report schema, ordered blockers, path/payload redaction, and fixed non-claims. Identical file paths shared by roles MAY be measured once; conflicting digest OR length MUST be rejected before member access. The source-copy API MUST NOT grant directory acceptance to another role or reinterpret a source-only observation as a complete bundle report.

#### Scenario: Existing file-only bundle is consumed

- GIVEN an existing complete valid regular-file bundle, including its file-valued source closure
- WHEN verification and source-only copy run
- THEN matching file bytes MUST retain their original unframed identities
- AND report fields and non-claims MUST remain compatible.

#### Scenario: Shared path conflicts or acquires a directory role

- GIVEN two declarations share a path with differing digest or length, or a directory source path also declares a non-source role
- WHEN verification or source copying runs
- THEN that substitution MUST NOT produce a matching report or successful source copy.
