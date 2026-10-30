import json
import pathlib
import sys
import tomllib

lock = tomllib.loads(pathlib.Path(sys.argv[1]).read_text())
cohort = json.loads(pathlib.Path(sys.argv[2]).read_text())
packages = {package["name"]: package["version"] for package in lock["package"]}
for key, package in [
    ("wasmtime", "wasmtime"),
    ("wasmtime", "wasmtime-wasi"),
    ("wrpc_runtime_wasmtime", "wrpc-runtime-wasmtime"),
    ("wrpc_transport", "wrpc-transport"),
    ("wit_bindgen", "wit-bindgen"),
    ("wit_bindgen_wrpc", "wit-bindgen-wrpc"),
]:
    if packages.get(package) != cohort[key]:
        raise ValueError(f"source lock cohort mismatch for {package}")
print(json.dumps({"schema": "mantle-wrpc-upstream-lock-cohort-v1",
                  "source_revision": cohort["revision"],
                  "selected_packages": {name: packages[name] for name in [
                      "wasmtime", "wasmtime-wasi", "wrpc-runtime-wasmtime",
                      "wrpc-transport", "wit-bindgen", "wit-bindgen-wrpc",
                  ]}}, sort_keys=True, indent=2))
