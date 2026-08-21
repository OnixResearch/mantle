let Contract = import "../../../contracts/mantle-build-contract.ncl" in
let Fixtures = import "../producer/fixtures-v1.json" in
let replace = fun key value record => std.record.insert key value (std.record.remove key record) in
replace "schema" "mantle.build-request.v2" Fixtures.request | Contract.request
