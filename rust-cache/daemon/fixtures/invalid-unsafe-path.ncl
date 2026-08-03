let Contracts = import "../contracts.ncl" in
let policy = import "policy.ncl" in
(policy & {
  socket_path | force = "/run/user/1000/../escape.sock",
}) | Contracts.Policy
