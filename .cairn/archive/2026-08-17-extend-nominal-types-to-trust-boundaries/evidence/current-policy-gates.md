# Nominal trust-boundary current-policy gates

## Scope

These commands use the current generated policy from the sibling Cairn repository. They validate the scaffold against the current Cairn schema. They do not replace validation with Mantle's checked-in policy.

## Repository validation

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Result tail:

```text
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}
```

## Proposal gate

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal extend-nominal-types-to-trust-boundaries --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Result tail:

```text
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/proposal.md",
      "substantive_lines": 27
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
```

## Design gate

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design extend-nominal-types-to-trust-boundaries --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Result tail:

```text
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
```

## Tasks gate

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks extend-nominal-types-to-trust-boundaries --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Result tail:

```text
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/tasks.md",
      "substantive_tasks": 26,
      "task_done": 0,
      "task_todo": 26
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
```

## Remaining blocker

The default Mantle policy path still fails before lifecycle inspection because `cairn-policy/generated/cairn-policy.json` lacks `nominal_identity_policy`. No implementation task is complete.
