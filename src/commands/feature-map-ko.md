---
description: Build or refresh docs/FEATURES.md, a feature map indexed by screen region, user words, and literal UI strings, so a vague report lands on the right files.
argument-hint: "[update]"
allowed-tools: Read, Write, Edit, Grep, Glob, Bash
model: sonnet
effort: high
---

# Feature Map

프로젝트의 증상→코드 기능맵을 작성하는 얇은 진입점이다. 맵이 없으면 사용자에게
보이는 표면에서 안쪽으로 새로 만든다. 맵이 이미 있거나 `$ARGUMENTS` 가 `update`
이면 기록된 SHA부터 diff를 떠서, 파일이 바뀐 항목만 다시 검증한다.

호출 방법: 처음 한 번 실행해 맵을 만들고, 이후 기능이 배포되거나 UI 문구가 바뀐
뒤에 실행한다. 결과물은 `docs/FEATURES.md` 이고, 커지면 `docs/features/` 로 분할한다.

`CLAUDE.md` / `AGENTS.md` 에 넣을 포인터 한 줄을 제안하되 사용자가 동의할 때만
추가한다.

필수 원칙: 모든 경로는 이번 실행에서 읽은 파일에서 나오고, 모든 실제 UI 문구는
소스에서 grep되며, 모든 함정은 근거를 인용한다. 추측하지 않는다.

**전체 방법론과 문서 템플릿은 `feature-map` skill을 source of truth로 삼아 따른다.**
