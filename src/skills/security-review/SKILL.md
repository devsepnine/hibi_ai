---
name: security-review
description: Use this skill when adding authentication, handling user input, working with secrets, creating API endpoints, or implementing payment/sensitive features. Provides comprehensive security checklist and patterns. 보안 검토, 시큐리티 리뷰, 보안 취약점 점검, 인증/인가 검토, 보안 체크리스트.
keywords: [security, 보안, auth, 인증, vulnerability, 취약점, owasp]
---

# Security Review Skill

Ensures code follows security best practices and identifies vulnerabilities. Maps to **OWASP Top 10 (2021)** and references **CWE** IDs throughout.

## When to Activate

Auth/authz changes, user input handling, file uploads, new API endpoints, secrets/credentials, payment flows, sensitive data storage/transmission, third-party API integration, blockchain/wallet ops, before any production deployment.

## Core Safety Guards (NEVER violate)

- **NEVER** hardcode secrets (API keys, passwords, tokens, DB strings) in source/logs/commit history.
- **NEVER** concatenate user input into SQL, shell, or HTML.
- **NEVER** log sensitive data (passwords, tokens, full card numbers, PII, stack traces to clients).
- **NEVER** trust client-side validation alone — always re-validate server-side.
- **NEVER** sign blockchain transactions without verifying recipient + amount + balance.
- **ALWAYS** validate, normalize, encode every input crossing a trust boundary.
- **ALWAYS** authenticate AND authorize before any state change.

## Security Checklist (10 Categories)

### 1. Secrets Management — OWASP A02 Cryptographic Failures / CWE-798, CWE-259

- [ ] No hardcoded API keys, tokens, passwords, DB URLs in source
- [ ] All secrets in env vars (`process.env.X`); fail fast if missing
- [ ] `.env*` files in `.gitignore`; verified absent from git history (`git log -p`)
- [ ] Production secrets in hosting platform (Vercel/Railway/AWS Secrets Manager), not repo
- [ ] Rotated immediately if exposure suspected
- Refs: [CWE-798](https://cwe.mitre.org/data/definitions/798.html), [CWE-259](https://cwe.mitre.org/data/definitions/259.html)

### 2. Input Validation — OWASP A03 Injection / A04 Insecure Design / CWE-20

```typescript
import { z } from 'zod'
const CreateUserSchema = z.object({ email: z.string().email(), name: z.string().min(1).max(100) })
const validated = CreateUserSchema.parse(input) // throws on invalid
```

- [ ] All user inputs validated with schema (Zod / Yup / Joi)
- [ ] **Whitelist** allowed values, never blacklist
- [ ] File uploads: enforce size cap (e.g. 5MB), MIME type, AND extension allowlist
- [ ] No raw user input in queries, paths, shell commands, or template strings
- [ ] Error responses don't leak schema/internal field names
- Refs: [CWE-20](https://cwe.mitre.org/data/definitions/20.html), [CWE-434](https://cwe.mitre.org/data/definitions/434.html) (file upload)

### 3. SQL Injection Prevention — OWASP A03 / CWE-89

- [ ] All queries parameterized (`$1`, `?`, ORM bindings) — never string concat
- [ ] Supabase/Prisma/Drizzle query builders used correctly (no `.raw()` with user input)
- [ ] Stored procedures use bind parameters
- Example: `db.query('SELECT * FROM users WHERE email = $1', [email])` — never inline-interpolate user input
- Refs: [CWE-89](https://cwe.mitre.org/data/definitions/89.html)

### 4. Authentication & Authorization — OWASP A01 Broken Access Control / A07 Auth Failures / CWE-285, CWE-287

- [ ] Tokens in **httpOnly + Secure + SameSite=Strict** cookies (NOT localStorage — XSS-vulnerable)
- [ ] Authorization checked **before every** sensitive operation (role + ownership)
- [ ] Supabase: **Row Level Security (RLS)** enabled on all tables; policies tested
- [ ] Session expiry, rotation on privilege change, logout invalidates server-side
- [ ] Password hashing: bcrypt/argon2 with proper cost factor; never MD5/SHA1
- [ ] MFA available for admin/payment paths
- Refs: [CWE-285](https://cwe.mitre.org/data/definitions/285.html), [CWE-287](https://cwe.mitre.org/data/definitions/287.html), [CWE-639](https://cwe.mitre.org/data/definitions/639.html) (IDOR)

### 5. XSS Prevention — OWASP A03 / CWE-79

- [ ] User-provided HTML sanitized via DOMPurify with strict tag/attr allowlist
- [ ] Raw-HTML React props (e.g. inner-HTML setters) only fed sanitized output
- [ ] Content Security Policy header set (`default-src 'self'`, restrict `script-src`)
- [ ] `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY` headers
- [ ] User-controlled URLs validated (no `javascript:` or `data:` schemes)
- Refs: [CWE-79](https://cwe.mitre.org/data/definitions/79.html), [MDN CSP](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)

### 6. CSRF Protection — OWASP A01 / CWE-352

- [ ] CSRF token (or double-submit cookie) on all state-changing requests
- [ ] `SameSite=Strict` on session cookies
- [ ] Mutation endpoints require `POST/PUT/DELETE` (never `GET`)
- [ ] Origin / Referer header verified for sensitive ops
- Refs: [CWE-352](https://cwe.mitre.org/data/definitions/352.html), [OWASP CSRF Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)

### 7. Rate Limiting — OWASP A04 / CWE-770, CWE-307

- [ ] Global API rate limit (e.g. 100 req / 15 min per IP)
- [ ] **Stricter** limits on expensive ops (search, AI inference, file processing)
- [ ] Login: progressive backoff or lockout after N failures (CWE-307 brute force)
- [ ] Both IP-based AND user-based limiting where applicable
- [ ] Returns `429 Too Many Requests` with `Retry-After` header
- Refs: [CWE-770](https://cwe.mitre.org/data/definitions/770.html), [CWE-307](https://cwe.mitre.org/data/definitions/307.html)

### 8. Sensitive Data Exposure — OWASP A02 / A09 Logging Failures / CWE-532, CWE-209

- [ ] Logs **redact** passwords, tokens, full PAN, CVV, SSN, JWT bodies
- [ ] Log only safe identifiers: `userId`, `last4`, `requestId`
- [ ] Client error responses are generic (`"An error occurred"`); details only server-side
- [ ] No stack traces, SQL errors, or internal paths in client responses
- [ ] HTTPS enforced in production (HSTS header)
- [ ] Sensitive data encrypted at rest (DB-level or app-level)
- Refs: [CWE-532](https://cwe.mitre.org/data/definitions/532.html), [CWE-209](https://cwe.mitre.org/data/definitions/209.html)

### 9. Blockchain Security (Solana / Web3) — CWE-345, CWE-347

- [ ] Wallet ownership verified via signature (`@solana/web3.js verify`) before granting access
- [ ] Transaction recipient address validated against expected
- [ ] Transaction amount checked against per-user / per-session caps
- [ ] User balance verified server-side before submitting tx
- [ ] **No blind signing** — UI displays full tx details to user
- [ ] Replay protection: nonces / recent blockhash validated
- Refs: [CWE-345](https://cwe.mitre.org/data/definitions/345.html), [CWE-347](https://cwe.mitre.org/data/definitions/347.html)

### 10. Dependency Security — OWASP A06 Vulnerable Components / CWE-1104, CWE-937

- [ ] `npm audit` / `pnpm audit` clean (no high/critical)
- [ ] Lock files (`package-lock.json` / `pnpm-lock.yaml`) committed
- [ ] CI uses `npm ci` (not `npm install`) for reproducibility
- [ ] Dependabot / Renovate enabled
- [ ] Periodic `npm outdated` review; security patches applied promptly
- Refs: [CWE-1104](https://cwe.mitre.org/data/definitions/1104.html), [npm audit docs](https://docs.npmjs.com/cli/v10/commands/npm-audit)

## Security Testing (Required)

```typescript
test('requires authentication',  async () => expect((await fetch('/api/protected')).status).toBe(401))
test('requires admin role',      async () => expect((await fetch('/api/admin', { headers:{ Authorization:`Bearer ${userToken}` } })).status).toBe(403))
test('rejects invalid input',    async () => expect((await fetch('/api/users', { method:'POST', body: JSON.stringify({ email:'bad' }) })).status).toBe(400))
test('enforces rate limits',     async () => { const rs = await Promise.all(Array(101).fill(0).map(() => fetch('/api/x'))); expect(rs.some(r => r.status === 429)).toBe(true) })
```

Required coverage: success path + failure path for each of authn, authz, validation, rate limit.

## Pre-Deployment Security Checklist

Before ANY production deployment, all 10 categories above must pass, plus:

- [ ] HTTPS enforced (HSTS); CORS allowlist explicit (no `*` for credentialed)
- [ ] Security headers: CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy
- [ ] Secrets rotated if any exposure path existed during development
- [ ] Penetration test or security review sign-off for high-risk features (payment, auth, admin)
- [ ] Incident response runbook updated; on-call notified

## OWASP Top 10 (2021) → Category Map

| OWASP            | This Skill's Categories      |
|------------------|------------------------------|
| A01 Broken Access Control     | 4 (Authz), 6 (CSRF) |
| A02 Cryptographic Failures    | 1 (Secrets), 8 (Data Exposure) |
| A03 Injection                 | 2 (Input), 3 (SQLi), 5 (XSS) |
| A04 Insecure Design           | 2 (Input), 7 (Rate Limit) |
| A05 Security Misconfiguration | 5 (CSP/Headers), Pre-Deploy |
| A06 Vulnerable Components     | 10 (Dependencies) |
| A07 Identification & Auth     | 4 (Authn) |
| A08 Software & Data Integrity | 9 (Blockchain), 10 (Lockfiles) |
| A09 Logging Failures          | 8 (Logging) |
| A10 SSRF                      | 2 (Input — URL validation) |

## Resources

- [OWASP Top 10 (2021)](https://owasp.org/www-project-top-ten/)
- [OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [Next.js Security](https://nextjs.org/docs/app/building-your-application/configuring/content-security-policy)
- [Supabase Auth & RLS](https://supabase.com/docs/guides/auth)
- [Web Security Academy (PortSwigger)](https://portswigger.net/web-security)

---

**Remember**: Security is not optional. One vulnerability can compromise the entire platform. When in doubt, err on the side of caution and invoke the **code-reviewer** agent.
