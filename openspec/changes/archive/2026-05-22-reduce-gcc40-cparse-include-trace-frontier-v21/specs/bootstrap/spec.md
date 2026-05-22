## MODIFIED Requirements

### Requirement: GCC version ladder

The system MUST keep GCC 4.0 native-frontier evidence fail-closed as bounded diagnostic evidence until native GCC correctness is proven.

#### Scenario: v21 include-trace frontier receipt is required

- GIVEN the GCC 4.0 source-frontier reduction receipt
- WHEN parity validates the GCC 4.0 row
- THEN it requires schema `mantle-gcc40-native-cc1-source-frontier-reduction-v21`
- AND it requires compact include-trace markers from the c-parse preprocessor probe
- AND it still requires the focused real `make -C gcc c-parse.o` frontier to fail with rc=2 and include-flood evidence
- AND it does not promote GCC 4.0 beyond partial/frontier status
