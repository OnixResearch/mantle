let Adapter = import "external-adapter.ncl" in
let report = import "../../schemas/machine-contracts/fixtures/mantle-package-impact.valid.json" in
Adapter report
