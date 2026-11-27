---
name: verification-loop
description: Build, type, test, and security verification loop. Use before reporting completion or when a build or type check fails. 빌드 검증, 타입 체크, 테스트 실행, 검증 루프, 완료 전 검증.
---

# Verification Loop Skill

A comprehensive verification system for Claude Code sessions.

Phases 1 to 6 are proxies: a green build, type check, and test suite say the code is consistent, not that the change does what was asked. Phase 7 checks the real artifact. Only a run that includes Phase 7 may report `Overall: READY`; a partial scope such as `/verify quick`, `pre-commit`, or Continuous Mode reports `Overall: PARTIAL`, and Phase 7 runs once, at completion.

## When to Use

Invoke this skill:
- After completing a feature or significant code change
- Before creating a PR
- When you want to ensure quality gates pass
- After refactoring

## Execution setup

Before running code, builds, tests, or development servers:

1. Discover `rundebugconfig` MCP tools. If available, call `list_configurations` to confirm the connection and find settings for the current project.
2. When connected, inspect a candidate with `get_configuration` to confirm its directory and command match the task. Check `list_sessions` before starting another run. Use an existing relevant session for logs or a running server.
3. Prefer `run_configuration` for a matching saved configuration when the tool is available. Read `read_session_output` and `list_sessions` for output, final status, and exit code. A returned session ID proves only that a session exists. For a server, check readiness and the requested behavior.
4. Missing tools can reflect the app's permission tier. Use the available read tools even when execution tools are absent. Do not bypass an explicit permission denial with shell execution.
5. If the MCP is unavailable, no configuration matches, or execution tools are absent, state the limitation and use an otherwise permitted shell command. Do not create or rewrite saved configurations just to run a check. If a start request has an uncertain outcome, check sessions before retrying or falling back to avoid duplicate processes.

Select commands from the project's documentation, manifest, and CI, then use the execution setup above. If the project has no corresponding step, record `N/A` with the reason and continue with applicable checks.

## Verification Phases

### Phase 1: Build

Run the project build when the artifact requires one. Resolve a failed build before running checks that depend on its output. For a Python CLI run directly from source, exercise its actual entry point.

### Phase 2: Type check

Use the checker and configuration the project provides, such as its TypeScript type check script or Rust's `cargo check`.

### Phase 3: Lint

Run the linters and formatters configured for the affected files.

### Phase 4: Test suite

Exercise the changed behavior with the repository's test runner. Record passed, failed, and skipped tests. When the project sets a coverage threshold, compare the measured result with that threshold.

### Phase 5: Security checks

Follow the data and permission boundaries affected by the change. Check input handling, credential exposure, and logs. Use the repository's scanner where available and select the relevant file types. Report paths only for potential secrets. If using a shell search, check which tool the name invokes and whether its file filters cover the intended paths. For a normal `rg` or `grep` search, exit code 1 means no matches and a value above 1 indicates an error.

### Phase 6: Diff Review
```bash
# Show what changed
git status --short
git diff --stat
git diff --cached --stat
```

Review each changed file for:
- Unintended changes
- Missing error handling
- Potential edge cases

Apply `coding-standards` → `references/review-checklist.md`, Failure paths in self-review and PR review. Record the relevant failure scenarios, evidence, and untested limits separately from success checks. Use `tdd-workflow` for test case design.

### Phase 7: Observe the Real Artifact

Exercise the changed behavior directly and read the result:

| Change | Check |
|--------|-------|
| CLI | Run the real command on a real input |
| UI | Walk the changed flow in the running app, using `/e2e`, the `run` skill, or browser automation |
| Parser / migration | Replay a saved real input and diff the output |
| Storage / config | Read back the value that was written |
| Performance | Compare before and after measurements on the same harness |
| Docs only | Re-read the diff as its reader would; run evals only when behavior-bearing text changed, such as skill, agent, or command prompts |

- Read the actual value, not a cached or derived representation such as a file mtime, an agent's self-report, or an old screenshot.
- When the changed flow has a reachable failure, exercise a representative failure as well as success and inspect the promised state after failure. Use isolated data and controlled dependencies. If this cannot be run, report `INCONCLUSIVE` and the missing evidence; explain `N/A` when no failure scenario applies.
- For a changed UI flow, exercise the normal path and each distinct user-visible failure state affected by the change. Verify the visible message, whether the action is enabled, and whether the request or state change occurred as expected. Exercise recovery when the UI offers it. Use the project's browser test setup. Use local mocks when a real external call falls outside the scope of verification. When real integration is in scope, verify it in the authorized test environment. Report which dependencies were mocked.
- Capture the success screen and each error screen as soon as the assertions for that screen pass. Capture errors while the message and affected controls are visible, before correcting input, retrying, dismissing, or navigating. Use synthetic data, give files names that identify the scenario and browser, open the images to check that they show the intended state, and link them alongside the test results. Record any UI state you could not exercise or capture.
- When a check fails, suspect the observation method before the system.
- Prefer a script that re-runs the comparison over a one-time look; keep its output as the evidence.
- Never hand the user a check you could have run. If it truly cannot run here, for example with no device or no credentials, mark it `INCONCLUSIVE` and say what would settle it. `INCONCLUSIVE`, or a check run in a different environment than the one changed, is `NOT READY`.

## Output Format

When auditing a collection, record which items were inventoried, which contracts were checked together, and which behaviors were executed. For example, compile a caller against the interface its guide links to. In a repeated review, revisit relevant paths that previous rounds did not exercise and report the scope of any zero finding result.

After running all phases, produce a verification report:

```
VERIFICATION REPORT
==================

Build:     [PASS/FAIL/N/A]
Types:     [PASS/FAIL/N/A] (X errors)
Lint:      [PASS/FAIL/N/A] (X warnings)
Tests:     [PASS/FAIL/N/A] (passed, failed, skipped; coverage if measured)
Security:  [PASS/FAIL/N/A] (X issues)
Diff:      [X files changed]
Artifact:  [PASS/FAIL/INCONCLUSIVE] (what was run, what was observed)
Failure paths: [PASS/FAIL/INCONCLUSIVE/N/A] (scenarios, state after failure, evidence or limits)
UI evidence: (links to success and failure screenshots when applicable)

Overall:   [READY/NOT READY/PARTIAL] for PR

Issues to Fix:
1. ...
2. ...
```

Label every claim in the report and the completion message as **measured**, meaning you ran it and saw the output, **inferred**, meaning it follows from something measured, or **guess**. Report inferred claims as inferred; only measured claims can carry the verdict.

## Continuous Mode

For long sessions, run verification every 15 minutes or after major changes:

```markdown
Set a mental checkpoint:
- After completing each function
- After finishing a component
- Before moving to next task

Run: /verify
```

## Integration with Hooks

This skill complements PostToolUse hooks but provides deeper verification.
Hooks catch issues immediately; this skill provides comprehensive review.
