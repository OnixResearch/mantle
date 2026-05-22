## MODIFIED Requirements

### Requirement: GCC version ladder

The system MUST keep GCC 4.0 native-frontier evidence fail-closed as bounded diagnostic evidence until native GCC correctness is proven.

#### Scenario: v22 auto-host ssize frontier receipt is required

- GIVEN the GCC 4.0 source-frontier reduction receipt
- WHEN parity validates the GCC 4.0 row
- THEN it requires schema `mantle-gcc40-native-cc1-source-frontier-reduction-v22`
- AND it requires markers for the direct generated `auto-host.h` `ssize_t` seam probe
- AND it records whether the focused real `make -C gcc c-parse.o` target advances beyond the include-flood frontier
- AND it does not promote GCC 4.0 beyond partial/frontier status
