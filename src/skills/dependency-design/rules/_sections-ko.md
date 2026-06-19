# Sections

이 파일은 모든 섹션과 그 순서, 영향도 수준, 설명을 정의한다. 괄호 안의 섹션 ID는 rule을 그룹화하는 데 사용되는 파일명 접두사이다.

---

## 1. Complexity & Context (Cynefin) (complexity)

**Impact:** HIGH  
**Description:** 문제를 얼마나 잘 이해하고 있는지에 coupling 전략을 맞추며, Cynefin 도메인을 사용해 사전 구조화와 결정 보류 사이를 선택한다.

## 2. Coupling Types & Threat Ranking (coupling)

**Impact:** CRITICAL  
**Description:** 해로운 coupling을 식별하고 줄인다; 현대적 위협 순서는 Control > External > Common > Contents > Stamp > Data이다.

## 3. Dependency Direction & Structure (dependency)

**Impact:** CRITICAL  
**Description:** dependency를 단방향이고 비순환적이며 변경 빈도별로 격리되도록 유지해, 변동이 잦은 부분이 안정적인 부분을 끌고 가지 못하게 한다.

## 4. Abstraction & Module Boundary (abstraction)

**Impact:** HIGH  
**Description:** 모듈 경계 전반에 일관되고 최소이며 추상화된 지식을 공개해, 호출자가 구현이 아니라 의도에 의존하도록 한다.

## 5. Layered & Monorepo Architecture (architecture)

**Impact:** MEDIUM  
**Description:** layered 구조와 Turbo monorepo를 적용해 app에서 공유 package로 흐르는 단방향 dependency 흐름을 구성한다.

## 6. AI-Friendly Ownership (ai)

**Impact:** MEDIUM  
**Description:** AI agent가 제한된 context window 안에서 격리된 부분을 소유하고 수정할 수 있도록 코드를 구조화한다.
