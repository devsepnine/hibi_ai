# Dependency Design

바이브코딩 소프트웨어를 수정 가능하고 AI가 소유 가능하게 유지하는 의존성, 결합, 추상화 의사결정을 위한 구조화된 저장소이다. 이 방법론은 문제의 이해 가능성을 분류하고, 그에 맞는 결합 강도를 매칭하고, 단방향 의존성을 강제하고, 추상화를 일관되게 유지한다.

## Structure

- `SKILL.md` / `SKILL-ko.md` - Skill 진입점: when-to-apply 표, core decision flow, references로의 링크. `-ko` 파일은 byte-identical frontmatter를 유지한 채 본문을 한국어로 옮긴 것이다.
- `references/` - 필요할 때 읽는 심화 방법론:
  - `complexity.md` - 복잡성, 이해 가능성, Cynefin framework
  - `coupling-models.md` - module / connascence / domain coupling 모델
  - `abstraction.md` - interface/implementation/context, 추상화 일관성
  - `ai-ownership.md` - 부분적이고 컨텍스트로 제한된 AI 소유를 위한 코드 구조화
  - `monorepo.md` - 레이어 추상화와 Turbo monorepo 단방향 의존성
- `rules/` - 개별 rule 파일 (rule당 하나):
  - `_sections.md` - 섹션 메타데이터 (section 접두사, 제목, impact level)
  - `_template.md` - 새 rule 작성용 템플릿
  - `<section>-<topic>.md` - 개별 rule 파일
- `metadata.json` - 문서 메타데이터 (version, organization, abstract, references)
- **`AGENTS.md`** - 컴파일된 출력 (`rules/`에서 생성됨)
- `evals/` - skill이 올바르게 트리거되고 적용되는지 검증하는 eval 케이스

## Creating a New Rule

1. `rules/_template.md`를 `rules/<section>-<topic>.md`로 복사한다.
2. `rules/_sections.md`에 선언된 section 접두사를 선택한다 (각 접두사는 section 제목과 impact level에 매핑된다). 기존 접두사가 맞지 않으면 먼저 `rules/_sections.md`에 새 섹션을 추가한 뒤 그 접두사를 사용한다.
3. frontmatter와 내용을 채운다. 명확한 **Incorrect** 예시(무엇이 연쇄적으로 번지는지에 대한 설명 포함)와 **Correct** 예시(이제 왜 결합이 허용 가능하거나 단방향인지에 대한 설명 포함)를 작성한다.
4. 컴파일된 rule set이 새 파일을 반영하도록 `AGENTS.md`를 재생성한다.

> `-ko` 파일에서는 각 rule의 `title`, `impactDescription`, 그리고 섹션 제목을 영어로 유지한다(본문 산문만 번역). `AGENTS.md` / `AGENTS-ko.md`의 목차 앵커가 이 제목들로부터 생성되므로, 제목을 번역하면 앵커 링크가 깨진다.

## Impact Levels

- `CRITICAL` - 기본적인 의존성 방향 / 결합 규칙. 위반하면 유지보수 불가능하고 격리할 수 없는 코드가 된다.
- `HIGH` - 상당한 수정 가능성 또는 AI 소유 가능성 향상 (추상화 일관성, 단방향 레이어링).
- `MEDIUM` - 연쇄 영향을 줄이고 경계를 명확히 하는 좋은 관행.
