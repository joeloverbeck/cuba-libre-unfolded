# Coding standards

Read during review: the Standards axis of [code-review](.claude/skills/code-review/SKILL.md) applies these judgement rules, which no check can hold. `CLAUDE.md`, the glossary, the foundations and the agent documents name the rest.

- **Accurate claims.** Rustdoc, documents, pull-request text and done notes say exactly what the code and the evidence hold at this revision. Each statement of what was tried says how it is known: measured in a named run, read in code or a document, or not tried. It keeps the conditions it held under: the mode, the settings, the machine, the count of runs. A count, or an "every", "all" or "none", matches the code. A described behaviour names the test that reaches it or traces it through the code.
- **Decisions, not status.** ADRs, foundations and specs state what is decided and who decided it. What is still to come, and what awaits the owner, stays on the issue or pull request (`GOV-2`). A decision the agents made is attributed to them, not to the owner.
- **Readings.** A constitution rule a record lists as consulted was read whole (`scripts/rule ID` prints it).
- **Exceptions.** A change that relies on an exception to a binding rule quotes the rule's evidence test and the evidence that meets it, and names the alternatives not yet tried.
