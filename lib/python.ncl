# Offline Python/uv adapter. Acquisition is deliberately outside this recipe:
# use mantle.fetchurl/fetchTarball/fetchGit and the generic PackageMirror bundle.
let mantle = import "lib.ncl" in
let unique = fun paths =>
  std.array.fold_left (fun seen path => if std.array.elem path seen then seen else seen @ [path]) [] paths
in
fun { name, adapter, config, python, uv, project, artifacts,
      native_inputs, runtime_inputs, git_sources ? [] } =>
  let native = {
    inputs = std.record.map (fun _role path => path | mantle.StorePath) native_inputs.inputs,
    env = std.record.map (fun _key value => value | String) native_inputs.env,
    path = native_inputs.path | Array String,
  } in
  # Nickel records are recursive: `name = name` would refer to the field itself.
  let derivation_name = name in
  let roots = unique (
    ([adapter, config, python, uv, project] | Array mantle.StorePath)
    @ (artifacts | Array mantle.StorePath)
    @ (std.record.values native.inputs)
    @ (runtime_inputs | Array mantle.StorePath)
    @ (git_sources | Array mantle.StorePath)
  ) in {
    name = derivation_name,
    builder = adapter,
    system = 'x86_64-linux,
    outputs = ["out"],
    addressing_mode = 'input-addressed,
    sandbox = 'native,
    env = {
      MANTLE_PYTHON_CONFIG = config,
      MANTLE_PYTHON_NATIVE_INPUTS = std.serialize 'Json native,
      MANTLE_PYTHON_RUNTIME_INPUTS = std.serialize 'Json runtime_inputs,
      MANTLE_PYTHON_INPUTS = std.serialize 'Json roots,
    },
    inputs = roots,
  } | mantle.Derivation
