# Security Rules

## 필수 보안 점검

모든 커밋 전에:
- [ ] 하드코딩된 시크릿이 없는지 (API 키, 비밀번호, 토큰)
- [ ] 모든 사용자 입력 검증 완료
- [ ] SQL injection 방지 (parameterized query)
- [ ] XSS 방지 (sanitize된 HTML)
- [ ] CSRF 보호 활성화
- [ ] 인증/인가 검증 완료
- [ ] 모든 엔드포인트에 rate limiting 적용
- [ ] 에러 메시지가 민감 정보를 노출하지 않음

## 보안 체크리스트 (요약)

* **절대 금지:** 코드/로그에 시크릿 하드코딩, 민감 데이터 노출, SQL Injection/XSS/CSRF 취약점 허용.
* **항상:** 입력 검증, parameterized query, 인증 점검을 사용한다.

## 시크릿 관리

```typescript
// NEVER: Hardcoded secrets
const apiKey = "sk-proj-xxxxx"

// ALWAYS: Environment variables
const apiKey = process.env.OPENAI_API_KEY

if (!apiKey) {
  throw new Error('OPENAI_API_KEY not configured')
}
```

## 보안 대응 프로토콜

보안 이슈 발견 시:
1. 즉시 STOP
2. **security-reviewer** 에이전트 사용
3. 진행 전에 CRITICAL 이슈를 먼저 수정
4. 노출된 시크릿은 모두 회전(rotate)
5. 유사 이슈가 다른 코드에도 있는지 코드베이스 전체를 검토
