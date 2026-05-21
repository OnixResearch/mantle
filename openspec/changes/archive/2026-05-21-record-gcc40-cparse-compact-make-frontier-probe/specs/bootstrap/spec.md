## MODIFIED Requirements

### Requirement: GCC version ladder
The bootstrap parity map SHALL keep GCC 4.0 source-frontier evidence partial unless native source-build correctness is proven.

#### Scenario: GCC 4.0 c-parse compact make frontier is accepted
- **GIVEN** a `gcc.4.0` frontier receipt with schema `mantle-gcc40-native-cc1-source-frontier-reduction-v8`
- **AND** the receipt records the focused `c-parse.o` make-error boundary and compact diagnostic marker contract
- **WHEN** bootstrap parity evaluates the GCC 4.0 row
- **THEN** the row remains evidence-backed `partial`
- **AND** the compact diagnostic derivation markers are checked against `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- **AND** no native GCC 4.0 compiler/source-build correctness is claimed.

#### Scenario: stale pre-v8 c-parse source-frontier evidence is rejected
- **GIVEN** a `gcc.4.0` frontier receipt using a pre-v8 source-frontier schema or missing the compact diagnostic marker contract
- **WHEN** bootstrap parity evaluates the GCC 4.0 row
- **THEN** the row fails closed instead of silently accepting stale diagnostic/source-frontier evidence.
