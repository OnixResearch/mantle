# Scaffold validation transcript

Change: `onix-module-eval-boundary`
Date: 2026-05-30

## Commands

```sh
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate proposal onix-module-eval-boundary --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate design onix-module-eval-boundary --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate tasks onix-module-eval-boundary --root .
```

## Results

`cairn validate --root .`:

```json
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
```

Proposal gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "3a14a1b69ff1856d0512886ce792612010e723269fab7197e5100c9042fbc274",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "26a6f79a553b64918e25f5eb1fae98c461bb7544eb9fa2db4aa7a1d12478b54b",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

Design gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "5823c2c346755d525a91f84de802cfe147b127f495a146b44f5b58fcd1c9072d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "18b8c15d72915ac2a69449eb657a1e078ebaa19cda27eb2fd904779eefe245c3",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

Initial tasks gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "3d2ebb0fb56a71609435c3659dc3162b4b588e9fd79a07453b1e2b4c29ceaa36",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "f31554f9d22a2834edc541605e481b805ca4876885f9b761e83aa3c49ac0ea51",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Post-task-evidence update validation:

```json
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
```

Post-task-evidence update tasks gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "2bfc081535111583ed9630df2e9570fb8b6ec066cf64231e486b18468f8ba4a0",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c82d83db0af2f4745a5b226c93746b5084e9045200311f16d72c253900e381ed",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Mantle-owned module runtime wording update

After sharpening the boundary to state that Mantle owns the generic module runtime and Onix owns frontend lowering, reran validation and all gates:

```json
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
```

Proposal gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "6cd1c69e8bd1ffd61d092ccf5e055d7d193fe8363d2c2201550bee07cfe7714b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "cb36197c2d3a62eeda111c162fec4790698731fded36aea2b218e485aab617cc",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

Design gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "53e3c19e64ff33373bd2a5e1c5575bbf32b0bb21b2251b6a71998fff4e2a59e7",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4808e3ccb27d56785fa7ccfc8b9d91efe1e3db8081be3af9b2be9527b166ee82",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

Tasks gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "6bfc9e07d7d249d86e92bc044c42051abd17fad524bef203fc7a446d85fc7e15",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "59521594e5cf4489ffa513c1e45fd0ba1409f16a58ead8d8d34612df627b28d9",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Build-tool-boundary correction

After correcting the design so the module layer sits above Mantle and Mantle remains only a build-tool boundary, reran validation and all gates:

```json
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
```

Proposal gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "2055f5fd3147dc2f52a32901c2bef6c9bfb64a08ac13b64e4090f0202395f068",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "65a288bc28b4b188245ba44b9f8ce28a28f5dd225564f2e2e6b1c9a5f612bdff",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

Design gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "93282a433a24544fb751533cccf85aa531d17b3099c932eac269b171de8f6ba0",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "318d3c4d2c5fb9b5c9903f60ed8e161fccfa33b0e0bc5a53586f8fd301b13fa5",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

Tasks gate:

```json
{
  "change": "onix-module-eval-boundary",
  "input_hash": "90853842098c36ef0ffa3c20f43dbdc217385af8305d7a14e68585d90cd0c074",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0c245d6e6d079572a2cc677f1ba54886bd1003da5be498eb52f4801d77a1de52",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
