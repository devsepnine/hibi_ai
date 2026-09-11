# Dependency Design

소프트웨어를 수정 가능하고 AI가 소유할 수 있는 상태로 유지하는 dependency·coupling·추상화 결정.

상류에서 가져온 rule set이다. 진입점은 `SKILL.md`이고, 규칙 본체는 `rules/`에
rule 당 파일 하나로 들어 있다. 상류의 컴파일 산출물 `AGENTS.md`는 가져오지
않는다 — `rules/`가 원본이고 `SKILL.md`가 색인이다.

## 구성

- `SKILL.md` / `SKILL-ko.md` — 진입점: 적용 시점, 카테고리별 rule 색인
- `rules/` — rule 당 파일 하나
  - `_sections.md` — 섹션 순서, 파일명 prefix, impact 등급, 설명
  - `_template.md` — rule 파일 템플릿 (frontmatter 형태, Incorrect/Correct 형식)
  - `<prefix>-<topic>.md` — 개별 rule
- `references/` — 필요할 때 읽는 심화 방법론 (복잡성, coupling 모델, 추상화, AI 소유권, monorepo)
- `evals/` — `evals.json`은 출력 품질을, `trigger-eval.json`은 설명이 발화하는지를 측정

## Impact 등급

- `CRITICAL` — 기반이 되는 의존성 방향·coupling 규칙. 위반하면 유지보수도 격리도 불가능한 코드가 된다
- `HIGH` — 수정 가능성이나 AI 소유 가능성을 크게 높인다 (추상화 일관성, 단방향 레이어링)
- `MEDIUM` — 파급을 줄이고 경계를 명확히 하는 좋은 관행

## rule 추가하기

1. `rules/_template.md`를 `rules/<prefix>-<topic>.md`로 복사한다.
2. `rules/_sections.md`에 선언된 prefix를 쓴다. 맞는 것이 없으면 먼저 그 파일에 섹션을 추가한다.
3. `SKILL.md`와 `SKILL-ko.md`의 rule 색인에 추가한다 (빌드 단계가 없으므로 수동이다).

`-ko` 파일에서 각 rule의 `title`과 `impactDescription`, 그리고 섹션 제목은 영문으로
유지하고 본문 산문만 번역한다. `SKILL.md`가 그 영문 제목으로 rule을 색인하므로,
제목을 번역하면 색인이 어긋난다.
