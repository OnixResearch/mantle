let Contracts = import "../contracts.ncl" in
let policy = import "../default.ncl" in

(policy & { ambient_host_fallback = true })
| Contracts.AuditPolicy
| Contracts.ExactAuditPolicy
