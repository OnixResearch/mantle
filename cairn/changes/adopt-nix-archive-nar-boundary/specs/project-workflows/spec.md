## ADDED Requirements

### Requirement: Recursive project hashes use the shared filesystem NAR adapter

r[project_workflows.nix_archive_recursive_hashing] Mantle MUST compute recursive project source hashes through the shared filesystem NAR adapter for each supported Nix hash algorithm. Flat file hashing MUST remain outside NAR encoding. A failed recursive observation MUST NOT update lock state or generated project inputs.

#### Scenario: A recursive source hash succeeds

GIVEN a project source tree is valid under the selected byte, mode, symlink, case-hack, and size policy
WHEN refresh computes its recursive hash
THEN Mantle MUST encode the tree through the shared adapter and return the requested digest
AND the result MUST match the accepted NAR parity corpus for that algorithm.

#### Scenario: A flat source hash is requested

GIVEN a project input requires a flat content hash instead of a recursive NAR hash
WHEN refresh computes the input hash
THEN Mantle MUST keep the existing flat hash path
AND it MUST NOT wrap the file in NAR encoding.

#### Scenario: Recursive observation fails

GIVEN the tree is unreadable, changes during observation, exceeds a named limit, uses unsupported platform behavior, or fails parity policy
WHEN refresh processes the input
THEN it MUST report that input as failed without substituting a manifest value
AND it MUST NOT write a new lock result or generated binding for the failed input.
