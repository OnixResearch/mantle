let Contract = import "../../../contracts/mantle-build-contract.ncl" in
let Fixtures = import "../producer/fixtures-v1.json" in
Fixtures.success | Contract.observation
