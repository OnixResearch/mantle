# Remote credential nominal-boundary gates

## Scope

These commands use the current generated policy from the sibling Cairn repository. They validate the edited change text against the current Cairn schema. They do not prove implementation.

## Proposal gate

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal harden-remote-credential-boundary --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Result tail:

```text
      "path": "./cairn/changes/harden-remote-credential-boundary/proposal.md",
      "substantive_lines": 23
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
```

## Design gate

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design harden-remote-credential-boundary --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Result tail:

```text
      "path": "./cairn/changes/harden-remote-credential-boundary/specs/remote-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 20,
      "substantive_requirement_blocks": 9
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
```

## Tasks gate

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks harden-remote-credential-boundary --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

Result tail:

```text
      "path": "./cairn/changes/harden-remote-credential-boundary/tasks.md",
      "substantive_tasks": 36,
      "task_done": 0,
      "task_todo": 36
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
```

## Remaining blocker

The default Mantle policy path still fails before lifecycle inspection because `cairn-policy/generated/cairn-policy.json` lacks `nominal_identity_policy`. Credential implementation tasks remain unchecked.
