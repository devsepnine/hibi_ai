---
description: Comprehensive security and quality review of uncommitted changes. Checks hardcoded secrets, input validation, injection risks, code style violations.
allowed-tools: Read, Grep, Bash, Glob
model: sonnet
effort: medium
---

# Code Review

커밋되지 않은 변경 사항에 대한 종합적인 보안 및 품질 검토를 수행한다.

1. 변경된 파일 가져오기: git diff --name-only HEAD

2. 변경된 각 파일에 대해 다음을 확인한다:

**Security Issues (CRITICAL):**
- 하드코딩된 자격 증명, API 키, 토큰
- SQL 인젝션 취약점
- XSS 취약점
- 입력 검증 누락
- 안전하지 않은 의존성
- 경로 탐색(Path traversal) 위험

**Code Quality (HIGH):**
- 50줄을 초과하는 함수
- 800줄을 초과하는 파일
- 4단계를 초과하는 중첩 깊이
- 누락된 오류 처리
- console.log 문
- TODO/FIXME 주석
- 공개 API에 대한 JSDoc 누락

**Best Practices (MEDIUM):**
- 변이(Mutation) 패턴 (불변 패턴 사용 권장)
- 코드/주석에 이모지 사용
- 신규 코드에 대한 테스트 누락
- 접근성(a11y) 이슈

3. 다음을 포함한 보고서를 생성한다:
   - 심각도: CRITICAL, HIGH, MEDIUM, LOW
   - 파일 위치 및 라인 번호
   - 이슈 설명
   - 수정안 제안

4. CRITICAL 또는 HIGH 이슈 발견 시 커밋을 차단한다

보안 취약점이 있는 코드는 절대 승인하지 않는다!
