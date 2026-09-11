---
description: Extract reusable patterns from the current session into a new skill — its description, trigger vocabulary, and progressive-disclosure body. NOT for writing a note about what you learned — 학습 노트 작성은 obsidian-notes.
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

트리거 여부를 결정하는 필드는 `description` 하나뿐이고, 이 필드는 고정된 예산을 두고
경쟁한다. 설치된 모든 스킬의 `name` + `description` 합계가 약 8,000자(컨텍스트 윈도우의
1%) 안에 들어가야 한다. 예산을 넘기면 들어가지 못한 스킬은 description이 **통째로**
사라지고 이름만 남는다 — 자동 트리거가 아예 불가능해진다. 따라서:

- **200자 이하를 목표로, 상한 220자.** 여기서 쓰지 않은 산문이 다른 모든 스킬의 트리거
  신뢰도가 된다.
- **`when_to_use:` 로 옮겨도 절약되지 않는다** — 같은 예산 안에서 `description` 뒤에
  이어붙여진다. `keywords:` 는 스키마가 받아주지만 무시되며, 아무 역할도 하지 않는다.
- **한국어를 유지한다.** 한글은 음절 하나가 1자이고(`코드리뷰` 4 vs `code review` 11)
  사용자가 실제로 입력하는 표현이다. 짧은 영문 명령형 description은 한국어 질의에서
  측정 가능하게 실패한다.

네 부분을 이 순서로 — 네 번째는 혼동될 만한 형제 스킬이 실제로 있을 때만:

```
<what it does: compressed noun phrase> Use when <trigger condition>. <한국어 트리거 어휘>. NOT for <confusable skill>.
```

템플릿의 `<...>` 는 모두 실제 내용으로 바꾼다. placeholder가 남은 파일은 로드는 되면서
아무 질의에도 걸리지 않는다.

```markdown
---
name: <kebab-case-name>
description: <what it does> Use when <trigger condition>. <한국어 트리거 어휘>.
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
