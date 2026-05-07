# Sections

이 파일은 모든 섹션과 그 정렬 순서, 영향도(impact level), 설명을 정의한다.
괄호 안의 섹션 ID는 규칙 그룹화에 쓰이는 파일명 prefix다.

---

## 1. Core Rendering (rendering)

**Impact:** CRITICAL  
**Description:** React Native의 핵심 렌더링 규칙. 위반 시 런타임 크래시
또는 깨진 UI가 발생한다.

## 2. List Performance (list-performance)

**Impact:** HIGH  
**Description:** 가상화 리스트(FlatList, LegendList, FlashList)를 매끄러운
스크롤과 빠른 업데이트를 위해 최적화한다.

## 3. Animation (animation)

**Impact:** HIGH  
**Description:** GPU 가속 애니메이션, Reanimated 패턴, 제스처 중 렌더 thrashing
회피.

## 4. Scroll Performance (scroll)

**Impact:** HIGH  
**Description:** 렌더 thrashing 없이 스크롤 위치를 추적한다.

## 5. Navigation (navigation)

**Impact:** HIGH  
**Description:** stack과 tab navigation에 JS 기반 대안 대신 native navigator
사용.

## 6. React State (react-state)

**Impact:** MEDIUM  
**Description:** stale closure와 불필요한 re-render를 피하는 React state
관리 패턴.

## 7. State Architecture (state)

**Impact:** MEDIUM  
**Description:** state 변수와 derived value에 대한 ground truth 원칙.

## 8. React Compiler (react-compiler)

**Impact:** MEDIUM  
**Description:** React Compiler를 React Native와 Reanimated와 함께 쓰기 위한
호환성 패턴.

## 9. User Interface (ui)

**Impact:** MEDIUM  
**Description:** 이미지, menu, modal, styling, 플랫폼 일관 인터페이스를 위한
네이티브 UI 패턴.

## 10. Design System (design-system)

**Impact:** MEDIUM  
**Description:** 유지보수 가능한 컴포넌트 라이브러리를 위한 아키텍처 패턴.

## 11. Monorepo (monorepo)

**Impact:** LOW  
**Description:** monorepo에서 의존성 관리와 native module 설정.

## 12. Third-Party Dependencies (imports)

**Impact:** LOW  
**Description:** 유지보수성을 위해 서드파티 의존성을 wrapping하고 re-export한다.

## 13. JavaScript (js)

**Impact:** LOW  
**Description:** 비싼 객체 생성의 호이스팅 같은 마이크로 최적화.

## 14. Fonts (fonts)

**Impact:** LOW  
**Description:** 성능 향상을 위한 native font loading.
