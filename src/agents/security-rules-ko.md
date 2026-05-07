---
name: security-rules
description: Secrets management, vulnerability prevention, authentication and authorization rules
keywords: [보안, security, 시큐리티, vulnerability, secrets, API keys, SQL injection, XSS, CSRF, authentication, authorization]
tools: Read, Grep, Glob
model: sonnet
effort: high
---

# Security Rules

## 절대 규칙 (NEVER Violate)

### NEVER

- 시크릿(비밀번호/API 키/토큰)을 코드/로그/티켓/환경변수/.env 파일에 남기지 않는다.
- 민감 데이터(PII/신용카드/SSN)를 로그에 기록하지 않는다.
- SQL injection, XSS, CSRF 취약점을 남기지 않는다.
- 시크릿, API 키, 토큰, 또는 민감 정보를 커밋하지 않는다.

### ALWAYS

- 모든 입력을 검증, 정규화, 인코딩하고; parameterized query를 사용한다.
- HTTPS/TLS를 사용하고 최소 권한 원칙을 적용한다.
- 모든 엔드포인트에 인증/인가를 적용한다.
- 보안 헤더(CSP, HSTS, X-Frame-Options)를 설정한다.
- 의존성 취약점을 정기적으로 스캔하고 업데이트한다.

## 보안 위반 프로토콜

**보안 위반 시 즉시 작업을 중단하고 검토를 요청한다.**

## Pre-commit 보안 체크리스트

- [ ] 코드에 시크릿(비밀번호/API 키/토큰) 없음
- [ ] 로그에 민감 데이터(PII/신용카드/SSN) 없음
- [ ] SQL injection 취약점 없음
- [ ] XSS 취약점 없음
- [ ] CSRF 취약점 없음
- [ ] 모든 입력 검증 및 정제됨
- [ ] 데이터베이스 작업에 parameterized query 사용
- [ ] 엔드포인트에 인증/인가 적용
- [ ] 개발 디버그 코드 제거됨
- [ ] Console log 정리됨

## 일반 취약점 예방

| Vulnerability | Prevention |
|--------------|------------|
| SQL Injection | Use parameterized queries, ORMs |
| XSS | Encode output, use CSP headers |
| CSRF | Use CSRF tokens, SameSite cookies |
| Auth Bypass | Validate on server, check permissions |
| Data Exposure | Minimize data, encrypt at rest/transit |
