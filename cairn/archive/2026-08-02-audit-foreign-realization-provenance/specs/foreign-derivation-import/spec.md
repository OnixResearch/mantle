## ADDED Requirements

### Requirement: Castore-backed foreign provenance audit

r[foreign_derivation_import.castore_provenance_audit] Mantle MUST generate foreign output provenance observations from admitted signed PathInfo and castore content. The production scanner MUST NOT trust ambient exported paths or caller-supplied observations alone.

#### Scenario: Complete closure is scanned from castore

GIVEN a realized foreign root has signed PathInfo and complete castore content
WHEN the provenance audit scans the root
THEN it MUST walk the admitted closure with bounded deterministic traversal
AND every observation MUST bind the exact PathInfo, node, blob, path-map, and policy identities used.

#### Scenario: Incomplete castore closure fails audit

GIVEN a required closure node or blob is absent, corrupt, unsigned, or inconsistent with PathInfo
WHEN the audit reaches that object
THEN it MUST record a stable incomplete-closure finding and fail the audit
AND it MUST NOT read an ambient host path as a replacement.

#### Scenario: Traversal limit fails closed

GIVEN scanning exceeds a configured node, blob, byte, depth, finding, container, or recursion limit
WHEN the limit is reached
THEN the audit MUST stop with a stable limit finding
AND it MUST NOT report a passing or complete scan.

### Requirement: Explicit executable payload classification

r[foreign_derivation_import.executable_payload_classification] Mantle MUST classify every executable or executable-containing payload under a bounded reviewed class. Unknown executable content, untranslated foreign references, and unresolved executable targets MUST fail the provenance audit.

#### Scenario: ELF reference resolves inside closure

GIVEN an executable ELF payload contains recognized store references
WHEN the pure classifier and resolver inspect the payload
THEN every required target MUST resolve to an admitted exact target closure path
AND any declared foreign store path or missing target MUST produce a failed finding.

#### Scenario: Script shebang resolves inside closure

GIVEN an executable script has a bounded valid shebang
WHEN the classifier resolves its interpreter
THEN the interpreter MUST be an admitted target path or an explicitly allowed profile path
AND a foreign, relative, malformed, missing, or escaped interpreter MUST fail the audit.

#### Scenario: Symlink remains within admitted content

GIVEN a symlink appears in the realized closure
WHEN the resolver evaluates its target
THEN the target MUST remain within the admitted logical closure rules
AND an absolute foreign target, path escape, loop, or missing target MUST fail the audit.

#### Scenario: Container payload is inspected with bounds

GIVEN a supported archive or initrd contains executable entries
WHEN the scanner inspects the container
THEN it MUST classify those entries under configured recursion and expansion limits
AND unsupported containers or hidden unclassified executables MUST fail the audit.

#### Scenario: Unknown executable bytes fail closed

GIVEN a regular file has executable mode but matches no accepted payload class
WHEN classification completes
THEN it MUST produce an unclassified-executable finding
AND it MUST NOT treat the bytes as harmless data through a text heuristic.

### Requirement: Deterministic foreign provenance audit receipt

r[foreign_derivation_import.provenance_audit_receipt] Mantle MUST emit `mantle-foreign-provenance-audit-v1` for each requested foreign output audit. The receipt MUST report exact scope, observations, limits, findings, disposition, and non-claims.

#### Scenario: Passing audit reports bounded provenance facts

GIVEN every scanned object is complete, classified, and free of unresolved foreign references
WHEN the audit receipt is emitted
THEN it MUST bind realization receipt, root and closure identities, path-map digest, profile digest, scanner policy, limits, observation digest, and passing disposition
AND the strongest state MAY be `provenance-audited` for that bounded scope.

#### Scenario: Failed audit does not erase realization evidence

GIVEN a realized output produces one or more failed audit findings
WHEN Mantle reports the result
THEN the realization receipt and build report MUST remain unchanged as execution observations
AND the strongest state MUST remain realized with audit failure rather than provenance-audited.

#### Scenario: Audit preserves explicit non-claims

GIVEN a foreign output passes the provenance audit
WHEN an operator reviews the receipt
THEN compiler correctness, source correctness, package correctness, runtime behavior, bootstrap parity, reproducibility, OS bootability, deployment safety, and release eligibility MUST remain non-claims
AND the receipt MUST state that static bounded classification does not prove those properties.

#### Scenario: Equivalent observation order is deterministic

GIVEN two scans observe equivalent closure content in different service or traversal order
WHEN each audit is canonicalized
THEN their observation digest, finding order, and disposition MUST match
AND temporary paths, service iteration order, and wall-clock values MUST not affect identity.
