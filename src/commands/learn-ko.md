---
description: Extract reusable patterns from the current session into skills. Saves successful workflow patterns for future use.
allowed-tools: Read, Grep, Write
model: haiku
effort: low
---

# /learn - Extract Reusable Patterns

현재 세션을 분석하여 skill로 저장할 만한 패턴을 추출한다.

## Trigger

세션 중 비자명한 문제를 해결한 시점에 언제든 `/learn`을 실행한다.

## What to Extract

다음을 찾는다:

1. **Error Resolution Patterns**
   - 어떤 오류가 발생했는가?
   - 근본 원인은 무엇이었는가?
   - 무엇이 그것을 해결했는가?
   - 유사한 오류에 재사용 가능한가?

2. **Debugging Techniques**
   - 자명하지 않은 디버깅 단계
   - 효과적이었던 도구 조합
   - 진단 패턴

3. **Workarounds**
   - 라이브러리 특이사항
   - API 제약
   - 버전별 수정 사항

4. **Project-Specific Patterns**
   - 발견된 코드베이스 컨벤션
   - 결정된 아키텍처
   - 통합 패턴

## Output Format

`~/.claude/skills/learned/[pattern-name].md`에 skill 파일을 생성한다:

```markdown
# [Descriptive Pattern Name]

**Extracted:** [Date]
**Context:** [Brief description of when this applies]

## Problem
[What problem this solves - be specific]

## Solution
[The pattern/technique/workaround]

## Example
[Code example if applicable]

## When to Use
[Trigger conditions - what should activate this skill]
```

## Process

1. 추출 가능한 패턴을 찾기 위해 세션을 검토한다
2. 가장 가치 있고 재사용 가능한 인사이트를 식별한다
3. skill 파일 초안을 작성한다
4. 저장 전 사용자에게 확인을 요청한다
5. `~/.claude/skills/learned/`에 저장한다

## Notes

- 사소한 수정(오타, 단순한 문법 오류)은 추출하지 않는다
- 일회성 이슈(특정 API 장애 등)는 추출하지 않는다
- 향후 세션에서 시간을 절약할 수 있는 패턴에 집중한다
- skill을 집중적으로 유지한다 — skill 하나당 패턴 하나
