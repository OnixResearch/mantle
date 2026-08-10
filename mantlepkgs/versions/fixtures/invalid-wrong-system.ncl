let Contracts = import "../contracts.ncl" in
let cohort = import "cohort-base.ncl" in
(cohort & { system | force = "aarch64-linux" }) | Contracts.Cohort
