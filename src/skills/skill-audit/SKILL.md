---
name: skill-audit
description: "Audit and improve skills for usefulness, overlap and situational fit. 스킬 효용성, 중복, 발동 조건, 품질 감사."
---

# Audit and improve skills

Use this workflow when the object under review is a skill or a collection of skills. Identify the requested decision, the source or installed copy under review, and whether the user wants findings or edits. For a lesson from one completed task, `skill-improvement` owns the retrospective when available. Use existing evaluation tools and fixtures to run trials.

## Establish what each skill owns

1. For each skill in scope, record a realistic user request, the inputs it needs, the decision it changes and the deliverable another person can inspect. Follow the entry point to relevant references, scripts and assets. Look for callers, automatic imports and explicit invocations before proposing a move or removal.
2. Find the closest alternative, including ordinary model behavior or an existing project rule. Compare the same request against both owners. Different wording can prescribe the same action. A designer creating focus states and a reviewer testing them can share vocabulary while producing different deliverables.
3. For a collection, list every entry and identify which ones received a behavioral trial. The optional [inventory script](scripts/inventory.py) enumerates immediate skill directories and finds matching instruction bodies in entry points, references and rules. Pass the parent skills directory as its argument. Use matches to select passages for review.

## Test utility, routing and execution separately

- **Utility.** Write down what would count as a better user outcome before seeing answers. Compare the same request and inputs with and without the target skill, leaving the closest alternative available in both runs. Inspect the outputs against the same criteria. Record gains, ties, regressions and missing evidence. Weigh the impact on real tasks against extra reading, steps, tool calls and upkeep. Use observed demand or the consequence of a rare failure when that evidence exists.
- **Routing.** Test ordinary user phrasing, a close request owned by a neighbor and a request that genuinely needs both when applicable. Identify the desired owner and why. Keep catalog visibility, a reader's selection, an observed runtime invocation and a successful result as distinct observations. For an always loaded skill, test its influence on unrelated work. For an explicit invocation, test the named path.
- **Execution.** Trace a representative task from request to deliverable. Try a boundary where one condition changes the right action, such as unavailable data, a missing tool or a narrower user instruction. Verify referenced resources and current project conventions. Inspect actual output, state, or errors. Record the evidence still missing when a dependency prevents the run.

Trace any command that writes, sends or deletes to the user's authorization and the promised side effects. Use a disposable fixture for behavioral trials. Check whether a suggested default creates extra work, a stale example contradicts the rule, or an optional resource became a prerequisite. Cite the path and the scenario where this changes the outcome.

Use [comparative trials](references/comparative-trials.md) to choose cases and diagnose ambiguous results. Keep test inputs and expected behavior separate when giving a case to a fresh evaluator.

## Decide, improve and retest

Lead with findings that could produce the wrong action or deliverable. For each consequential finding, give the affected file or instruction, triggering condition, observed result, evidence and smallest justified change. A tie leaves the added benefit open on those cases. Keep a distinct owner when it serves a verified task. Consider narrowing discovery, correcting a rule, moving detail, consolidating equivalent owners, or retiring guidance whose supported work is covered elsewhere. Check unique exceptions, callers and installation modes before recommending consolidation.

For a review request, return findings and proposed changes. When edits are in scope, change the responsible source, preserve required mirrors and repeat the original failure, a fresh variant and the closest unaffected task. Check package discovery and listing budget when descriptions change. Continue while a new observation supports another fix. Finish with the decision for each target, the tested scope, regressions and unresolved questions. Report an inconclusive run when the comparison cannot support a verdict.
