# Code Thresholds

Project-wide complexity limits. Two-tier model: **Soft** targets guide design, **Hard** limits block merge.

## Threshold Limits

| Metric | Soft (target) | Hard (block) | Why | On Violation |
|--------|---------------|--------------|-----|--------------|
| File Length | ≤ 300 LOC | ≤ 500 LOC | Single responsibility, scan in one glance | Split module by concern |
| Function Length | ≤ 50 LOC | ≤ 80 LOC | Easier testing, naming, reuse | Extract helpers |
| Parameters | ≤ 5 | ≤ 7 | Call-site clarity, loose coupling | Introduce options struct / builder |
| Cyclomatic Complexity | ≤ 10 | ≤ 15 | Branch explosion caps testable cases | Early returns, strategy split |
| Nesting Depth | ≤ 4 | ≤ 6 | Linear reading flow | Guard clauses, extract function |

**Soft** (warning): Discuss in PR review, refactor if feasible.
**Hard** (error): Must refactor before merge, or document exception.

## Function Size > File Size

A 500 LOC file with small cohesive functions is healthier than a 250 LOC file with a single 200 LOC function. Prioritize:

1. **Function length** — strongest signal of complexity
2. **Cyclomatic complexity** — predicts test burden
3. **Nesting depth** — predicts reader cognitive load
4. **File length** — last, as splitting cohesive modules can hurt locality

## Measurement Rules

- **LOC**: Excludes blank lines and comment-only lines.
- **Parameters**: Counts positional + named + optional; destructured objects count as one.
- **Cyclomatic Complexity**: +1 each for `if`, `else if`, `match/case` arm, `&&`/`||`, `?:`, loop, `catch`.
- **Nesting Depth**: Brace depth inside function body (control blocks, closures).

## Allowed Exceptions

Exceptions require a top-of-file comment stating the reason. Hard limits still apply unless explicitly exempted:

- Auto-generated code (protobuf, OpenAPI, codegen stubs) — exempt from all
- Test fixtures / data tables — File LOC exempt, functions still apply
- Type definition files (types.rs, d.ts) — File LOC Hard raised to 800
- Unavoidable branch maps (state machines, route tables) — Complexity exempt
- One-shot migration scripts — exempt from all

## Enforcement Tools

- **Rust**: `cargo clippy -- -W clippy::cognitive_complexity -W clippy::too_many_arguments`, `tokei`
- **TypeScript**: ESLint `max-lines` (warn:300, error:500), `max-lines-per-function`, `complexity`, `max-params`
- **Python**: `radon cc`, `flake8 --max-complexity=10`

## Refactor Triggers

Apply the **Rule of Three**: repeating a pattern 3+ times signals extraction. Combined with Soft threshold breach, refactor immediately rather than deferring.

## Related Rules

- `coding-style.md` — General coding style and file organization
- `pull-request-rules.md` — PR checklist including these thresholds
- `development-workflow.md` — TDD and review workflow
