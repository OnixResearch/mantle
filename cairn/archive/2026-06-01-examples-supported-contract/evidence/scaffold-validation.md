# Scaffold validation

Change: `examples-supported-contract`

## cairn validate --root .

```text
this derivation will be built:
  /nix/store/rffpi4yqjhsnfim0bwaj07v8r9i3x6si-cairn-0.1.0.drv
cannot build on 'ssh-ng://root@10.10.10.1': error: failed to start SSH connection to '10.10.10.1'
building '/nix/store/rffpi4yqjhsnfim0bwaj07v8r9i3x6si-cairn-0.1.0.drv'...
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 9,
  "valid": true
}

```

Exit: `0`

## cairn gate proposal examples-supported-contract --root .

```text
this derivation will be built:
  /nix/store/06qdzs98gn5zygv4r84hf4w56ls4xl5a-cairn-0.1.0.drv
cannot build on 'ssh-ng://root@10.10.10.1': error: failed to start SSH connection to '10.10.10.1'
building '/nix/store/06qdzs98gn5zygv4r84hf4w56ls4xl5a-cairn-0.1.0.drv'...
{
  "change": "examples-supported-contract",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "8775df1934445ebe2bd88f65b9f93bf51d7c2d6c49498aa2228585fbfe9cf907",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4b8f28668bb79615a3c0fa35a25934f964295b4783f25e3118c27209f0c331c9",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn gate design examples-supported-contract --root .

```text
{
  "change": "examples-supported-contract",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "3e74ae6a95cc58e990d081f8623cb6b19b7f47a01bf6f3a983943273f5c52eef",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "a272f4df3e2c083b9c62734ae7c5e4c6567de3f75cbac420351ce98307b5b26d",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn gate tasks examples-supported-contract --root .

```text
{
  "change": "examples-supported-contract",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a9568b1995d8717a201815f84cbe3a178fd8810aa1bffd8cb44c593271f7261a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "984fb2869f46b1fd05b08d3def9b6b6d4499696919be31efb463fb5d973baf3f",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

