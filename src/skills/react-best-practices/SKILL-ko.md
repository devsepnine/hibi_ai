---
name: vercel-react-best-practices
description: React/Next.js performance optimization (Vercel Engineering) — data-fetching waterfalls, bundle size, server/client fetching, re-render and rendering patterns. Use when writing, reviewing, or refactoring React/Next.js for performance. 리액트 성능 최적화, Next.js 최적화, 번들 최적화.
keywords: [react, 리액트, nextjs, performance, 성능최적화, bundle, 번들]
license: MIT
metadata:
  author: vercel
  version: "1.0.0"
---

# Vercel React Best Practices

Vercel이 유지 관리하는 React 및 Next.js 애플리케이션 성능 최적화 종합 가이드. 8개 카테고리에 걸쳐 67개 규칙을 임팩트 우선순위로 정리하여 자동화된 리팩토링과 코드 생성을 안내한다.

## 적용 시점

다음 상황에서 이 가이드를 참조한다:
- 새 React 컴포넌트나 Next.js 페이지를 작성할 때
- 데이터 페칭 구현 (클라이언트 또는 서버 사이드)
- 성능 이슈를 위한 코드 리뷰
- 기존 React/Next.js 코드 리팩토링
- 번들 크기 또는 로드 타임 최적화

## 우선순위별 규칙 카테고리

| Priority | Category | Impact | Prefix |
|----------|----------|--------|--------|
| 1 | Eliminating Waterfalls | CRITICAL | `async-` |
| 2 | Bundle Size Optimization | CRITICAL | `bundle-` |
| 3 | Server-Side Performance | HIGH | `server-` |
| 4 | Client-Side Data Fetching | MEDIUM-HIGH | `client-` |
| 5 | Re-render Optimization | MEDIUM | `rerender-` |
| 6 | Rendering Performance | MEDIUM | `rendering-` |
| 7 | JavaScript Performance | LOW-MEDIUM | `js-` |
| 8 | Advanced Patterns | LOW | `advanced-` |

## Quick Reference

### 1. Eliminating Waterfalls (CRITICAL)

- `async-cheap-condition-before-await` - 플래그나 원격 값을 await 하기 전에 저렴한 동기 조건을 먼저 검사한다
- `async-defer-await` - 실제로 사용되는 분기 안으로 await를 옮긴다
- `async-parallel` - 독립적 연산에는 Promise.all()을 사용한다
- `async-dependencies` - 부분 의존성에는 better-all을 사용한다
- `async-api-routes` - API 라우트에서 promise를 일찍 시작하고 늦게 await 한다
- `async-suspense-boundaries` - Suspense로 콘텐츠를 스트리밍한다
- `async-optimistic-ui` - Action 내부에서 `useOptimistic`을 써서 자동 롤백 가능한 무지연 mutation을 만든다

### 2. Bundle Size Optimization (CRITICAL)

- `bundle-barrel-imports` - 직접 import하고 barrel 파일은 피한다
- `bundle-dynamic-imports` - 무거운 컴포넌트는 next/dynamic을 사용한다
- `bundle-defer-third-party` - 분석/로깅은 hydration 이후에 로드한다
- `bundle-conditional` - 기능이 활성화될 때만 모듈을 로드한다
- `bundle-preload` - 체감 속도를 위해 hover/focus 시점에 preload 한다

### 3. Server-Side Performance (HIGH)

- `server-auth-actions` - 서버 액션을 API 라우트처럼 인증한다
- `server-cache-react` - 요청 단위 dedup을 위해 React.cache()를 사용한다
- `server-cache-lru` - 요청 간 캐싱에는 LRU 캐시를 사용한다
- `server-dedup-props` - RSC props에서 직렬화 중복을 피한다
- `server-hoist-static-io` - 정적 I/O(폰트, 로고)는 모듈 레벨로 끌어올린다
- `server-serialization` - 클라이언트 컴포넌트로 전달하는 데이터를 최소화한다
- `server-parallel-fetching` - fetch 병렬화를 위해 컴포넌트를 재구성한다
- `server-parallel-nested-fetching` - 중첩 fetch는 항목별로 Promise.all로 묶는다
- `server-after-nonblocking` - 논블로킹 작업에 after()를 사용한다

### 4. Client-Side Data Fetching (MEDIUM-HIGH)

- `client-swr-dedup` - SWR로 자동 요청 dedup을 적용한다
- `client-event-listeners` - 전역 이벤트 리스너를 dedup 한다
- `client-passive-event-listeners` - 스크롤에는 passive 리스너를 쓴다
- `client-localstorage-schema` - localStorage 데이터를 버저닝하고 최소화한다

### 5. Re-render Optimization (MEDIUM)

- `rerender-defer-reads` - 콜백에서만 쓰는 state는 구독하지 않는다
- `rerender-memo` - 비싼 작업은 메모이즈된 컴포넌트로 추출한다
- `rerender-memo-with-default-value` - non-primitive 기본 props는 hoist 한다
- `rerender-dependencies` - effect에서는 primitive 의존성을 사용한다
- `rerender-derived-state` - 원시값이 아니라 derived boolean을 구독한다
- `rerender-derived-state-no-effect` - effect가 아닌 render 중에 state를 derive 한다
- `rerender-functional-setstate` - 안정적 콜백을 위해 functional setState를 쓴다
- `rerender-lazy-state-init` - 비싼 값은 useState에 함수를 전달한다
- `rerender-simple-expression-in-memo` - 단순 primitive에는 memo를 쓰지 않는다
- `rerender-split-combined-hooks` - 의존성이 독립적인 hook은 분리한다
- `rerender-move-effect-to-event` - 인터랙션 로직은 이벤트 핸들러에 둔다
- `rerender-transitions` - 긴급하지 않은 업데이트에는 startTransition을 쓴다
- `rerender-use-deferred-value` - 입력 반응성을 유지하도록 비싼 렌더는 defer 한다
- `rerender-use-ref-transient-values` - 자주 바뀌는 transient 값에는 ref를 쓴다
- `rerender-no-inline-components` - 컴포넌트 안에 컴포넌트를 정의하지 않는다

### 6. Rendering Performance (MEDIUM)

- `rendering-animate-svg-wrapper` - SVG 요소가 아니라 div 래퍼를 애니메이션한다
- `rendering-content-visibility` - 긴 리스트에는 content-visibility를 쓴다
- `rendering-hoist-jsx` - 정적 JSX는 컴포넌트 외부로 추출한다
- `rendering-svg-precision` - SVG 좌표 정밀도를 줄인다
- `rendering-hydration-no-flicker` - 클라이언트 전용 데이터에는 inline script를 쓴다
- `rendering-hydration-suppress-warning` - 예상된 mismatch는 suppress 한다
- `rendering-activity` - 표시/숨김에는 Activity 컴포넌트를 사용한다
- `rendering-conditional-render` - 조건부 렌더에는 &&가 아니라 삼항을 쓴다
- `rendering-document-metadata` - 명령형 head 변경 대신 React 19의 네이티브 `<title>`/`<meta>`/`<link>` hoisting을 사용한다
- `rendering-usetransition-loading` - 로딩 상태에는 useTransition을 선호한다
- `rendering-resource-hints` - preload에는 React DOM resource hints를 쓴다
- `rendering-script-defer-async` - script 태그에는 defer 또는 async를 사용한다

### 7. JavaScript Performance (LOW-MEDIUM)

- `js-batch-dom-css` - CSS 변경은 클래스나 cssText로 묶는다
- `js-index-maps` - 반복 조회에는 Map을 만든다
- `js-cache-property-access` - 루프에서 객체 프로퍼티는 캐시한다
- `js-cache-function-results` - 함수 결과는 모듈 레벨 Map에 캐시한다
- `js-cache-storage` - localStorage/sessionStorage 읽기는 캐시한다
- `js-combine-iterations` - 여러 filter/map은 한 루프로 결합한다
- `js-length-check-first` - 비싼 비교 전에 배열 length를 먼저 확인한다
- `js-early-exit` - 함수에서 일찍 return 한다
- `js-hoist-regexp` - RegExp 생성을 루프 밖으로 hoist 한다
- `js-min-max-loop` - min/max는 sort 대신 루프로 처리한다
- `js-set-map-lookups` - O(1) 조회에는 Set/Map을 쓴다
- `js-tosorted-immutable` - 불변성을 위해 toSorted()를 쓴다
- `js-flatmap-filter` - 한 번에 map+filter 하려면 flatMap을 쓴다
- `js-request-idle-callback` - 비핵심 작업은 브라우저 idle 시간으로 미룬다

### 8. Advanced Patterns (LOW)

- `advanced-event-handler-refs` - 이벤트 핸들러는 ref에 저장한다
- `advanced-init-once` - 앱 로드당 한 번만 초기화한다
- `advanced-use-latest` - 안정적인 콜백 ref용 useLatest

## 사용 방법

자세한 설명과 코드 예제는 개별 규칙 파일을 읽는다:

```
rules/async-parallel.md
rules/bundle-barrel-imports.md
```

각 규칙 파일에는 다음이 들어 있다:
- 왜 중요한지에 대한 간단한 설명
- 잘못된 코드 예제와 설명
- 올바른 코드 예제와 설명
- 추가 컨텍스트와 참고 자료

## 전체 통합 문서

모든 규칙을 펼친 완전한 가이드: `AGENTS.md`
