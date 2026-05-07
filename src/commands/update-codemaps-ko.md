---
description: Scan codebase and generate token-lean architecture codemaps. Detects >30% drift, requests user approval before update.
allowed-tools: Read, Write, Bash, Grep, Glob
model: sonnet
effort: medium
---

# Update Codemaps

코드베이스 구조를 분석하고 아키텍처 문서를 업데이트한다.

1. import, export, 의존성에 대해 모든 소스 파일을 스캔한다
2. 다음 형식으로 토큰 효율적인 codemap을 생성한다:
   - codemaps/architecture.md - 전체 아키텍처
   - codemaps/backend.md - 백엔드 구조
   - codemaps/frontend.md - 프론트엔드 구조
   - codemaps/data.md - 데이터 모델 및 스키마

3. 이전 버전 대비 diff 비율을 계산한다
4. 변경이 30%를 초과하면 업데이트 전 사용자 승인을 요청한다
5. 각 codemap에 신선도 타임스탬프를 추가한다
6. 보고서를 .reports/codemap-diff.txt에 저장한다

분석에는 TypeScript/Node.js를 사용한다. 구현 세부사항이 아닌 상위 수준 구조에 집중한다.
