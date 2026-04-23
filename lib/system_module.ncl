let NameArray = Array String in
let OutputRecord = {
  exports | default = null,
  providers | { _ : Dyn } | default = {},
  nixos | default = null,
  files | default = null,
} in

{
  interface | {
    roles | { _ : Dyn },
  },
  impl | default = fun _args => { output = {} },
  inputs | NameArray | default = [],
  consumes_providers | NameArray | default = [],
  produces_providers | NameArray | default = [],
  priority | Number | default = 1000,
  output | OutputRecord | default = {
    exports = null,
    providers = {},
    nixos = null,
    files = null,
  },
}
