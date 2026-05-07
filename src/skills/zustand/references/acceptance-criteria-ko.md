# Zustand Store Acceptance Criteria (TypeScript, v5)

**Library**: Zustand v5+
**Purpose**: 생성된 Zustand 코드를 현재 (v5) 모범 사례와 대조하여 검증한다. v4에서의 변경 사항에 대한 Migration 섹션을 포함한다.

---

## 1. Import Patterns

### 1.1 ✅ CORRECT

```typescript
import { create } from 'zustand';
import { useShallow } from 'zustand/shallow';
import {
  devtools,
  persist,
  createJSONStorage,
  subscribeWithSelector,
} from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import type {} from '@redux-devtools/extension'; // for devtools typing
```

### 1.2 ❌ INCORRECT

```typescript
// Default import removed in v4+
import create from 'zustand';

// Old path — useShallow moved to 'zustand/shallow' in v5
import { useShallow } from 'zustand/react/shallow';

// `shallow` as selector equality arg removed from v5 `create`
// (use `useShallow` hook, or `createWithEqualityFn` from `zustand/traditional`)
import { shallow } from 'zustand/shallow';
```

---

## 2. Store Creation

### 2.1 ✅ CORRECT: Plain store (no middleware needed)

```typescript
import { create } from 'zustand';

interface MyStore {
  count: number;
  increment: () => void;
}

export const useMyStore = create<MyStore>()((set) => ({
  count: 0,
  increment: () => set((s) => ({ count: s.count + 1 })),
}));
```

`subscribeWithSelector`는 **필수가 아니다**. store가 React 외부에서 subscribe될 때만 추가한다 (§5 참조).

### 2.2 ✅ CORRECT: With `get()`

```typescript
export const useMyStore = create<MyStore>()((set, get) => ({
  count: 0,
  increment: () => set((s) => ({ count: s.count + 1 })),
  double: () => set({ count: get().count * 2 }),
}));
```

### 2.3 ✅ CORRECT: Generic syntax with middleware

**double parentheses**에 주의 — `create<T>()(middleware(...))`. 빈 쌍이 middleware를 통한 type inference를 가능하게 한다.

```typescript
export const useMyStore = create<MyStore>()(
  devtools((set) => ({ /* ... */ })),
);
```

### 2.4 ❌ INCORRECT: Missing `()` on generic with middleware

```typescript
// Generic inference breaks — always use create<T>()(...) shape
export const useMyStore = create<MyStore>(
  devtools((set) => ({ /* ... */ })),
);
```

---

## 3. State and Actions Separation

### 3.1 ✅ CORRECT

```typescript
export interface ProjectState {
  projects: Project[];
  selectedId: string | null;
  isLoading: boolean;
}

export interface ProjectActions {
  addProject: (project: Project) => void;
  selectProject: (id: string) => void;
  loadProjects: () => Promise<void>;
  reset: () => void;
}

export type ProjectStore = ProjectState & ProjectActions;

const initialState: ProjectState = {
  projects: [],
  selectedId: null,
  isLoading: false,
};

export const useProjectStore = create<ProjectStore>()((set, get) => ({
  ...initialState,
  addProject: (project) =>
    set((s) => ({ projects: [...s.projects, project] })),
  selectProject: (selectedId) => set({ selectedId }),
  loadProjects: async () => {
    set({ isLoading: true });
    try {
      const projects = await fetchProjects();
      set({ projects, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },
  reset: () => set(initialState),
}));
```

---

## 4. Selector Patterns

### 4.1 ✅ CORRECT: Single-field selectors

개별 selector가 가장 안전한 형태이다 — wrapper가 필요 없다.

```typescript
const count = useMyStore((s) => s.count);
const increment = useMyStore((s) => s.increment);
```

### 4.2 ✅ CORRECT: `useShallow` for arrays, tuples, objects

v5는 reference equality를 사용하므로, *매 render마다 새 array/object*를 반환하는 selector는 무한 루프를 일으킨다. `useShallow`로 wrap한다.

```typescript
import { useShallow } from 'zustand/shallow';

// Object literal — wrap
const { count, isLoading } = useMyStore(
  useShallow((s) => ({ count: s.count, isLoading: s.isLoading })),
);

// Tuple/array — wrap
const [count, increment] = useMyStore(
  useShallow((s) => [s.count, s.increment]),
);
```

### 4.3 ❌ INCORRECT: Unwrapped multi-field selector

```typescript
// Causes infinite re-render in v5
const { count, isLoading } = useMyStore((s) => ({
  count: s.count,
  isLoading: s.isLoading,
}));
```

### 4.4 ❌ INCORRECT: Destructuring the whole store

```typescript
// Re-renders on any state change
const { count, isLoading } = useMyStore();
```

### 4.5 Alternative: `createWithEqualityFn`

기존 코드베이스가 `create(..., shallow)` equality에 의존한다면, 모든 호출 사이트를 리팩토링하기보다 특정 store를 `zustand/traditional`의 `createWithEqualityFn`으로 마이그레이션한다.

```typescript
import { createWithEqualityFn as create } from 'zustand/traditional';
import { shallow } from 'zustand/shallow';

const useStore = createWithEqualityFn<MyStore>()((set) => ({ /* ... */ }));

// v4-style selector still works
const { count, text } = useStore((s) => ({ count: s.count, text: s.text }), shallow);
```

새 코드는 `useShallow`를 선호해야 한다.

---

## 5. Subscribe Outside React (`subscribeWithSelector`)

React 외부에서 subscribe할 때만 (event bridge, 로깅, URL sync, 분석) `subscribeWithSelector`를 포함한다. 그렇지 않으면 건너뛴다.

### 5.1 ✅ CORRECT

```typescript
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

export const useMyStore = create<MyStore>()(
  subscribeWithSelector((set) => ({ /* ... */ })),
);

const unsubscribe = useMyStore.subscribe(
  (s) => s.selectedId,
  (selectedId, prev) => console.log('selected:', prev, '->', selectedId),
  { fireImmediately: true, equalityFn: Object.is },
);
```

---

## 6. Persist Middleware

### 6.1 ✅ CORRECT: With partialize

```typescript
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set) => ({
      theme: 'dark',
      tempToken: null, // transient, don't persist
      setTheme: (theme) => set({ theme }),
    }),
    {
      name: 'settings-storage',
      storage: createJSONStorage(() => localStorage),
      partialize: (s) => ({ theme: s.theme }), // persist only `theme`
    },
  ),
);
```

### 6.2 Versioning & migration

```typescript
persist(
  (set) => ({ /* ... */ }),
  {
    name: 'settings-storage',
    version: 2,
    migrate: (persisted: any, fromVersion) => {
      if (fromVersion < 2) return { ...persisted, theme: persisted.color };
      return persisted;
    },
  },
);
```

---

## 7. Devtools Middleware

### 7.1 ✅ CORRECT

```typescript
import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import type {} from '@redux-devtools/extension';

export const useMyStore = create<MyStore>()(
  devtools(
    (set) => ({
      count: 0,
      increment: () =>
        set((s) => ({ count: s.count + 1 }), false, 'MyStore/increment'),
    }),
    { name: 'MyStore' },
  ),
);
```

모든 `set()` 호출에 이름을 준다 — 세 번째 인자가 Redux DevTools에 표시된다.

### 7.2 Compose order (common chain)

```typescript
// devtools wraps persist wraps the state creator
create<MyStore>()(devtools(persist((set) => ({ /* ... */ }), { name: '...' })));
```

---

## 8. Immer Middleware

### 8.1 ✅ CORRECT

```typescript
import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';

export const useTodoStore = create<TodoStore>()(
  immer((set) => ({
    todos: [],
    addTodo: (todo) =>
      set((s) => {
        s.todos.push(todo); // mutation ok inside immer
      }),
    toggleTodo: (id) =>
      set((s) => {
        const todo = s.todos.find((t) => t.id === id);
        if (todo) todo.completed = !todo.completed;
      }),
  })),
);
```

---

## 9. Async Actions

### 9.1 ✅ CORRECT

```typescript
export const useDataStore = create<DataStore>()((set) => ({
  data: null,
  isLoading: false,
  error: null,

  fetchData: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      const data = await api.getData(id);
      set({ data, isLoading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Request failed',
        isLoading: false,
      });
    }
  },
}));
```

시작 시 항상 `error`를 reset하고, 읽기 쉬운 메시지를 표면화하며, 모든 종료 경로에서 loading을 클리어한다.

---

## 10. Slices Pattern

모듈식 store에 `StateCreator<Full, Mutators, UnusedMutators, Slice>`를 사용한다. 전체 가이드는 [slices-pattern.md](./slices-pattern.md) 참조.

### 10.1 ✅ CORRECT

```typescript
import { create, StateCreator } from 'zustand';

interface UISlice {
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
}

interface DataSlice {
  items: Item[];
  addItem: (item: Item) => void;
}

type AppStore = UISlice & DataSlice;

const createUISlice: StateCreator<AppStore, [], [], UISlice> = (set) => ({
  isSidebarOpen: true,
  toggleSidebar: () => set((s) => ({ isSidebarOpen: !s.isSidebarOpen })),
});

const createDataSlice: StateCreator<AppStore, [], [], DataSlice> = (set) => ({
  items: [],
  addItem: (item) => set((s) => ({ items: [...s.items, item] })),
});

export const useAppStore = create<AppStore>()((...a) => ({
  ...createUISlice(...a),
  ...createDataSlice(...a),
}));
```

### 10.2 ❌ INCORRECT: Old `SetState<T>` signature

```typescript
// Outdated — use StateCreator<Full, [], [], Slice> instead
const createUISlice = (set: SetState<AppStore>): UISlice => ({ /* ... */ });
```

---

## 11. Testing Patterns

### 11.1 ✅ CORRECT: Reset between tests via `getInitialState()`

```typescript
beforeEach(() => {
  // v5 API — no need to duplicate initialState manually
  useMyStore.setState(useMyStore.getInitialState(), true);
});

test('increment advances count', () => {
  useMyStore.getState().increment();
  expect(useMyStore.getState().count).toBe(1);
});
```

### 11.2 ✅ CORRECT: Global reset registry (e.g., on logout)

```typescript
import { create as actualCreate, type StateCreator } from 'zustand';

const storeResetFns = new Set<() => void>();

export const resetAllStores = () => {
  storeResetFns.forEach((fn) => fn());
};

export const create = (<T>() => (creator: StateCreator<T>) => {
  const store = actualCreate<T>()(creator);
  storeResetFns.add(() => store.setState(store.getInitialState(), true));
  return store;
}) as typeof actualCreate;
```

---

## 12. Migration: v4 → v5

| v4 | v5 | Notes |
|----|----|-------|
| `create(..., shallow)` in selector | `useShallow((s) => ...)` hook | 권장 |
| `create(..., shallow)` in selector | `createWithEqualityFn` + `shallow` | 레거시 경로, `zustand/traditional` 사용 |
| `from 'zustand/react/shallow'` | `from 'zustand/shallow'` | 경로 변경 |
| 수동 `initialState` 캡처 | `store.getInitialState()` | 새 v5 API |
| wrapper 없는 모든 object/array selector | `useShallow`로 wrap | v5는 더 이상 기본적으로 memoize하지 않는다 |

핵심 동작 변경: v5는 React의 기본 reference equality와 일치하므로, fresh reference를 반환하는 모든 selector는 wrap되어야 한다.

---

## 13. Anti-Patterns Summary

```typescript
// ❌ Default import (pre-v4)
import create from 'zustand';

// ❌ Whole-store destructure — re-renders on every state change
const { count, items } = useStore();

// ❌ Unwrapped multi-field selector — infinite loop in v5
const data = useStore((s) => ({ count: s.count }));

// ❌ Unwrapped tuple selector — infinite loop in v5
const [a, b] = useStore((s) => [s.a, s.b]);

// ❌ Old useShallow path
import { useShallow } from 'zustand/react/shallow';

// ❌ Mutating without immer
set((s) => { s.items.push(item); return s; });

// ❌ Old SetState<T> slice signature
const createSlice = (set: SetState<Store>): Slice => ({ /* ... */ });

// ❌ Missing double parentheses with middleware
create<Store>(devtools(/* ... */)); // should be create<Store>()(devtools(/* ... */))
```
