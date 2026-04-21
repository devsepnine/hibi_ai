---
name: Review Checklist
description: Code review checklist emphasizing SOLID, Clean Code, Functionality, Consistency plus security, testing, performance
keywords: [리뷰, review, 검토, 레뷰, checklist, code quality, SOLID, clean code, functionality, consistency, security, testing, performance, documentation]
---

# Review Checklist

Review priorities (in order of enforcement weight): **SOLID → Clean Code →
Functionality → Consistency**, followed by Security / Testing / Performance /
Documentation. A change that scores well on later categories but fails any of
the four priority categories must be sent back for revision.

## Severity Legend

Tag every finding with one of the labels below so the author knows what blocks
merge vs. what can land as a follow-up.

- **Blocker** — must fix before merge. SOLID violations, security issues,
  broken functionality, data races, resource leaks, hard-limit overruns.
- **Major** — fix in this PR. Clean Code violations, Consistency breaks,
  missing tests for new code, soft-limit overruns with no plan.
- **Minor** — follow-up acceptable. Doc polish, naming taste, performance
  micro-opts, non-essential refactors.

If unsure, default up (Major → Blocker) rather than down.

## Code Quality

### Size Limits (see `rules/code-thresholds.md` for soft/hard tiers)
- [ ] File size ≤ 300 LOC soft / ≤ 500 LOC hard
- [ ] Function size ≤ 50 LOC soft / ≤ 80 LOC hard
- [ ] Parameters ≤ 5 soft / ≤ 7 hard
- [ ] Cyclomatic complexity ≤ 10 soft / ≤ 15 hard
- [ ] Nesting depth ≤ 4 soft / ≤ 6 hard
- [ ] Split/refactor if hard limits exceeded; discuss if soft exceeded

### SOLID Principles
- [ ] **S**ingle Responsibility: each module/class/function changes for exactly one reason
- [ ] **O**pen/Closed: behavior extends without modifying existing stable code
- [ ] **L**iskov Substitution: subtypes honor the contract of their supertype (no surprising overrides)
- [ ] **I**nterface Segregation: callers don't depend on methods they never use
- [ ] **D**ependency Inversion: high-level modules depend on abstractions (trait/interface), not concretions

### Clean Code
- [ ] Intention-revealing names (avoid `data`, `tmp`, single-letter loops except indices)
- [ ] Each function does one thing at one level of abstraction
- [ ] Side effects (I/O, network, mutation of shared state) isolated to boundary layers
- [ ] Guard clauses preferred over deep nesting
- [ ] Constants symbolized (no magic numbers/strings, no hardcoded paths)
- [ ] Code structured as Input → Processing → Return
- [ ] No dead code, no commented-out blocks, no `TODO` without ticket

## Functionality Review

- [ ] Correctly implements requirements end-to-end
- [ ] Edge cases handled (empty, null, max, concurrent, partial failure)
- [ ] Error handling specific and actionable (no catch-all that swallows context)
- [ ] No unintended side effects in unrelated modules
- [ ] Behavior parity verified against previous version when refactoring

## Concurrency & Resource Safety

This project uses Rust threads + `mpsc` channels + child processes. Review
every concurrent touchpoint before approving.

- [ ] Channel lifecycle explicit — `Sender`/`Receiver` drop points identified,
      no dangling producers after shutdown
- [ ] `cancel_rx` cooperative cancellation check placed between long phases
      (see `source::sync_all_sources` pattern)
- [ ] `JoinHandle` disposition explicit — either `join()` or intentionally
      detached with a one-line comment stating why
- [ ] No `Arc<Mutex<...>>` scope wider than necessary; hold the lock only
      across the critical section
- [ ] File handles / temp dirs use Drop-guard (own the cleanup, don't rely on
      the process exiting)
- [ ] Child processes have bounded lifetime — timeout or explicit kill on the
      cancel path; no orphaned zombies
- [ ] Spawned threads that read pipes exit naturally when the child is killed
      (no blocking reads past child death)
- [ ] Shared state between TUI tick and background thread goes through channels,
      not raw mutable references

## Error Handling (Rust)

- [ ] `unwrap` / `expect` only where an invariant is locally provable; panic
      messages name the invariant
- [ ] `anyhow::Context` attached at call boundaries so the chain reads like a
      narrative, not a stack trace
- [ ] Custom error types use `thiserror` with explicit variants; avoid
      `Box<dyn Error>` which loses match granularity
- [ ] `Result` is actually handled — no silent `let _ =` on fallible calls
      that touch external state
- [ ] TUI code never calls `eprintln!` (corrupts alternate screen); errors
      surface through `status_message` or typed channels
- [ ] Sensitive data (tokens, credentials) scrubbed from error messages
      before display or logging (`sanitize_stderr` pattern)

## Consistency Review

- [ ] Follows project coding conventions (naming, formatting, error pattern)
- [ ] Uses the same solution pattern as adjacent code (no two ways to solve one problem)
- [ ] Logging style, correlation IDs, and error message shape match the surrounding module
- [ ] Naming convention (snake_case / camelCase / PascalCase) aligned with language idiom and project standard
- [ ] API / response shape matches existing schemas (no ad-hoc fields)
- [ ] Dependency additions fit the existing stack (no duplicate libraries for the same job)
- [ ] Documentation tone and structure match surrounding docs

## Cross-platform Considerations

This project ships to macOS, Linux, and Windows (Homebrew + Scoop + source).
Any change that touches paths, commands, or file I/O must be reviewed against
all three.

- [ ] Path joins use `Path::join` / `PathBuf` — no hardcoded `/` or `\`
      separators; no string concatenation
- [ ] `canonicalize()` used when comparing paths that may contain symlinks
      (e.g., macOS `/tmp` → `/private/tmp`)
- [ ] MSYS / Cygwin path shapes (`/c/Users/...`) normalized via
      `normalize_git_path` before display or file ops on Windows
- [ ] Shell commands invoked via `Command::new("tool")` directly — **never**
      `cmd /c ...` (shell injection risk on Windows)
- [ ] User-provided command strings pass `is_safe_command` (blocks Unix
      `&|><;` and Windows `%^!` metacharacters)
- [ ] File permissions set explicitly when creating executables on Unix
      (`PermissionsExt::set_mode`); Windows ignores safely
- [ ] Line endings tolerated in comparisons (`normalize_line_endings` strips
      `\r` so CRLF vs LF does not flip "Modified" status)
- [ ] Hook/statusline paths derive from `dest_dir.file_name()` to support both
      `~/.claude` and `~/.codex` targets

## Security Review

- [ ] No secrets in code
- [ ] Inputs validated and sanitized
- [ ] No SQL injection vulnerabilities
- [ ] No XSS vulnerabilities
- [ ] Authentication/authorization applied
- [ ] See [security-rules.md](./security-rules.md) for full checklist

## Testing Review

- [ ] New code has tests
- [ ] Bug fixes have regression tests
- [ ] Tests are deterministic
- [ ] E2E has success and failure paths
- [ ] See [testing-rules.md](./testing-rules.md) for full checklist

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
