---
description: Find what a diff could break beyond itself before it merges, and prove the one fact it is safe because of by running real code.
argument-hint: "[file | symbol | PR number]"
allowed-tools: Read, Write, Grep, Glob, Bash, Agent
model: sonnet
effort: high
---

# Blast Radius

머지 전 파손 추적을 위한 얇은 진입점이다. `$ARGUMENTS`가 파일, 심볼, PR 번호 중 하나인 대상을
지정한다. 없으면 커밋되지 않은 diff를, 그다음 `git diff main...HEAD`를 쓴다.

타협 불가: 변경이 안전한 근거가 되는 사실 하나를 밝히고, 그것을 증거 사다리를 따라
실제 코드를 실행하는 스크립트나 테스트까지 끌어내린다. 그렇지 못하면 `UNPROVEN`으로 표시한다.
호출자나 API를 지어내지 않고, 증명 스크립트는 커밋하지 않는다.

**전체 방법, 증거 사다리, 보고 형식은 `blast-radius` skill에 있다. 그것을 단일 진실 원천으로 따른다.**
