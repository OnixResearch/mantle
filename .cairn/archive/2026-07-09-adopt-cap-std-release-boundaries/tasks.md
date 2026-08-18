## Tasks

- [x] [serial] Add `cap-std` only to Mantle crates that own filesystem shell/adaptor code. r[mantle.release_provenance.cap_std_boundary.dependency]
- [x] [serial] Introduce typed capability root wrappers for release evidence, witness rebuild, bootstrap, build artifact, and store roots. r[mantle.release_provenance.cap_std_boundary.root_wrappers]
- [x] [serial] Convert targeted path opens to capability-relative operations without changing pure planning cores. r[mantle.release_provenance.cap_std_boundary.conversion]
- [x] [serial] Add positive fixtures for valid relative paths under declared roots. r[mantle.release_provenance.cap_std_boundary.tests.positive]
- [x] [serial] Add negative fixtures for `../` traversal, absolute paths, missing root authority, and symlink escapes. r[mantle.release_provenance.cap_std_boundary.tests.negative]
- [x] [serial] Document the local filesystem-authority boundary and release-evidence non-claims. r[mantle.release_provenance.cap_std_boundary.docs]
- [x] [serial] Run focused release-evidence/build tests plus Cairn validation and gates. r[mantle.release_provenance.cap_std_boundary.validation]
