---
name: eval-harness
description: Eval-driven development — capability and regression suites, pass@k metrics. Use when evaluating AI output or running eval suites. 평가 프레임워크, 회귀 테스트, AI 평가, 성능 측정.
---

# Eval Harness Skill

A formal evaluation framework for Claude Code sessions, implementing eval-driven development (EDD) principles.

## Philosophy

Eval-Driven Development treats evals as the "unit tests of AI development":
- Define expected behavior BEFORE implementation
- Run evals continuously during development
- Track regressions with each change
- Use pass@k metrics for reliability measurement

## Runnable scripts

Two measurements here are executable rather than templates. Both need only
Python 3.9+ and the `claude` CLI. Paths below are relative to this skill's own
directory — prefix them with wherever it lives (`~/.claude/skills/eval-harness/`
when installed, `src/skills/eval-harness/` in the hibi-ai repo).

### Does a skill's description actually trigger?

```bash
python3 scripts/trigger_eval.py --skill do-178c \
  --eval-set ../do-178c/evals/trigger-eval.json --timeout 300 --jobs 4
```

The eval set is a JSON list of `{"query": ..., "should_trigger": true|false}`,
kept beside the skill it measures as `<skill>/evals/trigger-eval.json`. It is a
tracked input, not a run artifact, so `workspace/` — gitignored — cannot hold it:
a set nobody else can clone makes the regression gate unrepeatable.
Each query runs in a nested `claude -p` and the stream is searched for a
`Skill` tool_use whose `input.skill` equals the skill's **directory** name —
that is what the runtime emits, not the frontmatter `name:`.

Two gates it gives you, and why a harness without them yields numbers that
look like measurements but are not:

- **Bidirectional self-test.** Before scoring anything it proves a known
  positive fires and a known negative does not. Either failing exits 2 and
  reports no score at all. A detector that cannot report "dirty" proves
  nothing by reporting "clean".
- **Completion tracking.** Only a run that reached its own answer — a `result`
  event with subtype `success` — can testify that the skill was *not* chosen. A
  timeout emits no `result` at all, and `error_max_turns` means the turn budget
  ran out first. An **unfired** row that ended either way is INCONCLUSIVE, never
  PASS and never FAIL. A row that **fired** still scores by expectation even if
  the run died afterwards — firing is evidence no later failure retracts, so a
  slow positive is a PASS, not a re-run. Without this, every should-NOT query
  passes vacuously.

`--max-turns` (default 6) is the setting most likely to fabricate failures.
The nested session inherits your `CLAUDE.md`, so it spends early turns on the
pre-work checks that file mandates — `git status` and the like — before it ever
weighs a skill. Before the subtype rule existed, `--max-turns 2` made whole
eval sets read as FAIL with `tools=["Bash","Bash"], skills=[]`; under the
current rule those rows return INCONCLUSIVE (exit 3), or exit 2 if the gate
probe dies the same way. Either code is a turn-budget artifact, not a verdict on
the description — treat any `error_max_turns` row as a measurement that did not
happen and re-run it with a larger budget.

Exit status: 0 clear, 1 some FAIL, 2 self-test gate failed, 3 some
INCONCLUSIVE, 4 the harness could not run (no `claude` on PATH, unusable eval
set). Re-run INCONCLUSIVE rows serially with a longer `--timeout`, or a larger
`--max-turns` when the subtype says so, before quoting a figure — a partial
batch is not a score.

It measures the *installed* description, never the working copy: the nested
session reads `~/.claude/skills/`, and a repo path like `src/skills/` is not a
location Claude Code loads from. Install or sync the edited skill first, or the
run scores the previous description.

Do not substitute the `skill-creator` plugin's `run_eval` for this: it matches
a `<name>-skill-<uuid>` string the runtime never emits, gives up when the
first tool call is not Skill/Read, and discards nested stderr — so it returns
a stable, plausible, meaningless number. All three are already open upstream
with patches, filed independently: `anthropics/skills` #1419 (uuid match),
#1559 (first tool call, cites `run_eval.py:137-141` and `150-154`), #1478
(discarded failure scored as a verdict). Nothing here is worth re-filing.

### Skill-listing budget

```bash
python3 scripts/skill_budget.py ~/.claude/skills   # what the model actually sees
python3 scripts/skill_budget.py src/skills         # a repo tree before installing
```

The model-facing skill listing has a hard character budget,
`floor(context_window * 4 * 0.01)` = 8,000 at a 200K window. Over budget the
truncation is **all-or-nothing per skill**: whichever skills do not fit keep
only `- name` and lose their description, so they stop auto-triggering
entirely. Exits 1 when over (2 on a usage error, so a bad path never reads as
over budget), and flags descriptions past the 200-char authoring target plus any
`name:` that disagrees with its directory.

Two facts it encodes, both of which change how descriptions get written:

- `when_to_use` is concatenated onto `description` and measured as one string,
  so moving trigger vocabulary there saves nothing.
- Lengths count UTF-16 units, so a Hangul syllable costs the same as an ASCII
  letter — Korean trigger vocabulary is budget-efficient.

## Eval Types

### Capability Evals
Test if Claude can do something it couldn't before:
```markdown
[CAPABILITY EVAL: feature-name]
Task: Description of what Claude should accomplish
Success Criteria:
  - [ ] Criterion 1
  - [ ] Criterion 2
  - [ ] Criterion 3
Expected Output: Description of expected result
```

### Regression Evals
Ensure changes don't break existing functionality:
```markdown
[REGRESSION EVAL: feature-name]
Baseline: SHA or checkpoint name
Tests:
  - existing-test-1: PASS/FAIL
  - existing-test-2: PASS/FAIL
  - existing-test-3: PASS/FAIL
Result: X/Y passed (previously Y/Y)
```

## Grader Types

### 1. Code-Based Grader
Deterministic checks using code:
```bash
# Check if file contains expected pattern
grep -q "export function handleAuth" src/auth.ts && echo "PASS" || echo "FAIL"

# Check if tests pass
npm test -- --testPathPattern="auth" && echo "PASS" || echo "FAIL"

# Check if build succeeds
npm run build && echo "PASS" || echo "FAIL"
```

### 2. Model-Based Grader
Use Claude to evaluate open-ended outputs:
```markdown
[MODEL GRADER PROMPT]
Evaluate the following code change:
1. Does it solve the stated problem?
2. Is it well-structured?
3. Are edge cases handled?
4. Is error handling appropriate?

Score: 1-5 (1=poor, 5=excellent)
Reasoning: [explanation]
```

### 3. Human Grader
Flag for manual review:
```markdown
[HUMAN REVIEW REQUIRED]
Change: Description of what changed
Reason: Why human review is needed
Risk Level: LOW/MEDIUM/HIGH
```

## Metrics

### pass@k
"At least one success in k attempts"
- pass@1: First attempt success rate
- pass@3: Success within 3 attempts
- Typical target: pass@3 > 90%

### pass^k
"All k trials succeed"
- Higher bar for reliability
- pass^3: 3 consecutive successes
- Use for critical paths

## Eval Workflow

### 1. Define (Before Coding)
```markdown
## EVAL DEFINITION: feature-xyz

### Capability Evals
1. Can create new user account
2. Can validate email format
3. Can hash password securely

### Regression Evals
1. Existing login still works
2. Session management unchanged
3. Logout flow intact

### Success Metrics
- pass@3 > 90% for capability evals
- pass^3 = 100% for regression evals
```

### 2. Implement
Write code to pass the defined evals.

### 3. Evaluate
```bash
# Run capability evals
[Run each capability eval, record PASS/FAIL]

# Run regression evals
npm test -- --testPathPattern="existing"

# Generate report
```

### 4. Report
```markdown
EVAL REPORT: feature-xyz
========================

Capability Evals:
  create-user:     PASS (pass@1)
  validate-email:  PASS (pass@2)
  hash-password:   PASS (pass@1)
  Overall:         3/3 passed

Regression Evals:
  login-flow:      PASS
  session-mgmt:    PASS
  logout-flow:     PASS
  Overall:         3/3 passed

Metrics:
  pass@1: 67% (2/3)
  pass@3: 100% (3/3)

Status: READY FOR REVIEW
```

## Integration Patterns

### Pre-Implementation
```
/eval define feature-name
```
Creates eval definition file at `.claude/evals/feature-name.md`

### During Implementation
```
/eval check feature-name
```
Runs current evals and reports status

### Post-Implementation
```
/eval report feature-name
```
Generates full eval report

## Eval Storage

Store evals in project:
```
.claude/
  evals/
    feature-xyz.md      # Eval definition
    feature-xyz.log     # Eval run history
    baseline.json       # Regression baselines
```

## Best Practices

1. **Define evals BEFORE coding** - Forces clear thinking about success criteria
2. **Run evals frequently** - Catch regressions early
3. **Track pass@k over time** - Monitor reliability trends
4. **Use code graders when possible** - Deterministic > probabilistic
5. **Human review for security** - Never fully automate security checks
6. **Keep evals fast** - Slow evals don't get run
7. **Version evals with code** - Evals are first-class artifacts

## Example: Adding Authentication

```markdown
## EVAL: add-authentication

### Phase 1: Define (10 min)
Capability Evals:
- [ ] User can register with email/password
- [ ] User can login with valid credentials
- [ ] Invalid credentials rejected with proper error
- [ ] Sessions persist across page reloads
- [ ] Logout clears session

Regression Evals:
- [ ] Public routes still accessible
- [ ] API responses unchanged
- [ ] Database schema compatible

### Phase 2: Implement (varies)
[Write code]

### Phase 3: Evaluate
Run: /eval check add-authentication

### Phase 4: Report
EVAL REPORT: add-authentication
==============================
Capability: 5/5 passed (pass@3: 100%)
Regression: 3/3 passed (pass^3: 100%)
Status: SHIP IT
```
