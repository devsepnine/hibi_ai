---
description: Security review of a change against the 10-category checklist — secrets, input validation, injection, authn/authz, XSS, CSRF, rate limits, data exposure, blockchain/web3, dependencies
argument-hint: "[path|scope]"
allowed-tools: Read, Grep, Glob, Bash
model: sonnet
effort: high
---

# Security Review

Thin entry point for a security pass. Loads the `security-review` skill and audits the target named in `$ARGUMENTS` — or, with no argument, the uncommitted diff (`git diff --name-only HEAD`).

**Four non-negotiable guards (always apply, never strip):**

1. **No hardcoded secrets** — API keys, passwords, tokens, DB strings in source, logs, or commit history. If one is found, stop and report the exact location before anything else.
2. **No user input concatenated** into SQL, shell, or HTML.
3. **No sensitive data logged** — passwords, tokens, full card numbers, PII, stack traces returned to clients.
4. **Authenticate AND authorize before every state change** — client-side validation is never sufficient on its own.

Report findings by severity (CRITICAL / HIGH / MEDIUM / LOW) with `file:line`, the OWASP/CWE mapping, and the minimal fix. Never approve code with a CRITICAL or HIGH finding open.

This command is also the required A-tier sign-off in `/do-178c` for auth, payments, secrets, and input-handling changes.

**Full checklist, OWASP mapping, and required security tests live in the `security-review` skill — follow that as the source of truth.**
