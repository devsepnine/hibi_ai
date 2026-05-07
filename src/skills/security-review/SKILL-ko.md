---
name: security-review
description: Use this skill when adding authentication, handling user input, working with secrets, creating API endpoints, or implementing payment/sensitive features. Provides comprehensive security checklist and patterns. 보안 검토, 시큐리티 리뷰, 보안 취약점 점검, 인증/인가 검토, 보안 체크리스트.
keywords: [security, 보안, auth, 인증, vulnerability, 취약점, owasp]
---

# Security Review Skill

코드가 보안 베스트 프랙티스를 따르고 취약점을 식별하도록 보장한다. **OWASP Top 10 (2021)**에 매핑되며 전반적으로 **CWE** ID를 참조한다.

## 활성화 시점

인증/인가 변경, 사용자 입력 처리, 파일 업로드, 새 API 엔드포인트, 시크릿/자격증명, 결제 흐름, 민감 데이터 저장/전송, 서드파티 API 통합, 블록체인/지갑 작업, 모든 프로덕션 배포 직전.

## 핵심 안전 가드 (절대 위반 금지)

- **NEVER** 소스/로그/커밋 히스토리에 시크릿(API 키, 비밀번호, 토큰, DB 문자열)을 하드코딩한다.
- **NEVER** 사용자 입력을 SQL, shell, HTML에 concat 한다.
- **NEVER** 민감 데이터(비밀번호, 토큰, 전체 카드번호, PII, 클라이언트로 가는 stack trace)를 로그에 남긴다.
- **NEVER** 클라이언트 사이드 검증만 신뢰한다 — 항상 서버에서 재검증한다.
- **NEVER** 수신자, 금액, 잔액 검증 없이 블록체인 트랜잭션에 서명한다.
- **ALWAYS** 신뢰 경계를 넘는 모든 입력을 validate, normalize, encode 한다.
- **ALWAYS** 모든 상태 변경 전에 인증과 인가를 둘 다 수행한다.

## 보안 체크리스트 (10개 카테고리)

### 1. Secrets Management — OWASP A02 Cryptographic Failures / CWE-798, CWE-259

- [ ] 소스에 하드코딩된 API 키, 토큰, 비밀번호, DB URL 없음
- [ ] 모든 시크릿은 env var(`process.env.X`); 누락 시 fail fast
- [ ] `.env*`는 `.gitignore`에 포함; git history에 부재 확인 (`git log -p`)
- [ ] 프로덕션 시크릿은 호스팅 플랫폼(Vercel/Railway/AWS Secrets Manager)에, 레포에는 두지 않음
- [ ] 노출 의심 시 즉시 rotate
- Refs: [CWE-798](https://cwe.mitre.org/data/definitions/798.html), [CWE-259](https://cwe.mitre.org/data/definitions/259.html)

### 2. Input Validation — OWASP A03 Injection / A04 Insecure Design / CWE-20

```typescript
import { z } from 'zod'
const CreateUserSchema = z.object({ email: z.string().email(), name: z.string().min(1).max(100) })
const validated = CreateUserSchema.parse(input) // throws on invalid
```

- [ ] 모든 사용자 입력은 schema(Zod / Yup / Joi)로 검증
- [ ] **Whitelist** 허용값, blacklist 금지
- [ ] 파일 업로드: 크기 cap(예: 5MB), MIME type, 확장자 allowlist를 모두 강제
- [ ] 쿼리, 경로, shell 명령, template string에 raw 사용자 입력 금지
- [ ] 에러 응답이 schema/내부 필드명을 노출하지 않음
- Refs: [CWE-20](https://cwe.mitre.org/data/definitions/20.html), [CWE-434](https://cwe.mitre.org/data/definitions/434.html) (file upload)

### 3. SQL Injection Prevention — OWASP A03 / CWE-89

- [ ] 모든 쿼리는 parameterize (`$1`, `?`, ORM binding) — 절대 string concat 금지
- [ ] Supabase/Prisma/Drizzle 쿼리 빌더 정확히 사용 (사용자 입력에 `.raw()` 금지)
- [ ] Stored procedure는 bind 파라미터 사용
- 예: `db.query('SELECT * FROM users WHERE email = $1', [email])` — 사용자 입력을 인라인 보간하지 않는다
- Refs: [CWE-89](https://cwe.mitre.org/data/definitions/89.html)

### 4. Authentication & Authorization — OWASP A01 Broken Access Control / A07 Auth Failures / CWE-285, CWE-287

- [ ] 토큰은 **httpOnly + Secure + SameSite=Strict** 쿠키에 (localStorage 금지 — XSS 취약)
- [ ] 모든 민감 작업 **이전에** 인가 검사 (역할 + 소유권)
- [ ] Supabase: 모든 테이블에 **Row Level Security (RLS)** 활성화; 정책 테스트
- [ ] 세션 만료, 권한 변경 시 rotation, 로그아웃 시 서버 측 무효화
- [ ] 비밀번호 해싱: 적절한 cost factor의 bcrypt/argon2; MD5/SHA1 절대 금지
- [ ] admin/payment 경로는 MFA 제공
- Refs: [CWE-285](https://cwe.mitre.org/data/definitions/285.html), [CWE-287](https://cwe.mitre.org/data/definitions/287.html), [CWE-639](https://cwe.mitre.org/data/definitions/639.html) (IDOR)

### 5. XSS Prevention — OWASP A03 / CWE-79

- [ ] 사용자 제공 HTML은 엄격한 tag/attr allowlist의 DOMPurify로 sanitize
- [ ] Raw-HTML React props (예: inner-HTML setter)는 sanitize된 출력만 받음
- [ ] Content Security Policy 헤더 설정 (`default-src 'self'`, `script-src` 제한)
- [ ] `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY` 헤더
- [ ] 사용자 제어 URL 검증 (`javascript:` 또는 `data:` scheme 금지)
- Refs: [CWE-79](https://cwe.mitre.org/data/definitions/79.html), [MDN CSP](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)

### 6. CSRF Protection — OWASP A01 / CWE-352

- [ ] 모든 상태 변경 요청에 CSRF 토큰(또는 double-submit cookie)
- [ ] 세션 쿠키에 `SameSite=Strict`
- [ ] mutation 엔드포인트는 `POST/PUT/DELETE` 필수 (`GET` 금지)
- [ ] 민감 작업은 Origin / Referer 헤더 검증
- Refs: [CWE-352](https://cwe.mitre.org/data/definitions/352.html), [OWASP CSRF Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html)

### 7. Rate Limiting — OWASP A04 / CWE-770, CWE-307

- [ ] 전역 API rate limit (예: IP당 100 req / 15분)
- [ ] 비싼 작업(검색, AI 추론, 파일 처리)은 **더 엄격하게**
- [ ] 로그인: N회 실패 후 progressive backoff 또는 lockout (CWE-307 brute force)
- [ ] 해당되는 경우 IP 기반과 사용자 기반 limiting 둘 다
- [ ] `429 Too Many Requests`와 `Retry-After` 헤더 반환
- Refs: [CWE-770](https://cwe.mitre.org/data/definitions/770.html), [CWE-307](https://cwe.mitre.org/data/definitions/307.html)

### 8. Sensitive Data Exposure — OWASP A02 / A09 Logging Failures / CWE-532, CWE-209

- [ ] 로그는 비밀번호, 토큰, 전체 PAN, CVV, SSN, JWT body를 **redact**
- [ ] 안전한 식별자만 로그: `userId`, `last4`, `requestId`
- [ ] 클라이언트 에러 응답은 generic (`"An error occurred"`); 상세는 서버 측에만
- [ ] 클라이언트 응답에 stack trace, SQL 에러, 내부 경로 금지
- [ ] 프로덕션 HTTPS 강제 (HSTS 헤더)
- [ ] 민감 데이터는 저장 시 암호화 (DB 레벨 또는 앱 레벨)
- Refs: [CWE-532](https://cwe.mitre.org/data/definitions/532.html), [CWE-209](https://cwe.mitre.org/data/definitions/209.html)

### 9. Blockchain Security (Solana / Web3) — CWE-345, CWE-347

- [ ] 접근 허용 전 서명으로 지갑 소유권 검증 (`@solana/web3.js verify`)
- [ ] 트랜잭션 수신자 주소를 기대값과 비교 검증
- [ ] 트랜잭션 금액을 사용자별/세션별 cap에 대해 검사
- [ ] 트랜잭션 제출 전 사용자 잔액을 서버에서 검증
- [ ] **Blind signing 금지** — UI는 전체 tx 상세를 사용자에게 표시
- [ ] Replay 보호: nonce / 최근 blockhash 검증
- Refs: [CWE-345](https://cwe.mitre.org/data/definitions/345.html), [CWE-347](https://cwe.mitre.org/data/definitions/347.html)

### 10. Dependency Security — OWASP A06 Vulnerable Components / CWE-1104, CWE-937

- [ ] `npm audit` / `pnpm audit` 클린 (high/critical 없음)
- [ ] Lock file(`package-lock.json` / `pnpm-lock.yaml`) 커밋
- [ ] CI는 재현성을 위해 `npm install`이 아니라 `npm ci` 사용
- [ ] Dependabot / Renovate 활성화
- [ ] 주기적 `npm outdated` 검토; 보안 패치 신속 적용
- Refs: [CWE-1104](https://cwe.mitre.org/data/definitions/1104.html), [npm audit docs](https://docs.npmjs.com/cli/v10/commands/npm-audit)

## Security Testing (Required)

```typescript
test('requires authentication',  async () => expect((await fetch('/api/protected')).status).toBe(401))
test('requires admin role',      async () => expect((await fetch('/api/admin', { headers:{ Authorization:`Bearer ${userToken}` } })).status).toBe(403))
test('rejects invalid input',    async () => expect((await fetch('/api/users', { method:'POST', body: JSON.stringify({ email:'bad' }) })).status).toBe(400))
test('enforces rate limits',     async () => { const rs = await Promise.all(Array(101).fill(0).map(() => fetch('/api/x'))); expect(rs.some(r => r.status === 429)).toBe(true) })
```

필수 커버리지: authn, authz, validation, rate limit 각각에 대해 success path + failure path.

## 배포 전 보안 체크리스트

모든 프로덕션 배포 전, 위 10개 카테고리 모두 통과 + 다음:

- [ ] HTTPS 강제 (HSTS); CORS allowlist 명시 (credentialed에는 `*` 금지)
- [ ] 보안 헤더: CSP, X-Frame-Options, X-Content-Type-Options, Referrer-Policy
- [ ] 개발 중 노출 경로가 있었다면 시크릿 rotate
- [ ] 고위험 기능(결제, 인증, 관리자)에 대한 침투 테스트 또는 보안 리뷰 sign-off
- [ ] 사고 대응 runbook 갱신; on-call 통보

## OWASP Top 10 (2021) → 카테고리 매핑

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

## 자료

- [OWASP Top 10 (2021)](https://owasp.org/www-project-top-ten/)
- [OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [Next.js Security](https://nextjs.org/docs/app/building-your-application/configuring/content-security-policy)
- [Supabase Auth & RLS](https://supabase.com/docs/guides/auth)
- [Web Security Academy (PortSwigger)](https://portswigger.net/web-security)

---

**Remember**: 보안은 선택이 아니다. 단 하나의 취약점이 전체 플랫폼을 위험에 빠뜨릴 수 있다. 의심될 때는 신중하게 판단하고 **security-reviewer** 에이전트를 호출하라.
