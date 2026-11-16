# Triage labels

| Canonical role | GitHub label | Meaning |
|---|---|---|
| needs-triage | needs-triage | Maintainer needs to evaluate the issue |
| needs-info | needs-info | Waiting on reporter for more information |
| ready-for-agent | ready-for-agent | Specified for agent implementation, subject to activation authority |
| ready-for-human | ready-for-human | Requires human implementation |
| wontfix | wontfix | Will not be actioned |

Use these exact strings whenever a skill names a canonical triage role. Readiness assessment and work activation are separate. Applying `ready-for-agent` must satisfy `WORK-1` and `OPS-1`, and `FID-1` for rules-fidelity work; unattended execution also requires `AGENT-4`. The label itself grants neither implementation nor publication authority.
