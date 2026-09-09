# React Composition Patterns

확장되어도 유연함을 유지하는 React 컴포넌트 composition 패턴.

상류에서 가져온 rule set이다. 진입점은 `SKILL.md`이고, 규칙 본체는 `rules/`에
rule 당 파일 하나로 들어 있다. 상류의 컴파일 산출물 `AGENTS.md`는 가져오지
않는다 — `rules/`가 원본이고 `SKILL.md`가 색인이다.

## 구성

- `SKILL.md` / `SKILL-ko.md` — 진입점: 적용 시점, 카테고리별 rule 색인
- `rules/` — rule 당 파일 하나
  - `_sections.md` — 섹션 순서, 파일명 prefix, impact 등급, 설명
  - `_template.md` — rule 파일 템플릿 (frontmatter 형태, Incorrect/Correct 형식)
  - `<prefix>-<topic>.md` — 개별 rule
- `metadata.json` — 상류 문서 메타데이터 (버전, 요약, 참고 링크)

## Impact 등급

- `CRITICAL` — 기반 패턴. 유지보수 불가능한 코드를 막는다
- `HIGH` — 유지보수성을 크게 개선한다
- `MEDIUM` — 코드를 더 깔끔하게 만드는 좋은 관행

## rule 추가하기

1. `rules/_template.md`를 `rules/<prefix>-<topic>.md`로 복사한다.
2. `rules/_sections.md`에 선언된 prefix를 쓴다. 맞는 것이 없으면 먼저 그 파일에 섹션을 추가한다.
3. `SKILL.md`와 `SKILL-ko.md`의 rule 색인에 추가한다 (빌드 단계가 없으므로 수동이다).

## 핵심 원칙

1. **설정보다 composition** — prop을 추가하지 말고 소비자가 조합하게 한다
2. **state를 끌어올린다** — state는 provider에 두고 컴포넌트 안에 가두지 않는다
3. **내부를 조합한다** — 하위 컴포넌트는 prop이 아니라 context를 읽는다
4. **명시적 variant** — `isThread`를 받는 `Composer`가 아니라 `ThreadComposer` / `EditComposer`
