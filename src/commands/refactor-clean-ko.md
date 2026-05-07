---
description: Safely identify and remove dead code with test verification. Runs knip/depcheck/ts-prune, categorizes findings, deletes only after tests pass.
allowed-tools: Bash, Read, Edit, Grep
model: haiku
effort: low
---

# Refactor Clean

테스트 검증과 함께 데드 코드를 안전하게 식별하고 제거한다.

1. 데드 코드 분석 도구 실행:
   - knip: 사용되지 않는 export 및 파일 탐지
   - depcheck: 사용되지 않는 의존성 탐지
   - ts-prune: 사용되지 않는 TypeScript export 탐지

2. .reports/dead-code-analysis.md에 종합 보고서 생성

3. 심각도별로 발견 사항을 분류한다:
   - SAFE: 테스트 파일, 사용되지 않는 유틸리티
   - CAUTION: API 라우트, 컴포넌트
   - DANGER: 설정 파일, 메인 엔트리 포인트

4. 안전한 삭제만 제안한다

5. 각 삭제 전에:
   - 전체 테스트 스위트를 실행한다
   - 테스트 통과를 확인한다
   - 변경을 적용한다
   - 테스트를 재실행한다
   - 테스트 실패 시 롤백한다

6. 정리된 항목 요약 표시

테스트를 먼저 실행하지 않고는 절대로 코드를 삭제하지 않는다!
