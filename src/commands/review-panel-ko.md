---
description: Review an A/B-tier diff with a panel of code-reviewer agents, one lens each, then merge their findings into Act on, Consider, and Dismissed with a verdict.
argument-hint: "[file | PR number | base branch]"
allowed-tools: Agent, Read, Grep, Glob, Bash
model: sonnet
effort: high
---

# Review Panel

`CLAUDE.md` §9의 티어가 A나 B인 diff를 위한 독립 리뷰다. `$ARGUMENTS`는 파일, PR 번호, 기준 브랜치 중 하나를 대상으로 지정한다. 없으면 커밋되지 않은 diff를, 그다음 `git diff main...HEAD`를 쓴다. C에서 E 티어 diff라면 작업 후 `code-reviewer` 게이트 하나로 충분하다. 그렇게 말하고 멈춘다.

기본 리뷰 게이트는 여전히 `/code-review`나 `code-reviewer` agent이고, 이 패널은 A와 B 티어를 위한 추가 단계다. 독립성은 렌즈 하나씩을 맡은 별개의 agent에서 나온다. 산출물은 판정이다. 수정을 적용하거나 커밋하거나 푸시하지 않는다.

## 단계

1. **범위와 티어를 정한다.** diff와 리뷰어가 읽는 데 필요한 맥락 파일을 묶는다. 티어는 `do-178c` skill로 확인한다.
2. **의도를 한 문단으로 밝힌다.** 요청, 커밋 메시지, PR 본문, 코드에서 끌어온다. 의도가 불분명하면 진행하기 전에 묻는다.
3. **패널을 한 메시지에서 띄운다.** 리뷰어는 각각 `code-reviewer` agent이며 diff, 의도 문단, 자기 렌즈 하나를 받는다. 각자에게 의도가 아니라 실행을 판단하고, 찾은 것이 없으면 빈 목록을 돌려주라고 말한다.
   - **정확성과 데이터 손실**: 경계 사례, 오류 경로, 멱등성, 부분 쓰기, 동시 접근. 주장하는 실행 경로를 하나하나 추적한다.
   - **보안**: `security-review` skill을 따른다. 입력마다 출처에서 sink까지 추적한다.
   - **설계와 결합도**: `dependency-design` skill을 따른다.
   - **테스트**: `tdd-workflow`를 따른다. 동작이 깨지면 각 assertion이 실패하는지, 요구사항마다 테스트가 있는지 본다.
   - A 티어에서만 추적성과 구조 커버리지를 위해 `assurance-auditor`를 더한다.
4. **리드로서 병합한다.** 세션 전체 맥락을 쥔 쪽은 리드이므로, 모으지 말고 걸러서 판단한다.
   - 같은 문제를 다른 말로 묘사한 발견을 합치고, 각 발견을 어느 렌즈가 냈는지 기록한다.
   - 두 렌즈가 독립적으로 낸 발견은 남긴다.
   - 한 렌즈만 낸 발견은 남기기 전에 직접 코드를 추적해 다시 검증한다.
   - `code-reviewer`의 "Filter Before Reporting" 필터를 적용한다: 가정이 아닌 실제, 선호가 아닌 구체적 문제, 성급한 추상화 금지, 이 변경이 일으키거나 드러낸 것, nit을 경고로 부풀리지 않기.
   - 보안이나 정확성 발견은 경로를 추적하지 않고 버리지 않는다.
5. **A 티어라면** 사람의 리뷰를 요청하며 끝낸다. 패널이 그것을 대신하지 않는다.

## 응답

- **Intent**: 2단계의 문단.
- **Panel**: 렌즈별 발견 수.
- **Act on**: 많아야 5개 안팎. 각각 위치, 낸 렌즈, 머지를 막는 이유. 목록이 더 길면 필터가 느슨했다는 뜻이다.
- **Consider**: 지금 고치는 비용이 더 클 수 있는 실제 지적. 각각 트레이드오프와 함께.
- **Dismissed**: 사용자가 판단을 뒤집을 수 있도록 각각 한 줄로 이유를 적는다.
- **Verdict**: `code-reviewer` 승인 기준에 따른 `[APPROVE]`, `[WARN]`, `[BLOCK]`. A 티어라면 사람 리뷰 요청도 함께.
