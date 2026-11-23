# Rules and verification

This foundation specializes `PURP-2`, `WORK-2`, `EVID-3`, `EVID-4`, `REP-2`, `OPS-1`, `FID-1`–`FID-4`, `SIM-1`, `SIM-4`, and `INT-1` for a published board game. It defines the required source boundary; it does not certify an engine or substitute for the rules.

## Source authority

The owner selected GMT's current official Cuba Libre rules and applicable official corrections, pinned at acceptance. On 2026-11-14 GMT's fifth-printing product page links the March 2019 third-printing Living Rules and a third-printing playbook. The local resources contain that named rules edition, a second-printing playbook, third-printing faction aids, and an operations reference. The exact local hashes and official links are in [founding evidence](../research/founding-evidence.md). To read rule text, extract `~/Pictures/Cuba Libre Resources/resources/documents/rules-3rd-printing-living-march-2019.pdf` with `pdftotext -layout` into a scratch file and search for the rule number. This selects the source policy; it does not claim that the mixed local collection has been fully reconciled.

Before implementation, reconcile those editions and pin the actual controlling files, card text, corrections, and source precedence for the selected scope. Later official revisions are proposals to update the baseline, never silent runtime changes. An old errata sheet must not override a later corrected rule merely because it is labeled errata. The supplied Workshop scripts, object names, card crops, and artwork are reference/identity evidence, not a competing rules authority.

Inspect official sources before asking the owner to resolve a rule. An unresolved interpretation must be identified as such, with the source conflict and recommended reading. A local ruling can be accepted as a variant but cannot be called verified official fidelity. Unsupported states and choices must fail explicitly, without guessed bookkeeping or fabricated legal actions.

## Obligations that the feasibility work must settle

The source-derived scope must cover setup and deck construction; map spaces, adjacency, terrain and economic centers; piece availability and states; control, support/opposition and resources; faction order, eligibility, passing and the first/second-faction action restriction; operations and special activities; event precedence and persistent capabilities; transfers and negotiation timing; Propaganda phase order and ending conditions; faction victory and tie handling. Inspect interactions and exceptions, not only counts of cards or named actions.

This is an initial reading map, not an accepted complete fidelity topology. The first rules issue must derive the topology and independent difficult cases from the selected source set before adopting a durable rules representation. In particular, eligibility, special activities, event overrides, resource transfers, and Propaganda transitions can alter otherwise plausible actions. Source-defined optional rules and Non-Player procedures must be explicitly included or excluded rather than accidentally inherited from a script.

## Engine authority

Rust rules code owns legal actions and consequences. Human controls and agent sockets submit decisions through the same checked boundary. Presentation, narrative, and agents cannot move pieces, award resources, alter eligibility, or bind agreements by describing an outcome. Choice identities and the match revision must bind a submission to the state the player inspected; stale or unauthorized requests do not mutate it.

Routine bookkeeping is automatic; actual player choices remain choices. Multi-step operations must expose the remaining choices, costs, restrictions, and deliberate completion/cancellation options that the rules allow. Legal-action discovery must remain complete for the supported decision, even if presented hierarchically; a short list of recommended moves is not the legal set.

On 2026-11-24 the owner selected, as a binding constraint under `PURP-2`, where a player chooses a map space for an Operation: on the map, and nowhere else. This binds every Operation whose decision selects a space. The map shows each space's state for the open decision, read from the rules authority's own choices, and choosing a space there opens what the authority offers in it. The decision panel holds what is not a place: costs, steps that select no space, the ending, and the record of spaces already resolved. It lists no space as a control. The keyboard reaches the same spaces.

For Assault the owner ruled that a space where it would remove nothing stays choosable, as completeness above requires, and is shown as quietly as a space no choice acts on (R 3.2.4, 4.1). Special Activities and Events are not bound: the owner left Kidnap's space to be judged as more of the game is implemented. [HIGHLIGHTS.md](../../art/atlas/HIGHLIGHTS.md) specifies the selected map states and the space popover, first used by the 26July Terror Operation.

Save/resume must preserve unresolved decisions and randomness, so that restarting the same build continues honestly: no reroll, and no decision lost or applied twice. A save binds the accepted rules source. Art is admitted at launch, before any save opens, and is not part of a save's identity. Reuse existing mechanisms before choosing a new format. Diagnostic reproducibility is an engineering capability, not a commitment to a public replay product.

On 2026-11-23 the owner made saves disposable across builds, specializing `WORK-2` and `REP-2`: until the owner records here that this has ended, which is expected once the standard four-faction rules are complete, a save written by a build of another commit carries no promise. No work may add or keep code, data or tests whose purpose is to open such a save. A build opens a save only when it can continue it honestly; otherwise it refuses the save visibly and leaves every file as it was. [ADR 0003](../adr/0003-a-save-opens-by-exact-replay.md) records the save model this permits.

## Evidence for the claim

Rules expectations come from independently read sources and worked positions, not the implementation's own output. Tests must exercise source-derived interactions, refusals and invariant preservation through the actual rules path. Agent checks must demonstrate valid selection, real consequence, updated observation, and a subsequent decision affected by that consequence. Prompt review alone is insufficient.

A native slice must include the actual authored art, readable state and choices, and a complete bounded player task. First exercise and inspect it through the required [socket playtesting seat](03-engineering-and-reuse.md#required-agent-operated-playtesting), including actual rendered screenshots. Separately test save/resume after process restart and agent disconnect/rejoin, with no duplicated move or stale command accepted. A headless rule test establishes only the behavior it executes; the owner judges subjective usability, thematic communication, and enjoyment after the agent has completed feasible functional and visual verification.
