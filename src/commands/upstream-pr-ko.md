---
description: Promote a session-derived improvement into the distributed config as a reviewable PR — locate the upstream from the install manifest, gate candidates by blast radius, and open the PR with its evidence.
argument-hint: "[topic]"
allowed-tools: Read, Write, Edit, Grep, Glob, Bash
model: sonnet
effort: high
---

# Upstream PR

세션에서 얻은 것을 다른 사용자에게 배포되는 설정으로 되돌리기 위한 얇은 진입점이다.
업스트림은 `~/.hibi/install.json` 에서 확인하고(설치만 한 사용자는 clone이 없다),
installer가 관리하는 모든 디렉터리를 비교해 후보를 모으고, 게이트를 통과시킨 뒤
`improve/<topic>` 브랜치로 변경을 올린다.

호출 방법: 배포되는 규칙·스킬·에이전트·커맨드가 바뀌어야 한다는 것이 세션에서 드러났을
때, 또는 개선을 로컬에만 적용해 두어 다른 사용자에게 전달할 필요가 있을 때 실행한다.
`$ARGUMENTS` 가 있으면 그 주제로 범위를 좁힌다.

필수 원칙: 이 커맨드 실행은 로컬 브랜치와 커밋까지만 승인한다. 공유될 내용을 보여주고
`git push` 와 PR 생성 전에 **두 번째** 확인을 받는다 — 둘 다 외부로 나가는 행위이고,
세션 증거에는 먼저 다시 써야 할 고용주 코드나 내부 경로가 섞일 수 있다.

**전체 방법론 — 소스 확인, 후보 선별, 4단 게이트, PR 본문 구성 — 은 `upstream-pr` skill을 source of truth로 삼아 따른다.**
