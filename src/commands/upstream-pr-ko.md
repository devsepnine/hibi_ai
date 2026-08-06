---
description: Promote session-derived improvements into the distributed config as a reviewable PR — detect drift between installed config and src/, gate by blast radius, and open the PR with its evidence.
argument-hint: "[topic]"
allowed-tools: Read, Write, Edit, Grep, Glob, Bash
model: sonnet
effort: high
---

# Upstream PR

세션에서 얻은 개선점을 모든 installer 사용자에게 배포되는 `src/`로 환류하기 위한
얇은 진입점이다. PR의 핵심은 diff가 아니라 **증거와 영향 범위**다. 무엇이 실패했고
누구에게 도달하는지 모르면 리뷰어는 규칙 변경을 판단할 수 없다.

## 후보 수집

- **Drift**: 설치본과 저장소를 비교한다 — `diff -r ~/.claude/skills <repo>/src/skills`, `commands`·`agents`·`CLAUDE.md`·`AGENTS.md`도 동일하게. 로컬에만 있고 `src/`에 없으면 후보다. 반대로 `src/`에만 있고 로컬에 없으면 그것은 설치 누락이지 후보가 아니다.
- **세션 증거**: 사용자가 교정한 내용, 필요한 시점에 없던 지침, 배포 문서에서 발견된 결함(잘못된 경로, 서로 모순되는 규칙).
- `$ARGUMENTS` 가 있으면 그 주제로 범위를 좁힌다.

## 후보를 게이트에 통과시킨다

네 가지를 모두 통과해야 배포한다. 후보별로 판정을 밝힌다 — 탈락 사유도 승인만큼 유용하다.

1. **일반화 가능성** — 다른 스택·프로젝트 사용자에게도 이득인가? 저장소 한정 지침은 그 프로젝트의 `CLAUDE.md`로, 개인 취향은 `MEMORY.md`로 간다. 둘 다 `src/` 대상이 아니다.
2. **증거** — 이번 세션에서 실제로 무엇이 실패했거나 교정됐는지 인용할 수 있는가? 흔적 없는 의견은 PR이 아니라 메모다.
3. **티어 적합성** — `CLAUDE.md`/`AGENTS.md` 는 모든 사용자의 모든 세션에 로드된다(B 티어): 증거, 검토한 대안 최소 1개, 동작하는 가장 짧은 문구를 요구한다. 스킬·커맨드·에이전트는 조건부 로드다(C/D 티어): 문턱이 낮다.
4. **되돌릴 수 있는가** — 문구는 깔끔히 revert되지만, 사용자 습관을 바꾸는 규칙은 릴리즈된 뒤에는 완전히 회수되지 않는다. 회수가 어려울수록 문턱을 높인다.

## 커밋 전에 저장소 규약을 강제한다

- `src/` 의 모든 파일은 영문 원본과 `-ko.md` 쌍을 갖고, 설치되는 것은 영문뿐이다 — 따라서 KO 쌍이 `-ko` asset 경로를 참조하면 안 된다.
- 상세 정책은 스킬 한 곳에만 두고, `CLAUDE.md`/`AGENTS.md` 에는 그것을 가리키는 한 줄만 넣는다. 양쪽에 상세를 중복하는 것이 피해야 할 실패 모드다.
- 이모지 금지, 생성 마커 금지.

## PR을 올린다

개선은 `improve/<topic>` 브랜치로 가고 `main` 직커밋은 하지 않는다. 커밋은 논리 단위로
하나씩. PR 본문에는 변경마다 증거, 티어와 도달 범위, 기각한 대안, 롤백 방법을 적는다.
`src/` 는 패키지에 임베드되므로 머지된 변경은 사용자의 다음 `hibi --sync` 가 아니라
**다음 릴리즈** 때 도달한다 — 그래서 PR이 마지막 리뷰 게이트다.

`git push` 와 PR 생성 전에 사용자에게 확인받는다. 둘 다 외부로 나가는 행위이고,
이 커맨드를 실행한 것만으로 승인된 것이 아니다.

**PR 제목·본문 규약은 `pull-request` skill을 source of truth로 삼아 따른다. 변경 요약 섹션은 `qa-handoff` 를, 패턴을 새 스킬로 뽑는 작업은 `/learn` 을 재사용한다.**
