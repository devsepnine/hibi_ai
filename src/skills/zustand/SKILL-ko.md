---
name: zustand-store-ts
description: Create Zustand v5 stores with TypeScript — proper state/action separation, useShallow for object/array selectors, subscribeWithSelector for outside-React subscriptions, devtools + persist middleware, and slices pattern for larger stores. Use when building React state management, migrating from v4, fixing "infinite loop" selector bugs, or setting up modular stores. 주스탄드 상태관리, Zustand v5 스토어 작성, 전역 상태 관리, useShallow, slices pattern.
keywords: [zustand, 주스탄드, state-management, 상태관리, store, 스토어, useShallow, slices, v5, middleware, devtools, persist]
---

# Zustand Store (v5)

TypeScript 타입, 적절한 미들웨어, 흔한 함정(infinite re-render, zombie children, stale closure)을 피하는 selector 패턴과 함께 v5 베스트 프랙티스를 따라 Zustand store를 작성한다.

**Zustand v5+**를 가정한다. v4 마이그레이션 포인터는 [references/acceptance-criteria.md](references/acceptance-criteria.md) 참고.

## Quick Start

[assets/template.md](assets/template.md)에서 템플릿을 복사하고 다음을 치환:
- `{{StoreName}}` → PascalCase store 이름 (예: `Project`)
- `{{description}}` → 한 줄 JSDoc description

## State / Actions 분리

store 타입을 **state**(데이터)와 **actions**(mutator)로 분리한다. 이렇게 하면 의도가 명확해지고, selector를 위한 `Pick<>`이 쉬워지며, 절대 re-render하지 않는 action-only selector가 드러난다.

```typescript
export interface MyState {
  items: Item[];
  selectedId: string | null;
  isLoading: boolean;
  error: string | null;
}

export interface MyActions {
  addItem: (item: Item) => void;
  removeItem: (id: string) => void;
  loadItems: () => Promise<void>;
  reset: () => void;
}

export type MyStore = MyState & MyActions;
```

## 선택: 객체/배열에는 `useShallow`

v5는 React 기본 reference-equality 동작과 일치한다. 즉, **렌더마다 새 배열이나 객체를 반환하는** selector는 무한 렌더 루프를 일으킨다. `useShallow`로 해결한다.

```typescript
import { useShallow } from 'zustand/shallow';

// ❌ BAD — new array each render → infinite loop in v5
const [items, addItem] = useMyStore((s) => [s.items, s.addItem]);

// ✅ GOOD — stable reference via shallow compare
const [items, addItem] = useMyStore(
  useShallow((s) => [s.items, s.addItem]),
);

// ✅ GOOD — individual selectors (no wrapper needed)
const items = useMyStore((s) => s.items);
const addItem = useMyStore((s) => s.addItem);
```

엄지 손가락 규칙: **단일 필드 selector는 wrapper가 필요 없다**; 튜플, 배열, 객체 리터럴을 반환할 때는 `useShallow`로 감싼다.

v4의 equality-function API를 선호한다면 `zustand/traditional`의 `createWithEqualityFn`을 쓸 수 있다 — 하지만 `useShallow`가 v5의 권장 경로다.

## `subscribeWithSelector`: outside-React 전용

`subscribeWithSelector` 미들웨어는 `.subscribe()`에 selector + equality 인자를 추가한다. React 바깥(이벤트 브릿지, 로깅, URL 동기화)에서 store를 subscribe해야 할 때 **만** 포함시킨다. 컴포넌트 안에서는 `useShallow`가 같은 필요를 커버한다.

```typescript
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

export const useMyStore = create<MyStore>()(
  subscribeWithSelector((set, get) => ({
    /* state + actions */
  })),
);

// Subscribe to a single slice outside React
const unsub = useMyStore.subscribe(
  (s) => s.selectedId,
  (id) => console.log('selected changed:', id),
);
```

store가 React 안에서만 소비된다면 표면적을 줄이기 위해 미들웨어를 빼라.

## Devtools + Persist

`create<T>()(...)` 안에서 미들웨어를 합성한다. 순서가 중요하다: 바깥 wrapper가 최종 state를 본다. 흔한 체인: `devtools(persist(...))`.

```typescript
import { create } from 'zustand';
import { devtools, persist, createJSONStorage } from 'zustand/middleware';
import type {} from '@redux-devtools/extension'; // devtools typing

export const useMyStore = create<MyStore>()(
  devtools(
    persist(
      (set, get) => ({ /* state + actions */ }),
      {
        name: 'my-store',
        storage: createJSONStorage(() => localStorage),
        partialize: (s) => ({ selectedId: s.selectedId }), // only persist picks
      },
    ),
    { name: 'MyStore' }, // devtools instance name
  ),
);
```

가이드라인:
- devtools 트레이싱을 위해 모든 `set()`에 이름을 붙여라: `set(partial, false, 'action/name')`
- transient state(loading, error)를 storage에서 제외하려면 `partialize`를 사용한다
- 사용자 prefs는 `localStorage`, tab 범위 state는 `sessionStorage`를 선호한다

## 초기 state + reset

`reset()`과 v5의 `store.getInitialState()`가 동기화 상태로 유지되도록 `initialState`를 한 번만 추출한다.

```typescript
const initialState: MyState = {
  items: [], selectedId: null, isLoading: false, error: null,
};

// inside the store:
reset: () => set(initialState),

// or, v5 API — good for "reset all stores on logout":
useMyStore.setState(useMyStore.getInitialState(), true);
```

## Slices 패턴 (큰 store)

store가 ~5개 action을 넘거나 여러 도메인에 걸치면, typed slice로 분할해 합성한다. 각 slice는 cross-slice `get()`이 동작하도록 최종 store shape으로 파라미터화된 `StateCreator`다.

짧은 버전:
```typescript
import { create, StateCreator } from 'zustand';

const createBearSlice: StateCreator<Bear & Fish, [], [], Bear> = (set) => ({ ... });
const createFishSlice: StateCreator<Bear & Fish, [], [], Fish> = (set) => ({ ... });

export const useJungleStore = create<Bear & Fish>()((...a) => ({
  ...createBearSlice(...a),
  ...createFishSlice(...a),
}));
```

전체 가이드 + devtools 통합 + cross-slice 패턴: [references/slices-pattern.md](references/slices-pattern.md).

## 테스트

- 테스트 사이에 store reset: `useMyStore.setState(useMyStore.getInitialState(), true)`
- async action은 `await` 후 `store.getState()`로 assert
- fetch/IO는 boundary에서 mock; store에서 mock하지 않는다
- 다중 store 테스트 스위트는 global-reset wrapper를 도입한다 (slices 가이드 참고)

## 통합 단계

1. `src/frontend/src/store/<domain>.ts`에 store 생성
2. `src/frontend/src/store/index.ts`에서 export
3. `src/frontend/src/store/<domain>.test.ts`에 테스트 추가
4. store에 devtools가 필요하면 Redux DevTools 익스텐션이 설치돼 있는지 확인
