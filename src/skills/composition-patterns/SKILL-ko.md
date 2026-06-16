---
name: vercel-composition-patterns
description: Scalable React composition — compound components, context, state lifting, render props to replace boolean-prop proliferation; includes React 19 API changes. Use when designing reusable component APIs or refactoring prop-heavy components. 컴포지션 패턴, React 컴포넌트 설계, 컴파운드 컴포넌트.
keywords: [composition, 컴포지션, compound-components, react, 컴포넌트설계, render-props]
license: MIT
metadata:
  author: vercel
  version: '1.0.0'
---

# React 컴포지션 패턴

유연하고 유지보수 가능한 React 컴포넌트를 만드는 컴포지션 패턴. compound
component, 상태 끌어올리기, 내부 컴포지션을 사용해 boolean prop 증식을
방지한다. 이 패턴들은 코드베이스가 확장될 때 사람과 AI 에이전트 모두에게
작업하기 쉬운 코드를 만든다.

## 적용 시점

다음 상황에서 이 가이드라인을 참조한다:

- 다수의 boolean prop을 가진 컴포넌트 리팩토링
- 재사용 가능한 컴포넌트 라이브러리 구축
- 유연한 컴포넌트 API 설계
- 컴포넌트 아키텍처 리뷰
- compound component 또는 context provider 작업

## 우선순위별 규칙 카테고리

| Priority | Category                | Impact | Prefix          |
| -------- | ----------------------- | ------ | --------------- |
| 1        | Component Architecture  | HIGH   | `architecture-` |
| 2        | State Management        | MEDIUM | `state-`        |
| 3        | Implementation Patterns | MEDIUM | `patterns-`     |
| 4        | React 19 APIs           | MEDIUM | `react19-`      |

## 빠른 참조

### 1. Component Architecture (HIGH)

- `architecture-avoid-boolean-props` - 동작 커스터마이징을 위한 boolean
  prop 추가 금지; 컴포지션 사용
- `architecture-compound-components` - 공유 context로 복잡한 컴포넌트
  구조화

### 2. State Management (MEDIUM)

- `state-decouple-implementation` - Provider만이 상태 관리 방법을 알아야
  하는 유일한 곳
- `state-context-interface` - 의존성 주입을 위해 state, actions, meta로
  제네릭 인터페이스 정의
- `state-lift-state` - 형제 접근을 위해 상태를 provider 컴포넌트로 이동

### 3. Implementation Patterns (MEDIUM)

- `patterns-explicit-variants` - boolean 모드 대신 명시적 variant
  컴포넌트 생성
- `patterns-children-over-render-props` - renderX prop 대신 children
  으로 컴포지션

### 4. React 19 APIs (MEDIUM)

> **⚠️ React 19+ 전용.** React 18 이하 사용 시 이 섹션은 건너뛴다.

- `react19-no-forwardref` - `forwardRef` 사용 금지; `useContext()` 대신 `use()` 사용
- `react19-actions` - Actions + `useActionState` / `useFormStatus` / `useOptimistic`로 폼을 구성한다; 직접 만든 loading/error 배선은 제거

## 사용 방법

자세한 설명과 코드 예제는 개별 규칙 파일을 참조한다:

```
rules/architecture-avoid-boolean-props.md
rules/state-context-interface.md
```

각 규칙 파일은 다음을 포함한다:

- 왜 중요한지에 대한 간단한 설명
- 잘못된 코드 예제와 설명
- 올바른 코드 예제와 설명
- 추가 컨텍스트 및 참고 자료

## 전체 컴파일된 문서

모든 규칙이 펼쳐진 전체 가이드: `AGENTS.md`
