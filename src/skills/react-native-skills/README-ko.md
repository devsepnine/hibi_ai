# React Native Guidelines

에이전트와 LLM에 맞춰 정리한 React Native 성능 규칙.

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

- `CRITICAL` — 최우선. 크래시나 UI 깨짐을 유발한다
- `HIGH` — 성능을 상당히 개선한다
- `MEDIUM` — 중간 수준의 성능 개선
- `LOW` — 점진적 개선

## rule 추가하기

1. `rules/_template.md`를 `rules/<prefix>-<topic>.md`로 복사한다.
2. `rules/_sections.md`에 선언된 prefix를 쓴다. 맞는 것이 없으면 먼저 그 파일에 섹션을 추가한다.
3. `SKILL.md`와 `SKILL-ko.md`의 rule 색인에 추가한다 (빌드 단계가 없으므로 수동이다).
