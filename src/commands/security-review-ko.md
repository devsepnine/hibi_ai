---
description: Security review of a change against the 10-category checklist — secrets, input validation, injection, authn/authz, XSS, CSRF, rate limits, data exposure, blockchain/web3, dependencies
argument-hint: "[path|scope]"
allowed-tools: Read, Grep, Glob, Bash
model: sonnet
effort: high
---

# Security Review

보안 점검을 위한 얇은 진입점이다. `security-review` skill을 로드하고 `$ARGUMENTS`가 지정한 대상을 감사한다. 인자가 없으면 커밋되지 않은 diff(`git diff --name-only HEAD`)를 대상으로 한다.

**필수 준수 4원칙 (항상 적용, 절대 제거 금지):**

1. **시크릿 하드코딩 금지** — API 키, 비밀번호, 토큰, DB 접속 문자열이 소스·로그·커밋 이력에 있으면 안 된다. 발견 시 다른 작업보다 먼저 중단하고 정확한 위치를 보고한다.
2. **사용자 입력을 SQL·셸·HTML에 문자열 연결하지 않는다.**
3. **민감 데이터를 로그에 남기지 않는다** — 비밀번호, 토큰, 전체 카드번호, PII, 클라이언트로 반환되는 스택 트레이스.
4. **모든 상태 변경 전에 인증과 인가를 확인한다** — 클라이언트 측 검증만으로는 절대 충분하지 않다.

심각도별(CRITICAL / HIGH / MEDIUM / LOW)로 `file:line`, OWASP/CWE 매핑, 최소 수정안을 포함해 보고한다. CRITICAL 또는 HIGH가 열려 있는 코드는 절대 승인하지 않는다.

이 커맨드는 `/do-178c`에서 auth, payments, secrets, 입력 처리 변경에 대한 A-tier 필수 sign-off이기도 하다.

**전체 체크리스트, OWASP 매핑, 필수 보안 테스트는 `security-review` skill을 source of truth로 삼아 따른다.**
