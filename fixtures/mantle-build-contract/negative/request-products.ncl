let Contract = import "../../../contracts/mantle-build-contract.ncl" in
let Fixtures = import "../producer/fixtures-v1.json" in
let replace = fun key value record => std.record.insert key value (std.record.remove key record) in
replace "required_products" ["metadata", "binary"] Fixtures.request | Contract.request
