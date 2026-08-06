---
name: upstream-pr
description: Promote a session-derived improvement into the distributed config as a reviewable PR — find the upstream from the install manifest, separate real candidates from local noise, gate each one on generalizability, evidence, blast radius, and reversibility, then open the PR carrying that evidence. Use when a session reveals that a shipped rule, skill, agent, or command should change; when someone asks how to contribute a config improvement back; or when an improvement was applied locally and has to reach other users. 설정 개선 환류, 업스트림 PR, 배포 설정 기여, 세션 교훈 반영, 로컬 수정 업스트림, 기여 방법.
keywords: [upstream-pr, 업스트림PR, 설정개선, 기여, contribute, config-improvement, drift, 세션교훈, provenance, install-manifest, fork, PR]
---

# Upstream PR

세션에서 얻은 것을 다른 사용자에게 배포되는 설정으로 되돌린다. PR의 가치는 diff가
아니라 **증거와 영향 범위**다. 무엇이 실패했고 누구에게 도달하는지 모르면 리뷰어는
규칙 변경을 판단할 수 없다.

## 1. 소스 위치를 찾는다

대부분의 사용자는 설치만 하고 저장소를 clone하지 않는다. 체크아웃이 있다고 가정하지
않는다.

1. **`~/.hibi/install.json`** — installer가 남기는 매니페스트다. `source`(업스트림 URL), `version`(릴리즈 태그이므로 정확한 소스 트리를 복원 가능), `target`, installer가 관리하는 컴포넌트 목록이 들어 있다. 하드코딩된 URL보다 이 파일을 먼저 신뢰한다. 설치만 한 사용자에게 자기 설정의 출처를 알려주는 것도 이 파일이다.
2. **로컬 clone 있음** — `git remote -v` 가 그 source를 가리키면 그것을 쓴다.
3. **clone 없음** — 그 사실을 분명히 말하고, 매니페스트의 업스트림을 알려주고 `gh repo fork <owner>/<repo> --clone` 을 제안한다. 변경 내용은 이미 그 사람의 설치본에 있고, fork는 리뷰를 받기 위한 수단일 뿐이다.
4. **fork를 원하지 않음** — 제안 내용을 홈 디렉터리나 스크래치패드의 파일로 남기고 저장소 issues 페이지를 안내한다. 이슈에 남은 아이디어가 세션 종료와 함께 사라진 아이디어보다 낫다.
5. **매니페스트도 없음** (매니페스트 도입 전 설치) — `https://github.com/devsepnine/hibi_ai` 로 폴백하고, 재설치하면 다음부터 출처가 기록된다고 알려준다.

## 2. 후보를 수집한다

installer가 관리하는 **모든** 디렉터리를 비교한다. 일부만 보면 안 된다 — 에이전트나
output style에 대한 교정도 스킬만큼 업스트림 가치가 있다. Claude 대상은 `agents`,
`commands`, `contexts`, `rules`, `skills`, `output-styles`, `statusline`, `hooks`
그리고 설정 파일 `CLAUDE.md`·`settings.json` 이고, Codex 대상은 `skills` 와
`AGENTS.md` 다. installer가 실제로 무엇을 설치했는지는 매니페스트의 컴포넌트
목록을 기준으로 삼는다.

그다음 잡음을 걷어낸다. 그러지 않으면 실제 실행이 잡음에 묻힌다.

- **`-ko.md` 파일과 `*/workspace/` 하위 트리는 설계상 설치되지 않는다.** 모든 diff에서 "로컬에 없음"으로 나타난다. 이것은 누락이 아니라 의도된 제외다.
- **로컬 파일이 모두 이 저장소에서 온 것은 아니다.** `~/.hibi/sources.yaml` 로 다른 git·로컬 소스를 추가할 수 있고, 사용자가 직접 만든 스킬도 있다. 이 저장소의 설치 표면 밖에 있는 것은 아무리 좋아도 후보가 아니다. 매니페스트가 이미 이를 구분한다 — `components` 에는 bundled 소스에서 온 것만 담기고, 다른 소스는 `other_sources` 로 따로 표시된다.
- **세션 증거도 후보다**: 사용자가 교정한 내용, 필요한 시점에 없던 지침, 배포 문서에서 발견된 결함(잘못된 경로, 서로 모순되는 규칙).

## 3. 후보를 게이트에 통과시킨다

네 가지를 모두 통과해야 배포한다. 후보별로 판정을 밝힌다 — 탈락 사유도 승인만큼 유용하다.

1. **일반화 가능성** — 다른 스택·프로젝트 사용자에게도 이득인가? 저장소 한정 지침은 그 프로젝트의 `CLAUDE.md` 로, 개인 취향은 `MEMORY.md` 로 간다. 둘 다 업스트림 대상이 아니다.
2. **증거** — 실제로 무엇이 실패했거나 교정됐는지 인용할 수 있는가? 흔적 없는 의견은 PR이 아니라 메모다.
3. **티어 적합성** — `do-178c` skill의 A–E 척도로 티어를 부여한다. 항상 로드되는 지침(`CLAUDE.md`, `AGENTS.md`)은 모든 사용자의 모든 세션에 로드되므로 B에 해당한다: 증거, 검토한 대안 최소 1개, 동작하는 가장 짧은 문구를 요구한다. 조건부 로드되는 스킬·커맨드·에이전트는 문턱이 낮다.
4. **되돌릴 수 있는가** — 문구는 깔끔히 revert되지만, 사용자 습관을 바꾸는 규칙은 릴리즈된 뒤에는 완전히 회수되지 않는다. 회수가 어려울수록 문턱을 높인다.

## 4. 저장소 규약에 맞춘다

- **쌍을 맞춘다** — 모든 파일은 영문 원본과 `-ko.md` 쌍을 갖고, 설치되는 것은 영문뿐이다. 따라서 KO 쌍이 `-ko` asset 경로를 참조하면 안 된다.
- **단일 출처** — 상세 정책은 스킬 한 곳에만 두고, `CLAUDE.md`/`AGENTS.md` 에는 그것을 가리키는 한 줄만 넣는다. 양쪽에 상세를 중복하는 것이 피해야 할 실패 모드다.
- **Codex는 skills와 `AGENTS.md` 만 받는다** — Codex 사용자에게 필요한 방법론은 커맨드가 아니라 스킬에 둔다. 커맨드는 Claude 대상만 설치되기 때문이다. `AGENTS.md` 에서는 스킬을 이름으로 참조하고 슬래시 커맨드 문법을 쓰지 않는다.
- **`/learn` 산출물에는 쌍이 더 필요하다** — `/learn` 은 이미 frontmatter를 갖춘 `skills/<name>/SKILL.md` 를 설치 트리에 쓴다. 다만 배포본은 `src/skills/<name>/` 에 있고 `-ko.md` 쌍이 필요하다.
- 커밋 위생은 `commit-rules` skill을 따른다.

## 5. 무엇이든 밖으로 나가기 전에 묻는다

공유될 내용 — diff와 증거 문장 — 을 보여주고 업스트림에 올릴지 묻는다. 이것이 단계가
아니라 질문인 이유는 두 가지다.

- **증거는 실제 세션에서 나온다.** 고용주의 코드, 내부 경로, 티켓 ID, 고객명이 섞일 수 있다. 무엇이 실패했는지는 일반적인 형태로만 서술하고 비공개 세부는 절대 넣지 않는다. 그 세부 없이는 설명이 불가능하다면 증거를 먼저 다시 써야 한다고 말한다.
- **업스트림은 공개 저장소다.** 한번 push되면 나중에 브랜치를 지워도 내용은 공개된 상태로 남는다.

## 6. PR을 올린다

이 워크플로우를 실행하라는 요청 자체가 로컬 브랜치와 커밋을 승인하는 명시적 요청이다 —
`improve/<topic>`, 커밋은 논리 단위로 하나씩, `main` 직커밋은 하지 않는다. push와 PR
생성은 외부로 나가는 행위이므로 **두 번째** 확인이 따로 필요하고, 워크플로우를 실행한
것만으로는 승인되지 않는다.

PR 본문에는 변경마다 증거, 티어와 도달 범위, 기각한 대안, 롤백 방법을 적는다. 설정은
릴리즈 패키지에 임베드되므로 머지된 변경은 사용자의 다음 `hibi --sync` 가 아니라
**다음 릴리즈** 때 도달한다 — 그래서 PR이 마지막 리뷰 게이트다.

## 관련 스킬

| 필요한 것 | 위치 |
|---|---|
| PR 제목·본문 규약 | `pull-request` skill |
| PR 본문의 변경 요약 | `qa-handoff` skill |
| 티어 정의 (A–E) | `do-178c` skill |
| 패턴을 먼저 스킬로 추출 | `/learn` (승격 가능한 스킬을 생성; 승격 시 `-ko.md` 쌍 추가) |
