---
description: Iteratively fix TypeScript and build errors. Parses error output, applies minimal fixes one at a time, verifies after each. Stops on regression.
allowed-tools: Bash, Read, Edit, Grep
model: haiku
effort: low
---

# Build and Fix

TypeScript 및 빌드 오류를 점진적으로 수정한다.

1. 빌드 실행: npm run build 또는 pnpm build

2. 오류 출력 파싱:
   - 파일별로 그룹화한다
   - 심각도순으로 정렬한다

3. 각 오류에 대해:
   - 오류 컨텍스트(앞뒤 5줄)를 표시한다
   - 문제를 설명한다
   - 수정안을 제시한다
   - 수정을 적용한다
   - 빌드를 재실행한다
   - 오류가 해결되었는지 검증한다

4. 다음의 경우 중단한다:
   - 수정이 새로운 오류를 유발하는 경우
   - 동일한 오류가 3회 시도 후에도 지속되는 경우
   - 사용자가 일시 중지를 요청한 경우

5. 요약 표시:
   - 수정된 오류
   - 남아있는 오류
   - 새로 발생한 오류

안전을 위해 한 번에 하나의 오류만 수정한다!
