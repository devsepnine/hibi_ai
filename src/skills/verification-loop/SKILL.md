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

The commands below describe what to verify. Apply this execution setup before choosing how to run them.

## Verification Phases

### Phase 1: Build Verification
```bash
# Check if project builds
npm run build 2>&1 | tail -20
# OR
pnpm build 2>&1 | tail -20
```

If build fails, STOP and fix before continuing.

### Phase 2: Type Check
```bash
# TypeScript projects
npx tsc --noEmit 2>&1 | head -30

# Python projects
pyright . 2>&1 | head -30
```

Report all type errors. Fix critical ones before continuing.

### Phase 3: Lint Check
```bash
# JavaScript/TypeScript
npm run lint 2>&1 | head -30

# Python
ruff check . 2>&1 | head -30
```

### Phase 4: Test Suite
```bash
# Run tests with coverage
npm run test -- --coverage 2>&1 | tail -50

# Check coverage threshold
# Target: 80% minimum
```

Report:
- Total tests: X
- Passed: X
- Failed: X
- Coverage: X%

### Phase 5: Security Scan
```bash
# Check for secrets
grep -rn "sk-" --include="*.ts" --include="*.js" . 2>/dev/null | head -10
grep -rn "api_key" --include="*.ts" --include="*.js" . 2>/dev/null | head -10

# Check for console.log
grep -rn "console.log" --include="*.ts" --include="*.tsx" src/ 2>/dev/null | head -10
```

### Phase 6: Diff Review
```bash
# Show what changed
git diff --stat
git diff HEAD~1 --name-only
```

Review each changed file for:
- Unintended changes
- Missing error handling
- Potential edge cases

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
- When a check fails, suspect the observation method before the system.
- Prefer a script that re-runs the comparison over a one-time look; keep its output as the evidence.
- Never hand the user a check you could have run. If it truly cannot run here, for example with no device or no credentials, mark it `INCONCLUSIVE` and say what would settle it. `INCONCLUSIVE`, or a check run in a different environment than the one changed, is `NOT READY`.

## Output Format

After running all phases, produce a verification report:

```
VERIFICATION REPORT
==================

Build:     [PASS/FAIL]
Types:     [PASS/FAIL] (X errors)
Lint:      [PASS/FAIL] (X warnings)
Tests:     [PASS/FAIL] (X/Y passed, Z% coverage)
Security:  [PASS/FAIL] (X issues)
Diff:      [X files changed]
Artifact:  [PASS/FAIL/INCONCLUSIVE] (what was run, what was observed)

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
