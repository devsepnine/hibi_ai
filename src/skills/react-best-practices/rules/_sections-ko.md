# Sections

이 파일은 모든 섹션, 정렬 순서, 영향도, 설명을 정의한다.
괄호 안의 섹션 ID는 규칙을 그룹화하는 파일명 prefix이다.

---

## 1. Eliminating Waterfalls (async)

**Impact:** CRITICAL  
**Description:** 워터폴은 성능을 가장 크게 저해하는 요인이다. 순차 await 하나마다 풀 네트워크 지연이 누적된다. 이를 제거하면 가장 큰 성능 향상을 얻는다.

## 2. Bundle Size Optimization (bundle)

**Impact:** CRITICAL  
**Description:** 초기 번들 크기를 줄이면 Time to Interactive와 Largest Contentful Paint가 개선된다.

## 3. Server-Side Performance (server)

**Impact:** HIGH  
**Description:** 서버 측 렌더링과 데이터 페칭을 최적화하면 서버 측 워터폴이 사라지고 응답 시간이 줄어든다.

## 4. Client-Side Data Fetching (client)

**Impact:** MEDIUM-HIGH  
**Description:** 자동 중복 제거와 효율적인 데이터 페칭 패턴은 중복 네트워크 요청을 줄인다.

## 5. Re-render Optimization (rerender)

**Impact:** MEDIUM  
**Description:** 불필요한 리렌더링을 줄이면 낭비되는 연산이 줄고 UI 반응성이 향상된다.

## 6. Rendering Performance (rendering)

**Impact:** MEDIUM  
**Description:** 렌더링 과정을 최적화하면 브라우저가 처리해야 할 작업량이 줄어든다.

## 7. JavaScript Performance (js)

**Impact:** LOW-MEDIUM  
**Description:** 핫 패스에 대한 마이크로 최적화가 누적되면 의미 있는 개선이 된다.

## 8. Advanced Patterns (advanced)

**Impact:** LOW  
**Description:** 신중한 구현이 필요한 특정 케이스를 위한 고급 패턴이다.
