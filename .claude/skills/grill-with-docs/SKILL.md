---
name: grill-with-docs
description: Review an existing workflow or sharpen a design against the project's docs.
disable-model-invocation: true
---

Name the measurement behind each cost or saving a recommendation rests on, or call it unmeasured.

## Existing workflow

1. Check the documented procedure, the live interface, and the required inputs. Reproduce the reported failure when feasible. Before recommending a path as a default, trace its origin and check who maintains the files and whether they survive a fresh checkout.
2. Give the working procedure and explain the discrepancy in the same round. State the recommended change and name the checkout where the repaired procedure must work. When that checkout is `main`, include PR creation, merge and pull in the original approval request. Ask only for decisions that still require owner authority.
3. Apply the agreed changes when authorized. For an executable repair, follow the implementation sequence below: run focused checks, commit, use the repository’s code-review skill against the intended merge base, then run the final suite on the reviewed candidate. Verify the documented procedure against the interface. At handoff, name the checkout and revision that contain the fix and report what you exercised. [Implementation sequence](../implement/SKILL.md)

## Design

1. Read the current glossary, relevant decisions, and implementation. Verify facts yourself and use concrete scenarios to expose ambiguities.
2. In each round, number the decisions whose prerequisites are settled. Give a recommendation the user can accept with yes. Wait for the answers, then revisit the remaining decisions.
3. When the user confirms the design, capture agreed vocabulary and decisions in their existing homes as authorized. Use an ADR when the decision is costly to reverse, surprising without context, and reflects a real trade-off.

When an owner selects a design and authorizes implementation, follow the [implementation sequence](../implement/SKILL.md). Its committed-candidate code review covers standards and the selected design for merge. Review subsequent changes from the last reviewed commit.
