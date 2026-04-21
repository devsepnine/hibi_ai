---
name: zustand-store-ts
description: Create Zustand v5 stores with TypeScript — proper state/action separation, useShallow for object/array selectors, subscribeWithSelector for outside-React subscriptions, devtools + persist middleware, and slices pattern for larger stores. Use when building React state management, migrating from v4, fixing "infinite loop" selector bugs, or setting up modular stores. 주스탄드 상태관리, Zustand v5 스토어 작성, 전역 상태 관리, useShallow, slices pattern.
keywords: [zustand, 주스탄드, state-management, 상태관리, store, 스토어, useShallow, slices, v5, middleware, devtools, persist]
---

# Zustand Store (v5)

Create Zustand stores following v5 best practices with TypeScript types,
appropriate middleware, and selector patterns that avoid common pitfalls
(infinite re-renders, zombie children, stale closures).

Assumes **Zustand v5+**. For v4 migration pointers, see
[references/acceptance-criteria.md](references/acceptance-criteria.md).

## Quick Start

Copy the template from [assets/template.md](assets/template.md) and replace:
- `{{StoreName}}` → PascalCase store name (e.g., `Project`)
- `{{description}}` → one-line JSDoc description

## State / Actions Separation

Split the store type into **state** (data) and **actions** (mutators). This
keeps intent obvious, makes `Pick<>` for selectors trivial, and surfaces
action-only selectors (which never re-render).

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

## Selecting: `useShallow` for objects/arrays

v5 matches React's default reference-equality behavior, which means a
selector that returns a **new array or object on every render** causes an
infinite render loop. Fix it with `useShallow`.

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

Rule of thumb: **single-field selectors need no wrapper**; any time you return
a tuple, array, or object literal, wrap with `useShallow`.

If you prefer v4's equality-function API, use `createWithEqualityFn` from
`zustand/traditional` — but `useShallow` is the preferred v5 path.

## `subscribeWithSelector`: outside-React only

The `subscribeWithSelector` middleware adds a selector+equality argument to
`.subscribe()`. Include it **only** when the store needs to be subscribed to
outside React (event bridges, logging, URL sync). Inside components,
`useShallow` covers the same need.

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

If the store is only consumed inside React, drop the middleware for less
surface area.

## Devtools + Persist

Compose middleware inside `create<T>()(...)`. Order matters: outer wrapper
sees final state. Common chain: `devtools(persist(...))`.

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

Guidelines:
- Name every `set()` for devtools tracing: `set(partial, false, 'action/name')`
- Use `partialize` to exclude transient state (loading, error) from storage
- Prefer `localStorage` for user prefs, `sessionStorage` for tab-scoped state

## Initial state + reset

Extract `initialState` once so `reset()` and v5's `store.getInitialState()`
stay in sync.

```typescript
const initialState: MyState = {
  items: [], selectedId: null, isLoading: false, error: null,
};

// inside the store:
reset: () => set(initialState),

// or, v5 API — good for "reset all stores on logout":
useMyStore.setState(useMyStore.getInitialState(), true);
```

## Slices pattern (larger stores)

When a store grows past ~5 actions or spans multiple domains, split into
typed slices and compose. Each slice is a `StateCreator` parameterized by
the final store shape so cross-slice `get()` works.

Short version:
```typescript
import { create, StateCreator } from 'zustand';

const createBearSlice: StateCreator<Bear & Fish, [], [], Bear> = (set) => ({ ... });
const createFishSlice: StateCreator<Bear & Fish, [], [], Fish> = (set) => ({ ... });

export const useJungleStore = create<Bear & Fish>()((...a) => ({
  ...createBearSlice(...a),
  ...createFishSlice(...a),
}));
```

Full guide + devtools integration + cross-slice patterns:
[references/slices-pattern.md](references/slices-pattern.md).

## Testing

- Reset the store between tests: `useMyStore.setState(useMyStore.getInitialState(), true)`
- For async actions, assert on `store.getState()` after `await`
- Mock fetch/IO at the boundary, not the store
- For multi-store test suites, adopt the global-reset wrapper (see slices guide)

## Integration Steps

1. Create store under `src/frontend/src/store/<domain>.ts`
2. Export from `src/frontend/src/store/index.ts`
3. Add tests at `src/frontend/src/store/<domain>.test.ts`
4. If the store needs devtools, confirm Redux DevTools extension is installed
