---
name: skill-improvement
description: Propose reusable skill fixes from completed work. Use at task close after feedback or a workflow gap, or for an explicit retrospective. 스킬 개선, 작업 회고, 반복 실수.
---

# Propose skill improvements after work

For a completion review, finish and verify the user's original work and follow-up requests first. Review the completed work just before the final report. When the user explicitly asks for a review, examine the work completed so far.

## Decide if the lesson will transfer

1. Identify what the user corrected, what a follow-up request clarified, or what verification exposed. Name the missed decision and the observed consequence.
2. Read the current skill and relevant project instructions. Check whether the workflow lacked guidance, the right skill was hard to discover, or a clear instruction was missed in execution.
3. Replace the names, ticket, and other local details with a different plausible case. Propose a change only if the same cause would call for the same action and the change would likely prevent the observed problem. Repeated evidence helps. One case can qualify if it establishes a reusable mechanism or exposes a consequential gap that can recur. Skip routine scope additions, one-time incidents, and edge cases tied to one environment.

## Choose the narrowest fix

- For a requirement a check can enforce reliably, suggest a lint rule, hook, type, test, or script check. If the skill already gives clear guidance, avoid repeating the rule. Suggest a check only when it would have caught the miss.
- For a project convention, suggest a project instruction or check. For an explicitly durable personal preference, suggest the user's instructions or memory. Keep both at their own scope.
- For a gap in a reusable judgment call or workflow, propose the smallest change to the skill that owns it. If the skill failed to load, fix its routing or description. Suggest a new skill only when there is a distinct workflow and no skill owns it.

## Report the proposal

When a candidate qualifies, add a short proposal after the task results. Name the affected skill or check, the evidence from this task, and the behavior the change would improve next time. Ask whether to apply it unless the user already authorized that change. If no candidate qualifies, finish the task report without a skills section. When the user explicitly asks for a session review, say briefly why there is no proposal.

## Apply an approved change

Honor the scope the user specified. Use the available skill authoring guidance for skill edits.

For a hibi-ai skill, use `~/.hibi/install.json` when available to confirm the skill's source and install target. If no matching hibi-ai source checkout is available and the user approved an edit without specifying a scope, update only the installed skill used by the current agent and verify the change there. The local edit can proceed without a clone.

After verifying the installed change, report where you applied it. If it would also help other hibi-ai users, ask whether to open a PR to the source repository. Follow the `pull-request` skill after the user requests the PR.
