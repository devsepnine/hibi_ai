# Testing Requirements

## 최소 테스트 커버리지: 80%

테스트 유형 (모두 필수):
1. **Unit Tests** - 개별 함수, 유틸리티, 컴포넌트
2. **Integration Tests** - API 엔드포인트, 데이터베이스 작업
3. **E2E Tests** - 핵심 사용자 플로우 (Playwright)

## 테스트 주도 개발

필수 워크플로우:
1. 테스트 먼저 작성 (RED)
2. 테스트 실행 — 실패해야 한다 (FAIL)
3. 최소 구현 작성 (GREEN)
4. 테스트 실행 — 통과해야 한다 (PASS)
5. 리팩토링 (IMPROVE)
6. 커버리지 검증 (80%+)

## 테스트 실패 트러블슈팅

1. **tdd-guide** 에이전트 사용
2. 테스트 격리(isolation) 확인
3. mock이 올바른지 검증
4. 테스트가 잘못된 경우가 아니라면, 테스트가 아니라 구현을 수정한다

## 에이전트 지원

- **tdd-guide** — 신규 기능에 적극(PROACTIVE) 사용. 테스트 우선 작성을 강제
- **e2e-runner** — Playwright E2E 테스트 전문
