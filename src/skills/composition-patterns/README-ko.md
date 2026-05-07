# React Composition Patterns

확장 가능한 React composition 패턴을 위한 구조화된 저장소이다. 이러한 패턴은 compound component, state lifting, 내부 구성요소 composition을 통해 boolean prop 증식을 피하는 데 도움이 된다.

## Structure

- `rules/` - 개별 rule 파일 (rule당 하나)
  - `_sections.md` - 섹션 메타데이터 (제목, 영향도, 설명)
  - `_template.md` - 새 rule 작성용 템플릿
  - `area-description.md` - 개별 rule 파일
- `metadata.json` - 문서 메타데이터 (version, organization, abstract)
- **`AGENTS.md`** - 컴파일된 출력 (생성됨)

## Rules

### Component Architecture (CRITICAL)

- `architecture-avoid-boolean-props.md` - 동작을 커스터마이즈하기 위해 boolean prop을 추가하지 않는다
- `architecture-compound-components.md` - 공유 context를 가진 compound component로 구조화한다

### State Management (HIGH)

- `state-lift-state.md` - 상태를 provider 컴포넌트로 끌어올린다
- `state-context-interface.md` - 명확한 context 인터페이스를 정의한다 (state/actions/meta)
- `state-decouple-implementation.md` - 상태 관리와 UI를 분리한다

### Implementation Patterns (MEDIUM)

- `patterns-children-over-render-props.md` - renderX prop보다 children을 선호한다
- `patterns-explicit-variants.md` - 명시적인 컴포넌트 변형을 만든다

## Core Principles

1. **Composition over configuration** — prop을 추가하는 대신 소비자가 조합하게 한다
2. **Lift your state** — 컴포넌트에 갇혀 있지 않고 provider에 상태를 둔다
3. **Compose your internals** — 하위 컴포넌트는 prop이 아닌 context에 접근한다
4. **Explicit variants** — isThread를 가진 Composer가 아니라 ThreadComposer, EditComposer를 만든다

## Creating a New Rule

1. `rules/_template.md`를 `rules/area-description.md`로 복사한다
2. 적절한 area 접두사를 선택한다:
   - `architecture-` Component Architecture용
   - `state-` State Management용
   - `patterns-` Implementation Patterns용
3. frontmatter와 내용을 채운다
4. 명확한 예시와 설명이 있는지 확인한다

## Impact Levels

- `CRITICAL` - 기본 패턴, 유지보수 불가능한 코드를 방지한다
- `HIGH` - 상당한 유지보수성 향상
- `MEDIUM` - 더 깔끔한 코드를 위한 좋은 관행
