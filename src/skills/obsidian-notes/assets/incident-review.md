---
type: retro
status: active
created: YYYY-MM-DD
tags: [type/retro, project/<slug>, topic/incident]
project: <slug>
related: []
---

# Incident Review

## Impact and current state

Describe the affected behavior, scope, duration, and current recovery status. Label estimates and link to the evidence that supports them.

## Timeline

| Time | Observation or action | Evidence |
|---|---|---|
| <time and zone> | <what happened or changed> | <log, test result, or note> |

## Cause and uncertainty

State what the evidence confirms. Label suspected causes and give the observations for and against them. Record questions that remain open and what would answer them.

## Follow-up and verification

- [ ] <prevention or recovery action>. Record the owner and due date when known. Say how to verify that the action addresses the failure.

Keep `status: active` while follow-up actions are open. Use `status: closed` when none remain.
