# Sections

이 파일은 모든 섹션과 그 순서, 영향도 수준, 설명을 정의한다. 괄호 안의 섹션 ID는 rule을 그룹화하는 데 사용되는 파일명 접두사이다.

---

## 1. Component Architecture (architecture)

**Impact:** HIGH  
**Description:** prop 증식을 피하고 유연한 composition을 가능하게 하는 컴포넌트 구조화의 기본 패턴이다.

## 2. State Management (state)

**Impact:** MEDIUM  
**Description:** 조합된 컴포넌트 전반에 걸쳐 상태를 끌어올리고 공유 context를 관리하는 패턴이다.

## 3. Implementation Patterns (patterns)

**Impact:** MEDIUM  
**Description:** compound component와 context provider 구현을 위한 구체적인 기법이다.

## 4. React 19 APIs (react19)

**Impact:** MEDIUM  
**Description:** React 19+ only. `forwardRef`를 사용하지 않는다; `useContext()` 대신 `use()`를 사용한다.
