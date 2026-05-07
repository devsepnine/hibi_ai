---
description: Analyze test coverage and generate tests for under-covered files. Targets 80%+ coverage threshold.
allowed-tools: Bash, Read, Write, Edit
model: haiku
effort: low
---

# Test Coverage

테스트 커버리지를 분석하고 누락된 테스트를 생성한다.

1. 커버리지와 함께 테스트 실행: npm test --coverage 또는 pnpm test --coverage

2. 커버리지 리포트 분석 (coverage/coverage-summary.json)

3. 80% 커버리지 임계값 미만의 파일 식별

4. 커버리지가 부족한 각 파일에 대해:
   - 테스트되지 않은 코드 경로 분석
   - 함수에 대한 단위 테스트 생성
   - API에 대한 통합 테스트 생성
   - 핵심 흐름에 대한 E2E 테스트 생성

5. 새로운 테스트 통과 검증

6. 변경 전/후 커버리지 메트릭 표시

7. 프로젝트가 80% 이상의 전체 커버리지에 도달하도록 보장

다음에 집중한다:
- Happy path 시나리오
- 오류 처리
- Edge cases (null, undefined, empty)
- 경계 조건
