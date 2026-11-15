# Evidence for the first-slice interview

Read-only investigation on 2026-11-14 against Cuba Libre commit `ad372a95b56bd1aeaf4b4bb7944f9c637c733582`, plus the local glossary and this interview evidence. This supports the owner's requested `grill-with-docs` interview and subsequent `to-spec` synthesis. The published specification is [issue #2: Pin the rules baseline and select the first cumulative tactical slice](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/2). This document is supporting reference evidence, not an active work queue or an accepted rules baseline. GitHub owns work status and acceptance.

All five foundation files and all current research files were read. The GitHub connector returned an empty issue list during this inspection. The checkout was clean before these documentation additions. No executable, gameplay test, native launch, source acceptance, or consumer integration was demonstrated.

Consulted rules: `AUTH-1`–`AUTH-3`, `PURP-1`–`PURP-6`, `WORK-1`–`WORK-6`, `EVID-1`–`EVID-10`, `SEM-1`–`SEM-2`, `REP-1`–`REP-3`, `BOUND-1`–`BOUND-5`, `INT-1`–`INT-4`, `AGENT-1`–`AGENT-5`, `OPS-1`–`OPS-7`, `PUB-1`, and `GOV-1`–`GOV-3`. Foundations 1–5 retain their constraints.

## Recommendation and its limits

The next spec should target bounded source reconciliation, a coarse map of rule obligations and interactions, and selection of one difficult playable slice with independent expected outcomes and a concrete reuse disposition. This resolves the earliest prerequisite in foundation 2 and `OPS-1`. A staged first-playable spec is a material alternative, but its gameplay implementation cannot become ready merely because the source acceptance gate appears in its task list.

The deliverable must make the subsequent implementation decision concrete. It should identify the exact source set and precedence, ambiguities and omissions, one valid starting position, the complete decisions within the selected task, independent expected consequences and refusal cases, required shared mechanisms, and the estimated implementation and human-review burden. It must not grow into bulk card transcription, a generic COIN engine, or a speculative architecture backlog.

No full obligation map or final worked position has been derived or accepted by this preliminary investigation. The owner accepted the source task's two-active-hour allowance and one concise acceptance review, as recorded below. The existing full standard-game ambition remains unchanged.

## Source-derived candidates

GMT's [fifth-printing product page](https://www.gmtgames.com/p-1227-cuba-libre-5th-printing.aspx), checked live, still links the [March 2019 Living Rules](https://s3-us-west-2.amazonaws.com/gmtwebsiteassets/cuba/CL-RULES-2019-LivingRules.pdf) and [third-printing playbook](https://gmtwebsiteassets.s3-us-west-2.amazonaws.com/cuba/CL-PLAYBOOK-2018.pdf). The local second-printing playbook therefore still needs reconciliation. The product page places the [2013 corrections](https://s3-us-west-2.amazonaws.com/gmtwebsiteassets/cuba/Errata29Nov2013.pdf) under First Edition; their age and applicability matter to precedence.

1. **26July Kidnap, accompanying Terror, and a subsequent decision.** Rules 4.1 explicitly illustrate zero-resource 26July using Kidnap to finance Terror. Sections 3.3.4, 4.1.1 and 4.3.3 constrain the operation and its special activity. Cash ransom substitutes a Cash transfer for a Resource roll, so the engine cannot assume every Kidnap supplies spendable Resources. Special Activity use also changes the second eligible faction's options (2.3.4). This is the leading compact candidate for testing timing, funding, randomness and choice completeness.
2. **Negotiated Cash-funded extra operation.** Sections 2.3.6 and 4.5.2 connect Cash, transfers and an additional free Limited Operation. This offers a stronger negotiation and ownership case, with a larger timing and communication surface.
3. **Propaganda with contested Casino income.** Sections 6.1–6.5 combine victory timing, income, Skim, Cash deposits, support decisions and reset. It is a strong lifecycle case but likely substantially larger.

These are candidates, not accepted scenarios. An operation-resolution task and a complete card are different promises: a complete-card claim must cover every legal alternative in its supported position, not only Kidnap/Terror. The third-printing playbook's worked examples can help derive independent expectations; reproducing a scripted tutorial alone does not establish broader fidelity. Optional and Non-Player rules remain outside the accepted product scope.

## Refreshed reuse evidence

All four sibling checkouts were inspected read-only and were clean. No remote reachability or runtime behavior was verified.

| Repository | Local branch | Inspected HEAD |
|---|---|---|
| Arkham | `main` | `4ba5a7bd050a992b7d463e23d33633efe3125b5b` |
| mudsong | `elma-silverbough-dossier` | `26eb660eaf7d6fab25e3f06382a4a0130e7ab8c3` |
| Netrunner | `integrate/7-introductory-matches` | `07b2adb0dcdbf5ca75204290631febcb4bf473fa` |
| unfolded-tabletop | `main` | `7a8e30a72b2e1907c53905d7530f103eaf7fd31c` |

These are inspection identities, not chosen dependency pins. Arkham and Netrunner currently pin shared revision `84c857ecca4e6d22b2a5327e46b95642ea594acf`; mudsong pins `a94b3e2d173b742ed0b80e6dbda30eefe2f538dd`.

| Need | Current source and disposition |
|---|---|
| Development interaction and screenshots | Already shared: `unfolded-tabletop/src/seat.rs`, `src/seat/socket.rs`, `src/capture.rs`. Inspect actual adapters in Arkham `src/agent_seat.rs`, `src/gathering_tabletop/seat.rs` and mudsong `crates/app/src/seat.rs`, `crates/app/src/main.rs`, `crates/app/tests/seat.rs`. Cuba Libre owns observations and interaction targets. |
| Seat client tooling | Arkham `tools/src/seat.rs`, `tools/tests/seat.rs`; common behavior requires assessment/extraction rather than copying. |
| Live faction ownership and communication | Still Arkham-owned: `src/partners.rs`, `src/partners/{routing,shared,view,talk}.rs`, `src/gathering_tabletop/partners.rs`, `tools/src/partner.rs`, `tools/tests/partner.rs`. Extract common behavior if this consumer scope is selected. Cooperative investigator policy, peril and talk limits remain Arkham-specific. |
| Save replacement | Shared `src/record_replacement.rs`; Arkham `src/saved_state.rs`, `src/gathering/continuity.rs` and Netrunner `src/saved_state.rs` show consumer policy. Shared mechanics do not supply Cuba Libre save semantics. |
| Assets and physical geometry | Shared `src/assets.rs`, `card.rs`, `token.rs`, `loading.rs`, `camera.rs`, `count_label.rs`, `random.rs`. These do not constitute a complete installer. |
| ZIP installation | Still Arkham-owned: `src/assets/package.rs`, `package_tests.rs`, `src/asset_config.rs`, `src/bin/assets.rs`. Extract if selected; no distribution authority follows from installer availability. |

The development seat is mandatory for the first interactive slice and distinct from external faction players. Arkham's ownership is runtime-only; its 280-character talk behavior does not decide Cuba Libre proclamation limits or save persistence. Each necessary extraction retains foundation 3's two-active-hour attempt bound, affected-consumer verification and producer/consumer merge-and-pull ordering. The Cuba Libre reuse-waiver extension must be carried into participating work records before reliance.

## Owner selections from the interview

The owner accepted both first-round recommendations: the next spec targets the source baseline and slice-selection decision; the first playable payoff is a tactical exchange, with negotiation and external faction play following later. The mandatory development seat remains in the first interactive slice. These selections do not activate implementation or publication.

The owner then clarified the following product requirements. They govern the upcoming source/slice selection and future specs, alongside the foundations:

- **Cumulative development:** partial slices must build from spec to spec toward the eventual standard game. The normal deliverable is a retained part of that application, not a succession of stranded experiments. The source task must explain what its proposed slice contributes and how later work extends it; it must not invent a full implementation backlog or require preservation of a disproven mechanism. No standalone prototype lane is selected.
- **Clear decisions and automatic arithmetic:** the interface must make the current task, relevant concerns, variables, counts and sums clear. Players make choices; the application counts pieces and derives totals, costs and other bookkeeping. The visible explanation must make the arithmetic inspectable rather than requiring the player to reconstruct it.
- **Software-adapted map:** preserve regions, cities and other rule-relevant geography/topology. Physical-board trackers, token storage slots and similar bookkeeping furniture may be replaced or improved through software interfaces. This does not change rule semantics, three-dimensional pieces, VectorCraft authoring or the required three visual alternatives before design adoption.
- **Persistent victory overview:** the top of the gameplay interface must constantly show all four factions' current victory standings and thresholds, with access to what contributes to them. A tooltip breakdown is the owner's likely presentation preference, not yet a selected visual design. Inspect mudsong's existing tooltip behavior for shared extraction before implementing a new mechanism.

These clarify the intended software experience; they do not activate visual authoring, extraction or the full game. The initial source task must include victory calculations and their presentation implications when judging the first scenario's completeness and burden.

## Victory terminology and source implications

Live inspection of the March 2019 rules, sections 7.2–7.3, confirms that a single scalar is insufficient to explain every victory condition. During Propaganda checks, Government needs Total Support above 18 and every city at Active Support; 26July needs Opposition plus Bases above 15; Directorio needs controlled Population plus Bases above 9; Syndicate needs more than 7 open Casinos and Resources above 30.

The application should show the totals, threshold comparisons and additional unmet conditions. It must distinguish current conditions from a completed victory check. Final scoring uses victory margins; Government's city requirement is not part of its final margin, while Syndicate's margin is the lower of its two differences. Exact current source precedence and card exceptions remain part of the upcoming reconciliation task. “Victory standing” names the overview; “victory margin” retains its specific rules meaning.

## Mudsong tooltip follow-up

The owner's explicit tooltip request prompted a second read-only sibling inspection. Mudsong remained at `26eb660eaf7d6fab25e3f06382a4a0130e7ab8c3`, Netrunner at `07b2adb0dcdbf5ca75204290631febcb4bf473fa`, and shared at `7a8e30a72b2e1907c53905d7530f103eaf7fd31c`; Arkham had advanced to `71281007a41905bf25a95a29486e10fb11b0a83f`. Mudsong now had README/world-bible/design-authority changes and Netrunner Cargo manifest/lock changes. These concurrent changes were untouched; the earlier clean-checkout observation is historical, not a continuing assertion.

Mudsong calls the relevant mechanism pointing/growing. `crates/app/src/pointing.rs` handles delayed hover, switching between targets, dismissal, hotspot geometry and enlarged-content visibility. `crates/app/src/staging.rs` handles placement and current timing/presentation policy; `dimming.rs` and its shader handle screen dimming. Its current 300 ms dwell, 200 ms fade and 50% dimming are mudsong policy, not accepted Cuba Libre choices. Placement above the source with edge clamping may be useful. Mudsong's straight-on picking assumptions require assessment against Cuba Libre's interface.

`crates/app/src/seat.rs` exposes pending hover/fade and grown content for settle/observe/capture. `crates/app/src/pointing/tests.rs` includes dwell, transient crossing, direct switching, fade, dismissal and render-layer checks. These tests were inspected, not run. Kit rules, place cards, harm, miniatures, fate tokens, discard piles, character switching and action offers are mudsong-owned content and policy.

No shared tooltip module exists. Shared `src/count_label.rs` supplies numeric labels attached to 3D tokens; it does not compute victory totals or provide a fixed standings bar. Arkham's `src/tabletop_feedback.rs` and `src/gathering_tabletop/talk.rs` have related hover/focus behavior; Netrunner uses shared count labels but no equivalent tooltip mechanism was found in the inspected source.

The later bounded extraction should target only the hover lifecycle and positioning actually needed by both consumers, preserve mudsong's adapter and policy, and leave Cuba Libre's score calculations and breakdown meaning local. Dimming is not selected merely because mudsong implements it. Expected primary participants are shared, mudsong and Cuba Libre; any additional affected consumer must be justified by the final mechanism. Verify the producer and mudsong's ordinary checks (`cargo xtask verify`) plus its actual point/settle/observe/capture path, then Cuba Libre's contribution display and updates. Preserve consumer pins for rollback. This is an extraction candidate, not an accepted implementation design or an activated sibling task.

## Accepted second-round decisions

The owner answered “as you recommend” to both questions:

3. **Prepared position:** the first interactive slice may begin from a rules-valid midgame position in the actual application, with the map and all four victory standings, to complete a bounded tactical task. Later specs extend that application toward standard setup and complete games. The exact scenario and complete supported choice boundary still come from the source task.
4. **Initial allowance:** the next spec's execution has a maximum of two active agent-hours for source reconciliation, the coarse interaction map, candidate scenario and independent calculations, and reuse plan, followed by one concise owner acceptance review. At exhaustion, report the exact evidence and remaining gap and recommend a terminal decision; no silent continuation or readiness claim. This is the future source task's allowance, not authority to start it during interview preparation.

## Prepared handoff for `to-spec`

Suggested issue title: **Pin the rules baseline and select the first cumulative tactical slice**.

The next coding agent's immediate consumer need is a source-grounded decision about what can be implemented faithfully and at what cost. The source task resolves that prerequisite; production gameplay, art authoring, shared extraction and asset distribution remain later work. Synthesis should use the glossary, the four accepted interview answers and the owner's interface clarifications above. It must preserve foundations 1–5, including mandatory development-seat verification, Rust-only executable code, three VectorCraft design alternatives and coordinated reuse ordering.

The bounded deliverable should contain:

1. Exact controlling source identities, provenance and precedence, including the mixed-printing reconciliation and explicitly unresolved interpretations. Identify sources required for the full standard-game promise without bulk transcribing the entire card inventory.
2. A coarse source-derived map of the standard game's behavioral and cross-cutting obligations, exclusions and independent oracles. Estimate implementation, authoring, adjudication, testing, review and maintenance burden proportionately; do not enumerate the state space.
3. One recommended difficult, rules-valid tactical starting position and complete bounded player task. Explain why this case tests a material interaction; identify every legal choice within that task, stopping boundary, supported alternatives, expected arithmetic/consequences and relevant refusal cases. Kidnap/Terror is the leading hypothesis, not a preaccepted conclusion.
4. The first native slice's required visible information: current decision and owner, relevant restrictions and costs, automatic counts and explicit sums, all-faction victory standings and contribution breakdowns. Its geographic map may use software interfaces in place of physical-board bookkeeping furniture. Estimate the cost of a complete supported scenario rather than disguising missing legal alternatives as a finished task.
5. A refreshed reuse disposition and bounded extraction plan for mechanisms actually required by that slice, including the development seat, recovery and mudsong hover behavior where selected. Name producer and affected consumers, actual verification paths and rollback. No sibling changes occur in this source task.
6. A concise owner decision: accept the exact source baseline and proposed slice, narrow it, or resolve the disclosed remaining gap. Explain how accepted artifacts feed the next implementation spec and how the first native slice remains part of the growing application. No experiment is left as a disconnected substitute for that consumer path.

### Verification boundary to carry into the spec

For this source-only task, verify the complete proposed task against independently read official sources: starting-position validity, legal alternatives, timing, costs, derived sums, victory conditions, consequences and refusal expectations. Bind citations to pinned source identities; check reference/hash consistency mechanically where appropriate, and distinguish semantic source interpretation from those mechanical checks. Independently recompute the worked expectations rather than treating a proposed implementation or generated summary as its own oracle. Compare the scenario's obligations with the coarse interaction map and expose omitted or ambiguous behavior.

There is no application test seam yet, and this task does not create a surrogate rules engine, test framework or runnable prototype. Its result supports source/slice acceptance only. The subsequent implementation should be tested at the real checked game-decision boundary shared by human and agent input, then through the production application using the development seat for observe/interact/settle/capture. Recovery checks must include unresolved choices across process restart and stale submissions. These are future consumer acceptance requirements, not claims of evidence already obtained. Human playtesting follows feasible functional and screenshot verification.

The interview settled the current product-scope and appetite decisions. The owner subsequently invoked `to-spec` with this handoff and explicitly included its source-only verification boundary, accepted interface requirements and two-hour allowance. That invocation confirmed the final shared-understanding surface and authorized issue publication with `ready-for-agent`. Source identities, ambiguities, the exact scenario and estimates are outputs of the selected next task, not questions to shift onto the owner during this interview. Readiness for that bounded source task must not be described as readiness for gameplay implementation. Issue #2 contains the operative decisions so its use does not depend on these uncommitted supporting notes; this reference document does not own activation or status.

The specification was published as issue #2 with `ready-for-agent` under the explicit `to-spec` invocation. No source-task execution, gameplay, visual production, sibling modification, commit or merge was performed by that publication.

## Published ticket decomposition

The owner subsequently invoked `to-tickets` for #2. Its complete body was fetched and matched the published spec; it had no comments or existing sub-issues at inspection. The owner approved the proposed two-ticket decomposition with “Proceed with your recommendation.” Published children:

1. **[Issue #3: Reconcile official rules and recommend a difficult tactical case](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/3).** No predecessor. Deliver the identified source baseline, coarse interaction map, proportionate difficult-case comparison and a justified candidate with explicit gaps and a source-only handoff.
2. **[Issue #4: Verify the complete tactical scenario and cumulative implementation handoff](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/4).** Blocked by #3's usable source/candidate result. Work through the complete legal task and independent arithmetic, required visible information, refreshed reuse/extraction assessment, future real-application checks and burden estimate; present the combined owner decision.

Both children share #2's total two-active-hour allowance and one owner acceptance review. The first child does not introduce an intermediate human approval gate: its candidate can feed further bounded source analysis while remaining unaccepted for production. A closed predecessor without adequate evidence cannot unlock unsupported work. No code or prefactoring is in scope.

Publication readback verified both exact ticket bodies, both `ready-for-agent` labels, native sub-issue links to #2 and the native #4-blocked-by-#3 relationship. The parent's body, title, labels and open state were preserved. Ticket publication did not execute either source task. GitHub continues to own work status, activation and acceptance; these notes preserve the publication evidence only.

## Implementation-spec interview, 2026-11-15

The owner requested a fresh `grill-with-docs` reading of all foundations and research to recommend the next work and prepare a later `to-spec` invocation. Inspection fixed point: `38e988442dfa6ac287baf0a373e77fde5ac77764`, initially clean. All five foundations and four research documents were read. Live GitHub inspection found issues #1–#4 and #6 closed, and PRs #5 and #7 merged; no open work item was returned. The [acceptance record on #2](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/2#issuecomment-6885871005) accepts the baseline/scenario at `deacd635db5f71f4d3f092bd9cc99a9d96bb7aab`. Subsequent changes to those two source documents only clarify that acceptance. No executable consumer exists.

This section supports the next specification decision from #2's completed handoff. It is interview evidence, not a local ticket or activation record. No new issue, implementation, sibling mutation, visual adoption or publication is authorized by this note.

Consulted: AUTH-1–3; PURP-1–6; WORK-1–6; EVID-1–10; SEM-1–2; REP-1–3; BOUND-1–5; INT-1–4; AGENT-1–5; OPS-1–7; PUB-1; GOV-1–3; ADOPT-2; foundations 1–5. The immediate distinctions are accepted source versus actual consumer evidence, complete operation versus complete card, development seat versus faction seat, bounded effort, and independent human visual/play acceptance.

**Selected outcome:** specify the complete accepted Terror/Kidnap task in the growing native application. Preserve all supported choices, donor authorization, voluntary removal, both accepted local interpretations, full geographic presentation, all-faction standings and breakdowns, save/restart recovery and the development seat. Stop at displaying Directorio's next options. External faction players, proclamations, full-card execution, standard setup, bulk content, and asset redistribution remain outside this increment. The handoff estimates 49–86 active agent-hours plus up to four extraction hours; that estimate grants no allowance and has not been validated by implementation.

**Accepted Q1:** The owner answered “Yes — complete native tactical task.” This selects the future spec's outcome, rather than the smaller rules-only alternative. It does not start implementation.

**Accepted Q2:** The owner answered “Yes — MIT for original code.” Carry that selection into the future spec and executable-code publication; preserve third-party rights separately. No GMT material or supplied card image is relicensed. This records the selected license; no license file or code was published during the interview.

### Refreshed implementation facts

A fresh-context, read-only worker inspected the four repositories under their own instructions. Local HEADs were Arkham `71281007a41905bf25a95a29486e10fb11b0a83f`, mudsong `b6c05ac561bf52f4dabb7d77fa313889a768b628`, Netrunner `3a64b7138f906b5e79ca135e463b68f85415bde6`, and unfolded-tabletop `ed0c8866b11c6af5378dc8bffdc8dc98ce45c40b`. All were clean at inspection; Netrunner was on `integrate/7-introductory-matches`, the others on `main`. These facts do not establish remote publication or runtime success.

The handoff's two extraction candidates remain local: Arkham `tools/src/seat.rs` owns the socket client; mudsong `crates/app/src/pointing.rs::grow` owns the hover lifecycle. Arkham's interrupt checking needs a consumer adapter; its partner-specific `seat-wait` remains excluded. mudsong's content, picking, geometry and dimming stay local. Its proprietary foundation explicitly permits this narrow game-neutral extraction to the public MIT shared library; private content/source dumps are not part of Cuba's artifacts. The existing verification paths and coordinated rollback in the accepted handoff still apply.

Shared `src/seat.rs::ui_target` now provides Bevy UI-node center/clipping geometry; it replaces neither proposed extraction. Inspect it before writing Cuba's picking adapter. The inspected Arkham/Netrunner manifests still pin shared `a8b1aa5b1a6dbd4c9401ebb4ff3e815bff03c655`, and mudsong pins `a94b3e2d173b742ed0b80e6dbda30eefe2f538dd`. Refresh actual publication/migrations before selecting a dependency pin. No builds or native probes were performed. The source-only estimate remains uncertain.

### Accepted effort and human-review boundaries

**Accepted Q3:** The owner answered “as you recommend.” The future increment has a maximum of 90 cumulative active agent-hours, including at most two hours per extraction attempt (four total), implementation, art, verification and fixes, with zero paid provider calls. This ceiling accommodates the handoff's upper planning estimate; it is neither a delivery promise nor a requirement to spend it. Preserve earlier constitutional stop conditions and separate publication acceptance. The allocation is one primary-path native delivery, with necessary coordinated producer/consumer work charged to it and no unrelated work used to reset its allocation. No extension is automatic. This selects the future specification's allowance, not activation during the interview.

**Accepted Q4:** The owner answered “as you recommend.” Allow one combined visual-selection round, with three competent VectorCraft alternatives for each new design, bounded to 15 minutes of owner review; then one native tactical-play review bounded to 30 minutes after feasible agent checks. Preserve constantly visible standings, accessible contribution breakdowns, clear decision/cost/count explanations, and the complete task. The owner judges readable consequences, meaningful choice and physical-input comfort; source-backed decision tests, rendered inspection and process-restart/reconnect checks are the agent's responsibility. An inconclusive/rejected round counts and stops for a disposition; it does not authorize repeated redesign or playtesting. Additional review rounds require an explicit new burden allowance.

### Ready handoff for the native implementation specification

Suggested issue title: **Implement the first native Terror/Kidnap tactical task**. The four accepted answers settle the interview's remaining scope, original-code license, effort and human-review decisions. The next owner action is the requested `to-spec` invocation; do not restart the interview or activate implementation from this note. The earlier source-only handoff in this document is historical and must not be republished as the new task.

Synthesize from the accepted baseline/scenario at `deacd635db5f71f4d3f092bd9cc99a9d96bb7aab`, its acceptance in #2, foundations 1–5, the glossary, this interview and the refreshed reuse evidence. Preserve the complete operation boundary, including zero-space Op Only without Pass income and the accepted owed-Terror guard. Both remain disclosed local interpretations. No additional rule policy, full-card support or strategic-agent claim is selected.

The player-facing acceptance target is to understand the current tactical choice, inspect its costs and contributing counts, execute or deliberately finish the operation, see the consequences and changed continuation, and recover an unresolved task after interruption. A completed rules test is insufficient for that native outcome. Keep original authored art and the small Cuba-specific implementation as part of the growing application; avoid a separate tutorial executable or generalized COIN engine.

**Verification seam for synthesis:** use one checked game-decision boundary for legal-choice observation and submissions, shared by human controls and eventual faction inputs. Exercise source-derived expectations through that production boundary: all six Kidnap results, both Terror orders, late/no Kidnap, donor-owned transfers and refusals, own removal, zero-space completion, owed-Terror preservation, standing calculations and Directorio's resulting permissions. Refused/stale submissions must leave state and randomness unchanged. This boundary is to be implemented; no Cuba executable or test seam exists yet.

Exercise the same application through shared development-seat observe/interact/settle/capture, inspecting actual frames. Check restart while awaiting a donor, after the roll before owed Terror, and after Terror before Finish; reconnect and stale/duplicate input must not repeat effects. Refuse corrupt or incompatible saves without replacing the last valid state. Use the accepted handoff's bounded source-change/consumer check, independent oracle and recovery examples. This is the verification path explained during the interview and retained with Q4; source-only review and a parallel mock implementation cannot substitute for it. Broader recurrent/game fitness remains unproven beyond the actual task and recovery exercised.

**Delivery constraints:** all project-owned executable code is Rust. Consume existing shared mechanisms, and complete the two necessary bounded extractions with affected consumer checks, actual-path verification, PRs, dependency-order merges and main pulls before dependent Cuba implementation. Apply each repository's authority rules; the future specification must name these coordinated scopes and retain their individual publication gates. Reinspect changing revisions before choosing immutable dependencies. Keep original light VectorCraft sources/exports in Git; use locally admitted card faces with retained identities and clear missing-asset failure. Asset redistribution and a new ZIP installer are outside this slice.

The spec must carry the accepted 90-hour cumulative ceiling, per-extraction caps, zero paid calls, review limits, earlier constitutional stop conditions, disposition at exhaustion and exact publication boundary. GitHub owns activation, readiness and closure. The later `to-spec` invocation authorizes its issue publication according to that skill; it does not grant blanket acceptance or merging of future production changes. Foundations/README still contain the earlier unresolved-license wording; reconcile it to Q2 as part of the accepted code-licensing change, without changing third-party asset custody. No new ADR is warranted by these selections: the accepted source and this interview already record their reasons and constraints.

### Native specification published

The owner invoked `to-spec` with the accepted native implementation handoff. Published [issue #8: Implement the first native Terror/Kidnap tactical task](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/8), with `ready-for-agent`. Readback verified the exact issue body, title, open state and label. The issue incorporates the immutable accepted scenario, all four interview decisions, the agreed production decision/development-seat/recovery verification path, coordinated reuse constraints and publication limits. It owns subsequent work status; these notes are supporting evidence only.

The invocation confirmed synthesis from the agreed handoff without another interview. No implementation, decomposition, sibling mutation, commit, merge, asset publication or runtime activation occurred. The glossary/interview changes remain local and uncommitted, and #8 contains their operative decisions so execution need not recover this conversation. Readiness is not implementation activation or evidence of a successful native run.

### Approved decomposition of #8

The owner invoked `to-tickets` for #8. Its complete body and comments were fetched; it was open with `ready-for-agent`, no comments and no existing sub-issues at inspection. The owner approved the eight-ticket proposal with “Yes, the granularity looks right. Proceed.” GitHub owns the published tickets and dependencies; this record is supporting evidence, not a local work queue.

| Draft | Deliverable | Blocked by |
|---|---|---|
| T1 | Select and prepare the complete tactical interface: three competent VectorCraft alternatives per new design, one combined selection, editable sources and usable exports | None |
| T2 | Share the development-seat client, migrate Arkham and verify its real held-seat path | None |
| T3 | Share the hover lifecycle, migrate mudsong and verify its real pointing/reading path | None |
| T4 | Inspect the prepared native table and all standings, save/restart, and execute zero-space Operation Only completion through real controls/development seat | T1, T2, T3 |
| T5 | Authorize/decline Resource transfers locally, inspect changed balances/contributions and recover pending/committed transfers | T4 |
| T6 | Resolve full Terror/Kidnap timing and funding choices, including real donations, six outcomes, owed-Terror guarding and restart | T5 |
| T7 | Remove 26July pieces before/after effects, preserve obligations and complete the scenario's choice closure | T6 |
| T8 | Verify the complete integrated task/recovery and present the single bounded native owner review | T7 |

T1–T3 are necessary prerequisite products/refactorings with their own verifiable outputs and real consumer evidence; they are not surrogate proof of Cuba gameplay. T4–T7 each cross production decision, presentation, development-seat and persistence paths. T6 depends on T5 because no-Kidnap/late-Kidnap sequences and changed funding must be exercised using real authorized donations. Recovery ships with each introduced state rather than as a late disconnected subsystem. T8 verifies the combined contract and consumes the already-accepted human-play review; it does not introduce another review allowance.

All children share #8's 90-hour ceiling, two-hour-per-extraction caps, zero paid calls, visual/play review limits and earlier stop conditions. Ticket boundaries do not reset effort, WORK-3 allocation counts or acceptance requirements; partial demos remain incomplete evidence for #8. No new allocation waiver or automatic continuation is proposed. The smallest shared producer/consumer records remain owned by their respective repositories during authorized execution; these Cuba children coordinate the required result without preauthorizing sibling publication.

Published children, mapping T1–T8 respectively: [#9](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/9), [#10](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/10), [#11](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/11), [#12](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/12), [#13](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/13), [#14](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/14), [#15](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/15), and [#16](https://github.com/joeloverbeck/cuba-libre-unfolded/issues/16). All have native sub-issue links to #8 and `ready-for-agent`. Native blockers are #12 ← #9/#10/#11; #13 ← #12; #14 ← #13; #15 ← #14; #16 ← #15. The first three have no ticket predecessors; this does not itself activate them.

Readback verified every exact child body/title, open state, label and dependency set, plus the exact eight-member parent sub-issue set. The parent's body, title, labels and state were unchanged. No implementation, sibling mutation, commit, merge or runtime activation occurred. Consultations already recorded for #8 remain applicable; WORK-3–6 and AGENT-5 were reread for this decomposition. Safe return is separately authorized selection/execution through GitHub under #8's shared bounds.
