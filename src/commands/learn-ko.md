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

실제 skill로 쓴다: `~/.claude/skills/<kebab-case-name>/SKILL.md`. frontmatter 없는
평평한 파일은 로드되지 않으므로, 그렇게 저장한 패턴은 그대로 버려진다 — 디렉터리와
frontmatter가 있어야 발견된다.

`description` 이 트리거 여부를 결정한다. 한두 문장으로 쓰고 마지막에 사용자가 실제로
입력할 표현을 한국어까지 넣는다 — 짧은 명령형 description은 한국어 질의에서 트리거
성능이 떨어진다. 아래의 `<...>` 는 모두 실제 내용으로 바꾼다. placeholder가 남은
파일은 로드는 되면서 아무 질의에도 걸리지 않는다.

```markdown
---
name: <kebab-case-name>
description: <무엇을 하고 언제 쓰는지, 마지막에 실제 트리거 표현>
keywords: [<english-terms>, <한국어용어>]
---

# <패턴 이름>

## Problem
<이 패턴이 막아주는 실패. 무엇이 어떻게 잘못됐는지 구체적으로>

## Solution
<다시 유도하지 않고 바로 적용할 수 있게 서술한 패턴>

## Example
<도움이 되면 코드>

## When to Use
<트리거 조건, 그리고 쓰지 말아야 할 때>
```

## Process

1. 추출 가능한 패턴을 찾기 위해 세션을 검토한다
2. 가장 가치 있고 재사용 가능한 인사이트를 식별한다
3. skill 파일 초안을 작성한다
4. 저장 전 사용자에게 확인을 요청한다
5. `~/.claude/skills/<name>/SKILL.md`에 저장한다
6. 이 기기를 넘어 다른 사용자에게도 도움이 되는 패턴이면 `/upstream-pr` 을 실행해 배포 설정에 제안한다 — 배포 시에는 `src/skills/<name>/SKILL.md` 와 `-ko.md` 쌍으로 들어간다

## Notes

- 사소한 수정(오타, 단순한 문법 오류)은 추출하지 않는다
- 일회성 이슈(특정 API 장애 등)는 추출하지 않는다
- 향후 세션에서 시간을 절약할 수 있는 패턴에 집중한다
- skill을 집중적으로 유지한다 — skill 하나당 패턴 하나
