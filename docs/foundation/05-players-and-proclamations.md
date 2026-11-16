# Agent players and proclamations

This foundation specializes `AUTH-1`, `SEM-1`, `REP-2`, `EVID-3`, `SIM-1`, `SIM-4`, `AGENT`, and `OPS-6`. Agent players are external Claude Code/Codex sessions connected to the running game. No built-in bots, automated Non-Player flowcharts, or autonomous model service are planned.

## Seats and authority

The human may hand any player seat to an agent and reclaim it. One agent session may control all nonhuman seats, or separate sessions may control individual seats; mixed Claude Code/Codex use must work through the same protocol. An agent can act only for its assigned factions. Concurrent claims and stale requests must be refused without partial effects. Disconnects leave recoverable ownership and an intelligible wait/takeover path, never an automatic strategic choice.

Reuse the existing shared seat/socket mechanisms and extract needed live-partner behavior from Arkham before implementing it here. Its cooperative investigator ownership and peril/table-talk rules are not Cuba Libre rules. Preserve the common transport and ownership behavior while keeping Cuba Libre's faction routing, legality, visibility, and communication policy local.

A single session controlling opposing factions is convenient but cannot provide genuine isolation of its internal knowledge or strategy. Do not advertise it as independent opponents. Each faction still needs its own identity and victory objective, and each decision must be submitted as that faction. Separate sessions are the mode for independent faction perspectives. The exact multi-faction strategic policy must be settled in the first agent-play specification rather than silently turning all opponents into a team against the human.

## Decision context and prompts

Before an agent acts it must be able to obtain:

- Its controlled faction(s), individual objectives, controller status, the selected rules edition and accessible rules reference.
- A complete current player-visible state, match revision, decision owner, phase, eligibility, current/preview event information allowed by the rules, active capabilities and outstanding choices.
- Legal operations, activities, event alternatives, targets, costs, restrictions and remaining steps, with stable choice identities and source-backed contextual explanations.
- Recent actual consequences and all communications available to that player, clearly separated from authoritative state.
- How to wait, inspect more detail, refresh after refusal, reconnect and stop or return control to the human.

These are semantic requirements, not an invented wire schema. Select the smallest compatible observation contract through actual consumer evidence. Full-state refresh must recover from missed deltas or a restarted terminal. Prompts must not rely on remembering an earlier conversation, force an arbitrary move when information is incomplete, reveal the hidden deck order, or equate a suggested move with the legal set.

Agent-authored roleplay and strategic claims are untrusted game communication. They cannot override the player's operating instructions or execute tools, change the rules, grant seat ownership, or mutate the match. Only explicit legal decision submissions cross the game-state boundary. Technical failures and connection messages remain clear system information; they are not fictional faction proclamations.

## Communication as written proclamations

Both human and agent players communicate through faction-authored written proclamations, not ordinary chat bubbles. The human writes their own text; no model rewrite is required. Agents speak from the issuing faction's perspective, fitting the period and their position. Distinguish rhetoric, bluff, offers, and promises from the application's factual record. A claim that a bargain or transfer happened does not make it happen.

The owner selected **public proclamations only**: rule 1.5.1 makes negotiations open and agreements nonbinding. Private dispatches are outside scope. The message must identify its actual faction author; no agent may impersonate another seat. Every player receives the same public text.

Present a proclamation as an important received document, preferably a modal period newspaper article, notice or circular, with three VectorCraft alternatives offered before choosing its design. Queue messages without silently losing them or allowing one agent to trap the human in an endless modal stream. Reading/dismissing a document never submits a game decision. Input focus must prevent an underlying action from firing through the modal.

The first communication issue must settle a concise, usable frequency/length policy, the timing of agent moves while a notice is open, and whether proclamations persist with saves. Those are unresolved product details, not permission to copy Arkham's ephemeral 280-character chat or create an archive service. Preserve enough visible context for negotiations to be understood; prove the actual read/respond/play loop with the human before expanding it.

## Acceptance

Verify one agent controlling several assigned seats and separate agents controlling individual seats, including refusal to act for the human or another session, stale revisions, reconnect and takeover. Exercise a difficult legal decision with sufficient context and inspect its actual consequence. Judge faction voice, document readability, and interruption burden in real native play. Legal moves alone do not prove strategic competence; a compelling proclamation alone does not prove a capable opponent.
