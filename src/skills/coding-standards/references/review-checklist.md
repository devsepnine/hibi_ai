# Review Checklist

> Reference for the `coding-standards` skill. Emphasizes SOLID, Clean Code,
> Functionality, Consistency, plus security, testing, performance.

Give SOLID, Clean Code, functionality, and consistency equal weight. Rate each finding by the contract affected and the impact of the defect.

## Severity Legend

Tag every finding with one of the labels below so the author knows what blocks
merge vs. what can land as a follow-up.

- **Blocker**. A defect that must be resolved before merging, such as an incorrect result, a security defect, a data race, a broken compatibility contract, or a hard limit exceeded without an applicable exception.
- **Major**. A demonstrable maintainability problem or a significant verification gap in changed code. Describe the impact and the necessary fix.
- **Minor**. An improvement suitable for follow-up. Identify the concrete benefit of a naming or documentation change.

When severity is uncertain, distinguish confirmed impact from conditions that still need verification.

## Code Quality

### Size limits

Check project limits and exceptions before applying the defaults below. [Code thresholds](code-thresholds.md)
- [ ] File size ≤ 300 LOC soft / ≤ 500 LOC hard
- [ ] Function size ≤ 50 LOC soft / ≤ 80 LOC hard
- [ ] Parameters ≤ 5 soft / ≤ 7 hard
- [ ] Cyclomatic complexity ≤ 10 soft / ≤ 15 hard
- [ ] Nesting depth ≤ 4 soft / ≤ 6 hard
- [ ] Split/refactor if hard limits exceeded; discuss if soft exceeded

### SOLID Principles
- [ ] **S**ingle Responsibility: each module/class/function changes for exactly one reason
- [ ] At an extension point for independent implementations, existing callers keep working. For a closed set owned by the domain, adding a case includes reviewing existing branches.
- [ ] **L**iskov Substitution: subtypes honor the contract of their supertype, with no surprising overrides
- [ ] **I**nterface Segregation: callers don't depend on methods they never use
- [ ] **D**ependency Inversion: high-level modules depend on abstractions such as a trait or interface, not concretions

### Clean Code
- [ ] Intention-revealing names; avoid `data`, `tmp`, and single-letter loops except indices
- [ ] Each function does one thing at one level of abstraction
- [ ] Side effects such as I/O, network, and mutation of shared state isolated to boundary layers
- [ ] Guard clauses preferred over deep nesting
- [ ] Constants symbolized: no magic numbers/strings, no hardcoded paths
- [ ] Code structured as Input → Processing → Return
- [ ] Code is the spec: no comment restates what the code already says. Such comments are replaced by a refactor, by rename or extract, and the comments that remain carry only the *why*
- [ ] Comments lead with the conclusion, meaning a one-sentence point and then why, explain non-obvious intent, and match current behavior
- [ ] No over-commenting: meaningless or excessive comments are flagged as noise. Comment density is not quality
- [ ] Comments moved with the code they describe: after an extract/split/rename no doc is left on the wrong declaration, and no two doc blocks are stacked on one
- [ ] Comments outside the diff hunk still hold: renamed symbols and changed behaviour are not still described at call sites, in module headers, in sibling files, or in docs
- [ ] Every *why* comment's claim rests on a code path, config, or external source you can point at. An unfounded rationale is a defect even when the code it sits on is correct
- [ ] No dead code, no commented-out blocks, no `TODO` without ticket

## Functionality Review

- [ ] Correctly implements requirements end-to-end
- [ ] Edge cases handled: empty, null, max, concurrent, partial failure
- [ ] Error handling specific and actionable; no catch-all that swallows context
- [ ] No unintended side effects in unrelated modules
- [ ] Behavior parity verified against previous version when refactoring

### Failure paths in self-review and PR review

Apply this pass after coding and when reviewing a PR, even when CI and success tests pass.

1. Identify the changed behavior, its callers, and the assumptions needed for success. Use `tdd-workflow` → Design failure cases before code to select relevant, reachable failure scenarios. Explain when none applies.
2. Trace each selected failure from its trigger through the real operation to the caller. Check error propagation, success indicators, persisted and in-memory state, resource cleanup, and any promised recovery. A catch block alone does not establish correct failure behavior.
3. Compare the observed or traced result with the contract. For partial work, check the declared atomicity or per-item policy. Do not invent rollback, retries, or other requirements. Raise an undefined failure policy as an open decision.
4. Check whether a test or reproduction reaches that failure and asserts the promised outcome and state after failure. A mock that rejects, a green build, or a coverage percentage is not sufficient evidence by itself. Reuse adequate existing tests and add only checks justified by the change and risk.
5. Distinguish demonstrated defects, test gaps, and unverified risks. A defect finding names the reachable trigger, affected code location, incorrect outcome, and supporting evidence. Report unexecuted checks as limits, not as passed or as proven bugs.

- [ ] Relevant failure scenarios reviewed alongside success, with evidence or explicit limits
- [ ] State preservation, cleanup, partial effects, and recovery checked where the contract requires them

## Concurrency and resource ownership

Trace the work and resources that the change starts or uses.

- [ ] Identify who observes completion, handles cancellation, and releases resources at shutdown.
- [ ] Where code uses channels, pipes, or locks, check that waiting can finish if a producer or consumer exits early.
- [ ] Identify who can mutate shared state and check that access follows the project's synchronization model.

## Rust error handling

- [ ] Return recoverable errors with the information callers need to distinguish outcomes.
- [ ] Check the local invariant behind each use of `unwrap` or `expect`.
- [ ] Preserve diagnostic context and exclude credentials when displaying or logging an error.

## Consistency Review

- [ ] Follows project coding conventions for naming, formatting, and error pattern
- [ ] Uses the same solution pattern as adjacent code; no two ways to solve one problem
- [ ] Logging style, correlation IDs, and error message shape match the surrounding module
- [ ] Naming convention of `snake_case` / `camelCase` / `PascalCase` aligned with language idiom and project standard
- [ ] API / response shape matches existing schemas; no ad-hoc fields
- [ ] Dependency additions fit the existing stack; no duplicate libraries for the same job
- [ ] Documentation tone and structure match surrounding docs

## Supported platforms

Identify supported environments from the project's release configuration and documentation. If a changed operation differs across those environments, verify the contract on the affected targets.

- [ ] Build paths in the form callers expect. Exercise real paths when symlinks or case sensitivity affect their meaning.
- [ ] Preserve executable and argument boundaries. If paths with spaces or empty arguments are supported, check what the child receives.
- [ ] Apply permission and line ending conversions according to the artifact format and file comparison contract.

## Security Review

- [ ] No secrets in code
- [ ] Inputs validated and sanitized
- [ ] No SQL injection vulnerabilities
- [ ] No XSS vulnerabilities
- [ ] Authentication/authorization applied
- [ ] See the `security-review` skill for the full checklist

## Testing Review

- [ ] New code has tests
- [ ] Bug fixes have regression tests
- [ ] Tests are deterministic
- [ ] E2E has success and failure paths
- [ ] See the `tdd-workflow` skill for the full checklist

## Performance Review

- [ ] No obvious performance issues
- [ ] Database queries optimized
- [ ] No N+1 query problems
- [ ] Appropriate caching considered

## Documentation Review

- [ ] Complex logic is documented
- [ ] API changes documented
- [ ] README updated if needed
- [ ] Breaking changes noted

## Reviewer Actions

1. **Read**: Understand the context and purpose
2. **Verify**: Check against requirements
3. **Test**: Run tests locally if needed
4. **Comment**: Provide constructive feedback
5. **Approve/Request Changes**: Make clear decision
