let MachineRecord = {
  system | String,
  class | String | optional,
} in

let InstanceRecord = {
  machine | String,
  role | String,
  settings | optional,
  tags | Array String | default = [],
} in

let ServiceRecord = {
  instances | Array InstanceRecord,
} in

{
  machines | { _ : MachineRecord },
  services | { _ : ServiceRecord },
}
